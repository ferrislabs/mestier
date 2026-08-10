use serde_json::Value;

use super::ast::{Expr, Part, Path, PathRoot, PathSegment, Template};
use super::error::ExpressionError;

/// Entry point used by `parse_template`. A non-string value is always a
/// static literal; a string is scanned for `{{ }}` expressions.
pub(crate) fn parse(raw: &Value) -> Result<Template, ExpressionError> {
    match raw {
        Value::String(s) => parse_string_template(s),
        other => Ok(Template::literal(other.clone())),
    }
}

/// Splits `s` into literal text and `{{ }}` expressions, then decides which
/// `TemplateKind` it is:
/// - no expression at all -> a literal;
/// - trimmed down to exactly one expression -> whole, keeping its type;
/// - anything else -> interpolated, always producing a string (not built
///   yet).
fn parse_string_template(s: &str) -> Result<Template, ExpressionError> {
    let mut parts: Vec<Part> = Vec::new();
    let mut cursor = 0usize;

    while let Some(relative_start) = s[cursor..].find("{{") {
        let brace_start = cursor + relative_start;
        if brace_start > cursor {
            parts.push(Part::Text(s[cursor..brace_start].to_string()));
        }

        let expr_start = brace_start + "{{".len();
        let (expr, next_cursor) = parse_one_expression(s, expr_start, brace_start)?;
        parts.push(Part::Expr(expr));
        cursor = next_cursor;
    }

    if cursor < s.len() {
        parts.push(Part::Text(s[cursor..].to_string()));
    }

    let expr_count = parts.iter().filter(|p| matches!(p, Part::Expr(_))).count();

    if expr_count == 0 {
        return Ok(Template::literal(Value::String(s.to_string())));
    }

    if expr_count == 1 {
        let only_whitespace_around_it = parts.iter().all(|part| match part {
            Part::Text(text) => text.trim().is_empty(),
            Part::Expr(_) => true,
        });
        if only_whitespace_around_it {
            if let Some(expr) = parts.into_iter().find_map(|part| match part {
                Part::Expr(expr) => Some(expr),
                Part::Text(_) => None,
            }) {
                return Ok(Template::whole(expr));
            }
        }
    }

    unimplemented!("interpolated templates are not built yet")
}

/// Parses one `{{ ... }}` body starting right after the opening brace.
/// `brace_start` is the position of the `{{` itself, reported when the
/// expression is never closed. Returns the parsed expression and the byte
/// offset right after the closing `}}`.
fn parse_one_expression(
    s: &str,
    expr_start: usize,
    brace_start: usize,
) -> Result<(Expr, usize), ExpressionError> {
    let lexer = Lexer::new(s, expr_start);
    let mut parser = Parser::new(lexer)?;
    let expr = parser.parse_expression()?;

    match parser.cur.tok.clone() {
        Tok::End => Ok((expr, parser.lexer.pos)),
        Tok::Eof => Err(ExpressionError::Syntax {
            position: brace_start,
            message: "unterminated `{{`".to_string(),
        }),
        _ => Err(ExpressionError::Syntax {
            position: parser.cur.pos,
            message: "expected `}}`".to_string(),
        }),
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Ident(String),
    Number(String),
    Str(String),
    True,
    False,
    Null,
    And,
    Or,
    Not,
    Dot,
    Comma,
    LParen,
    RParen,
    LBracket,
    RBracket,
    EqOp,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    /// The closing `}}` of the current template expression.
    End,
    Eof,
}

#[derive(Debug, Clone)]
struct Spanned {
    tok: Tok,
    pos: usize,
}

/// Tokenizes one `{{ ... }}` body directly out of the surrounding template
/// string, so reported positions are byte offsets into what the user typed,
/// not into some extracted substring. String literals are lexed fully
/// (quotes and all) before `}}` is ever looked for, so a literal like
/// `"a}}b"` cannot be mistaken for the end of the expression.
struct Lexer<'a> {
    s: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(s: &'a str, pos: usize) -> Self {
        Self { s, pos }
    }

    fn peek_char(&self) -> Option<char> {
        self.s[self.pos..].chars().next()
    }

    fn peek_char_at(&self, n: usize) -> Option<char> {
        self.s[self.pos..].chars().nth(n)
    }

    fn bump_char(&mut self) -> Option<char> {
        let c = self.peek_char()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.bump_char();
            } else {
                break;
            }
        }
    }

    fn next_token(&mut self) -> Result<Spanned, ExpressionError> {
        self.skip_whitespace();
        let start = self.pos;

        if self.s[self.pos..].starts_with("}}") {
            self.pos += 2;
            return Ok(Spanned {
                tok: Tok::End,
                pos: start,
            });
        }

        let c = match self.peek_char() {
            None => {
                return Ok(Spanned {
                    tok: Tok::Eof,
                    pos: start,
                })
            }
            Some(c) => c,
        };

        match c {
            '(' => {
                self.bump_char();
                Ok(Spanned {
                    tok: Tok::LParen,
                    pos: start,
                })
            }
            ')' => {
                self.bump_char();
                Ok(Spanned {
                    tok: Tok::RParen,
                    pos: start,
                })
            }
            '[' => {
                self.bump_char();
                Ok(Spanned {
                    tok: Tok::LBracket,
                    pos: start,
                })
            }
            ']' => {
                self.bump_char();
                Ok(Spanned {
                    tok: Tok::RBracket,
                    pos: start,
                })
            }
            '.' => {
                self.bump_char();
                Ok(Spanned {
                    tok: Tok::Dot,
                    pos: start,
                })
            }
            ',' => {
                self.bump_char();
                Ok(Spanned {
                    tok: Tok::Comma,
                    pos: start,
                })
            }
            '=' => {
                self.bump_char();
                if self.peek_char() == Some('=') {
                    self.bump_char();
                    Ok(Spanned {
                        tok: Tok::EqOp,
                        pos: start,
                    })
                } else {
                    Err(ExpressionError::Syntax {
                        position: start,
                        message: "expected `==`".to_string(),
                    })
                }
            }
            '!' => {
                self.bump_char();
                if self.peek_char() == Some('=') {
                    self.bump_char();
                    Ok(Spanned {
                        tok: Tok::Ne,
                        pos: start,
                    })
                } else {
                    Err(ExpressionError::Syntax {
                        position: start,
                        message: "expected `!=`".to_string(),
                    })
                }
            }
            '<' => {
                self.bump_char();
                if self.peek_char() == Some('=') {
                    self.bump_char();
                    Ok(Spanned {
                        tok: Tok::Le,
                        pos: start,
                    })
                } else {
                    Ok(Spanned {
                        tok: Tok::Lt,
                        pos: start,
                    })
                }
            }
            '>' => {
                self.bump_char();
                if self.peek_char() == Some('=') {
                    self.bump_char();
                    Ok(Spanned {
                        tok: Tok::Ge,
                        pos: start,
                    })
                } else {
                    Ok(Spanned {
                        tok: Tok::Gt,
                        pos: start,
                    })
                }
            }
            '"' | '\'' => self.lex_string(c, start),
            '-' if self.peek_char_at(1).is_some_and(|c2| c2.is_ascii_digit()) => {
                self.lex_number(start)
            }
            c if c.is_ascii_digit() => self.lex_number(start),
            c if c.is_alphabetic() || c == '_' => Ok(self.lex_ident(start)),
            other => Err(ExpressionError::Syntax {
                position: start,
                message: format!("unexpected character `{other}`"),
            }),
        }
    }

    fn lex_string(&mut self, quote: char, start: usize) -> Result<Spanned, ExpressionError> {
        self.bump_char();
        let mut out = String::new();
        loop {
            match self.bump_char() {
                None => {
                    return Err(ExpressionError::Syntax {
                        position: start,
                        message: "unterminated string literal".to_string(),
                    })
                }
                Some(c) if c == quote => break,
                Some('\\') => match self.bump_char() {
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some('r') => out.push('\r'),
                    Some('\\') => out.push('\\'),
                    Some('"') => out.push('"'),
                    Some('\'') => out.push('\''),
                    Some(other) => {
                        return Err(ExpressionError::Syntax {
                            position: start,
                            message: format!("unknown escape sequence `\\{other}`"),
                        })
                    }
                    None => {
                        return Err(ExpressionError::Syntax {
                            position: start,
                            message: "unterminated string literal".to_string(),
                        })
                    }
                },
                Some(c) => out.push(c),
            }
        }
        Ok(Spanned {
            tok: Tok::Str(out),
            pos: start,
        })
    }

    fn lex_number(&mut self, start: usize) -> Result<Spanned, ExpressionError> {
        let mut text = String::new();
        if self.peek_char() == Some('-') {
            if let Some(c) = self.bump_char() {
                text.push(c);
            }
        }
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                text.push(c);
                self.bump_char();
            } else {
                break;
            }
        }
        if self.peek_char() == Some('.')
            && self.peek_char_at(1).is_some_and(|c| c.is_ascii_digit())
        {
            text.push('.');
            self.bump_char();
            while let Some(c) = self.peek_char() {
                if c.is_ascii_digit() {
                    text.push(c);
                    self.bump_char();
                } else {
                    break;
                }
            }
        }
        Ok(Spanned {
            tok: Tok::Number(text),
            pos: start,
        })
    }

    fn lex_ident(&mut self, start: usize) -> Spanned {
        let mut text = String::new();
        while let Some(c) = self.peek_char() {
            if c.is_alphanumeric() || c == '_' {
                text.push(c);
                self.bump_char();
            } else {
                break;
            }
        }
        let tok = match text.as_str() {
            "and" => Tok::And,
            "or" => Tok::Or,
            "not" => Tok::Not,
            "true" => Tok::True,
            "false" => Tok::False,
            "null" => Tok::Null,
            _ => Tok::Ident(text),
        };
        Spanned { tok, pos: start }
    }
}

/// Recursive-descent parser over one `{{ ... }}` body. Only literals are
/// handled yet; paths, operators and function calls are added next.
struct Parser<'a> {
    lexer: Lexer<'a>,
    cur: Spanned,
}

impl<'a> Parser<'a> {
    fn new(mut lexer: Lexer<'a>) -> Result<Self, ExpressionError> {
        let cur = lexer.next_token()?;
        Ok(Self { lexer, cur })
    }

    fn bump(&mut self) -> Result<Spanned, ExpressionError> {
        let next = self.lexer.next_token()?;
        Ok(std::mem::replace(&mut self.cur, next))
    }

    fn parse_expression(&mut self) -> Result<Expr, ExpressionError> {
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, ExpressionError> {
        match self.cur.tok.clone() {
            Tok::Number(text) => {
                let pos = self.cur.pos;
                self.bump()?;
                Ok(Expr::Literal(literal_number(&text, pos)?))
            }
            Tok::Str(s) => {
                self.bump()?;
                Ok(Expr::Literal(Value::String(s)))
            }
            Tok::True => {
                self.bump()?;
                Ok(Expr::Literal(Value::Bool(true)))
            }
            Tok::False => {
                self.bump()?;
                Ok(Expr::Literal(Value::Bool(false)))
            }
            Tok::Null => {
                self.bump()?;
                Ok(Expr::Literal(Value::Null))
            }
            Tok::Ident(name) => self.parse_ident_start(name),
            _ => Err(ExpressionError::Syntax {
                position: self.cur.pos,
                message: "expected a value".to_string(),
            }),
        }
    }

    /// An identifier starts either a path (`trigger`, `connectors`, `loop`)
    /// or a function call. Function calls are not parsed yet, so anything
    /// else is a syntax error naming the unknown identifier.
    fn parse_ident_start(&mut self, name: String) -> Result<Expr, ExpressionError> {
        let start_pos = self.cur.pos;
        let root = match name.as_str() {
            "trigger" => Some(PathRoot::Trigger),
            "connectors" => Some(PathRoot::Connectors),
            "loop" => Some(PathRoot::Loop),
            _ => None,
        };

        if let Some(root) = root {
            self.bump()?;
            let segments = self.parse_path_segments()?;
            if matches!(root, PathRoot::Connectors) && segments.is_empty() {
                return Err(ExpressionError::Syntax {
                    position: start_pos,
                    message: "`connectors` requires a connector id, e.g. `connectors.c1`"
                        .to_string(),
                });
            }
            return Ok(Expr::Path(Path { root, segments }));
        }

        Err(ExpressionError::Syntax {
            position: start_pos,
            message: format!(
                "unknown identifier `{name}`; expected `trigger`, `connectors`, `loop`"
            ),
        })
    }

    /// Consumes every trailing `.field` and `[index]` segment after a path
    /// root. Stops, without consuming, at the first token that is neither.
    fn parse_path_segments(&mut self) -> Result<Vec<PathSegment>, ExpressionError> {
        let mut segments = Vec::new();
        loop {
            match self.cur.tok.clone() {
                Tok::Dot => {
                    self.bump()?;
                    match self.cur.tok.clone() {
                        Tok::Ident(name) => {
                            self.bump()?;
                            segments.push(PathSegment::Field(name));
                        }
                        _ => {
                            return Err(ExpressionError::Syntax {
                                position: self.cur.pos,
                                message: "expected a field name after `.`".to_string(),
                            })
                        }
                    }
                }
                Tok::LBracket => {
                    self.bump()?;
                    match self.cur.tok.clone() {
                        Tok::Number(text) if is_index_literal(&text) => {
                            let index: usize = text.parse().map_err(|_| {
                                ExpressionError::Syntax {
                                    position: self.cur.pos,
                                    message: format!("invalid index `{text}`"),
                                }
                            })?;
                            self.bump()?;
                            self.expect_rbracket()?;
                            segments.push(PathSegment::Index(index));
                        }
                        _ => {
                            return Err(ExpressionError::Syntax {
                                position: self.cur.pos,
                                message: "expected a non-negative integer index".to_string(),
                            })
                        }
                    }
                }
                _ => break,
            }
        }
        Ok(segments)
    }

    fn expect_rbracket(&mut self) -> Result<(), ExpressionError> {
        if matches!(self.cur.tok, Tok::RBracket) {
            self.bump()?;
            Ok(())
        } else {
            Err(ExpressionError::Syntax {
                position: self.cur.pos,
                message: "expected `]`".to_string(),
            })
        }
    }
}

fn is_index_literal(text: &str) -> bool {
    !text.contains('.') && !text.starts_with('-')
}

/// Renders as an integer when the source had no `.`, as a float otherwise —
/// so a literal `5` round-trips as JSON `5`, not `5.0`.
fn literal_number(text: &str, pos: usize) -> Result<Value, ExpressionError> {
    if text.contains('.') {
        text.parse::<f64>()
            .ok()
            .and_then(serde_json::Number::from_f64)
            .map(Value::Number)
            .ok_or_else(|| ExpressionError::Syntax {
                position: pos,
                message: format!("invalid number `{text}`"),
            })
    } else {
        text.parse::<i64>()
            .ok()
            .map(|n| Value::Number(serde_json::Number::from(n)))
            .ok_or_else(|| ExpressionError::Syntax {
                position: pos,
                message: format!("invalid number `{text}`"),
            })
    }
}
