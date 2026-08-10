use serde_json::Value;

use super::ast::{BinOp, Expr, Part, Path, PathRoot, PathSegment, Template, TemplateKind, UnaryOp};
use super::context::ExpressionContext;
use super::error::ExpressionError;

impl Template {
    pub fn evaluate(&self, ctx: &ExpressionContext<'_>) -> Result<Value, ExpressionError> {
        match self.kind() {
            TemplateKind::Literal(value) => Ok(value.clone()),
            TemplateKind::Whole(expr) => eval_expr(expr, ctx),
            TemplateKind::Interpolated(parts) => {
                let mut out = String::new();
                for part in parts {
                    match part {
                        Part::Text(text) => out.push_str(text),
                        Part::Expr(expr) => out.push_str(&stringify(&eval_expr(expr, ctx)?)),
                    }
                }
                Ok(Value::String(out))
            }
        }
    }
}

fn eval_expr(expr: &Expr, ctx: &ExpressionContext<'_>) -> Result<Value, ExpressionError> {
    match expr {
        Expr::Literal(value) => Ok(value.clone()),
        Expr::Path(path) => resolve_path(path, ctx),
        Expr::Unary {
            op: UnaryOp::Not,
            expr: inner,
        } => {
            let value = eval_expr(inner, ctx)?;
            let flag = as_bool(&value, inner)?;
            Ok(Value::Bool(!flag))
        }
        Expr::Binary { op, left, right } => eval_binary(*op, left, right, ctx),
        Expr::Call { name, args } => eval_call(name, args, ctx),
    }
}

/// `and`/`or` short-circuit: the right operand is only evaluated (and only
/// type-checked) when the left one does not already decide the result. A
/// template like `{{ default(x, false) and y }}` relies on this to skip `y`
/// safely when `x` is absent.
fn eval_binary(
    op: BinOp,
    left: &Expr,
    right: &Expr,
    ctx: &ExpressionContext<'_>,
) -> Result<Value, ExpressionError> {
    match op {
        BinOp::And => {
            let flag = as_bool(&eval_expr(left, ctx)?, left)?;
            if !flag {
                return Ok(Value::Bool(false));
            }
            Ok(Value::Bool(as_bool(&eval_expr(right, ctx)?, right)?))
        }
        BinOp::Or => {
            let flag = as_bool(&eval_expr(left, ctx)?, left)?;
            if flag {
                return Ok(Value::Bool(true));
            }
            Ok(Value::Bool(as_bool(&eval_expr(right, ctx)?, right)?))
        }
        BinOp::Eq => Ok(Value::Bool(eval_expr(left, ctx)? == eval_expr(right, ctx)?)),
        BinOp::Ne => Ok(Value::Bool(eval_expr(left, ctx)? != eval_expr(right, ctx)?)),
        BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
            let left_value = eval_expr(left, ctx)?;
            let right_value = eval_expr(right, ctx)?;
            let l = as_number(&left_value, left)?;
            let r = as_number(&right_value, right)?;
            let result = match op {
                BinOp::Lt => l < r,
                BinOp::Le => l <= r,
                BinOp::Gt => l > r,
                BinOp::Ge => l >= r,
                BinOp::Eq | BinOp::Ne | BinOp::And | BinOp::Or => {
                    unreachable!("only ordering operators reach this arm")
                }
            };
            Ok(Value::Bool(result))
        }
    }
}

fn as_bool(value: &Value, source: &Expr) -> Result<bool, ExpressionError> {
    value
        .as_bool()
        .ok_or_else(|| ExpressionError::TypeMismatch {
            path: describe(source),
            expected: "boolean",
            got: type_name(value),
        })
}

fn as_number(value: &Value, source: &Expr) -> Result<f64, ExpressionError> {
    value.as_f64().ok_or_else(|| ExpressionError::TypeMismatch {
        path: describe(source),
        expected: "number",
        got: type_name(value),
    })
}

fn type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Renders an expression back to something readable for `TypeMismatch`,
/// e.g. a literal `1` describes as `1`, a path as its dotted source. There
/// is no span to quote here (only the top-level parser tracks positions),
/// so this is the closest thing to "the offending expression" an error
/// message can name.
fn describe(expr: &Expr) -> String {
    match expr {
        Expr::Literal(value) => value.to_string(),
        Expr::Path(path) => path.to_source(),
        Expr::Unary { op, expr } => format!("{} {}", op.keyword(), describe(expr)),
        Expr::Binary { op, left, right } => {
            format!("({} {} {})", describe(left), op.symbol(), describe(right))
        }
        Expr::Call { name, .. } => format!("{name}(...)"),
    }
}

/// Dispatches one of the fifteen functions from the frozen grammar. The
/// parser has already checked `name` and `args.len()` against
/// `function_arity`, so every arm below can index `args` directly.
///
/// `default` is the one function that does not evaluate its arguments
/// eagerly: it evaluates `args[0]` first and only falls back to `args[1]`
/// when that specifically raises `MissingPath` — any other error, including
/// a type mismatch or (were it possible here) a bad path deeper down, is
/// propagated rather than swallowed.
fn eval_call(
    name: &str,
    args: &[Expr],
    ctx: &ExpressionContext<'_>,
) -> Result<Value, ExpressionError> {
    match name {
        "default" => match eval_expr(&args[0], ctx) {
            Ok(value) => Ok(value),
            Err(ExpressionError::MissingPath { .. }) => eval_expr(&args[1], ctx),
            Err(other) => Err(other),
        },
        "upper" => Ok(Value::String(
            as_string(&eval_expr(&args[0], ctx)?, &args[0])?.to_uppercase(),
        )),
        "lower" => Ok(Value::String(
            as_string(&eval_expr(&args[0], ctx)?, &args[0])?.to_lowercase(),
        )),
        "trim" => Ok(Value::String(
            as_string(&eval_expr(&args[0], ctx)?, &args[0])?
                .trim()
                .to_string(),
        )),
        "len" => {
            let value = eval_expr(&args[0], ctx)?;
            let count = match &value {
                Value::String(s) => s.chars().count(),
                Value::Array(a) => a.len(),
                Value::Object(o) => o.len(),
                other => {
                    return Err(ExpressionError::TypeMismatch {
                        path: describe(&args[0]),
                        expected: "string, array or object",
                        got: type_name(other),
                    });
                }
            };
            Ok(Value::Number(serde_json::Number::from(count)))
        }
        "contains" => {
            let haystack = eval_expr(&args[0], ctx)?;
            let needle = eval_expr(&args[1], ctx)?;
            match &haystack {
                Value::String(s) => {
                    let needle = as_string(&needle, &args[1])?;
                    Ok(Value::Bool(s.contains(&needle)))
                }
                Value::Array(items) => Ok(Value::Bool(items.contains(&needle))),
                other => Err(ExpressionError::TypeMismatch {
                    path: describe(&args[0]),
                    expected: "string or array",
                    got: type_name(other),
                }),
            }
        }
        "starts_with" => {
            let s = as_string(&eval_expr(&args[0], ctx)?, &args[0])?;
            let prefix = as_string(&eval_expr(&args[1], ctx)?, &args[1])?;
            Ok(Value::Bool(s.starts_with(&prefix)))
        }
        "to_number" => {
            let value = eval_expr(&args[0], ctx)?;
            match &value {
                Value::Number(_) => Ok(value),
                Value::String(s) => {
                    parse_number_string(s).ok_or_else(|| ExpressionError::TypeMismatch {
                        path: describe(&args[0]),
                        expected: "a numeric string",
                        got: "string",
                    })
                }
                other => Err(ExpressionError::TypeMismatch {
                    path: describe(&args[0]),
                    expected: "number or numeric string",
                    got: type_name(other),
                }),
            }
        }
        "to_string" => Ok(Value::String(stringify(&eval_expr(&args[0], ctx)?))),
        "round" => eval_round(args, ctx),
        "concat" => {
            let mut out = String::new();
            for arg in args {
                out.push_str(&stringify(&eval_expr(arg, ctx)?));
            }
            Ok(Value::String(out))
        }
        "now" => Ok(Value::String(ctx.now.to_rfc3339())),
        "format_date" => {
            let s = as_string(&eval_expr(&args[0], ctx)?, &args[0])?;
            let fmt = as_string(&eval_expr(&args[1], ctx)?, &args[1])?;
            let parsed = chrono::DateTime::parse_from_rfc3339(&s).map_err(|_| {
                ExpressionError::TypeMismatch {
                    path: describe(&args[0]),
                    expected: "an RFC 3339 datetime string",
                    got: "string",
                }
            })?;
            Ok(Value::String(parsed.format(&fmt).to_string()))
        }
        "json" => Ok(Value::String(
            serde_json::to_string(&eval_expr(&args[0], ctx)?).unwrap_or_default(),
        )),
        other => unreachable!(
            "the parser only ever builds Expr::Call for a known function; got `{other}`"
        ),
    }
}

fn eval_round(args: &[Expr], ctx: &ExpressionContext<'_>) -> Result<Value, ExpressionError> {
    let n = as_number(&eval_expr(&args[0], ctx)?, &args[0])?;
    let decimals_value = eval_expr(&args[1], ctx)?;
    let decimals_f = as_number(&decimals_value, &args[1])?;
    if decimals_f < 0.0 || decimals_f.fract() != 0.0 {
        return Err(ExpressionError::TypeMismatch {
            path: describe(&args[1]),
            expected: "a non-negative integer",
            got: type_name(&decimals_value),
        });
    }
    let decimals = decimals_f as i32;
    let factor = 10f64.powi(decimals);
    let rounded = (n * factor).round() / factor;

    if decimals == 0 {
        Ok(Value::Number(serde_json::Number::from(rounded as i64)))
    } else {
        serde_json::Number::from_f64(rounded)
            .map(Value::Number)
            .ok_or_else(|| ExpressionError::TypeMismatch {
                path: describe(&args[0]),
                expected: "a finite number",
                got: "number",
            })
    }
}

fn as_string(value: &Value, source: &Expr) -> Result<String, ExpressionError> {
    match value {
        Value::String(s) => Ok(s.clone()),
        other => Err(ExpressionError::TypeMismatch {
            path: describe(source),
            expected: "string",
            got: type_name(other),
        }),
    }
}

/// Shared by `to_string`, `concat` and string interpolation: a string passes
/// through unquoted, everything else renders as its JSON form (`json()`
/// implicitly for arrays and objects) — so `{{ concat("items: ", arr) }}`
/// never produces `"[object Object]"`.
fn stringify(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Array(_) | Value::Object(_) => serde_json::to_string(value).unwrap_or_default(),
    }
}

/// Mirrors `literal_number` in the parser: an integer string stays an
/// integer JSON number, so `to_number("42")` compares equal to the literal
/// `42`, not `42.0`.
fn parse_number_string(s: &str) -> Option<Value> {
    let trimmed = s.trim();
    if trimmed.contains('.') {
        trimmed
            .parse::<f64>()
            .ok()
            .and_then(serde_json::Number::from_f64)
            .map(Value::Number)
    } else {
        trimmed
            .parse::<i64>()
            .ok()
            .map(|n| Value::Number(serde_json::Number::from(n)))
    }
}

/// Resolves one path against the context. A path is never partially
/// resolved and reported as `null`: any step that cannot be taken — a
/// missing trigger, an unknown connector id, a missing field, an
/// out-of-bounds index, indexing into the wrong container type — collapses
/// into a single `MissingPath` naming the whole path as written.
fn resolve_path(path: &Path, ctx: &ExpressionContext<'_>) -> Result<Value, ExpressionError> {
    let full_path = path.to_source();

    let (base, remaining): (Value, &[PathSegment]) = match path.root {
        PathRoot::Trigger => {
            let value = ctx.trigger.ok_or_else(|| ExpressionError::MissingPath {
                path: full_path.clone(),
            })?;
            (value.clone(), &path.segments[..])
        }
        PathRoot::Connectors => {
            // The parser guarantees `connectors` always has a leading field
            // segment (the connector id) — see `parse_ident_start`.
            let id = match path.segments.first() {
                Some(PathSegment::Field(id)) => id,
                _ => return Err(ExpressionError::MissingPath { path: full_path }),
            };
            let value = ctx
                .connectors
                .get(id)
                .ok_or_else(|| ExpressionError::MissingPath {
                    path: full_path.clone(),
                })?;
            (value.clone(), &path.segments[1..])
        }
        PathRoot::Loop => {
            let frame = ctx
                .loop_frame
                .as_ref()
                .ok_or(ExpressionError::LoopOutsideLoop)?;
            let mut synthetic = serde_json::Map::new();
            synthetic.insert("item".to_string(), frame.item.clone());
            synthetic.insert("index".to_string(), Value::from(frame.index));
            (Value::Object(synthetic), &path.segments[..])
        }
    };

    walk(base, remaining).ok_or(ExpressionError::MissingPath { path: full_path })
}

/// Walks a resolved base value through the remaining path segments. `None`
/// means the path does not exist from here on — the caller turns that into
/// a `MissingPath` naming the path in full, not just the part that broke.
fn walk(mut current: Value, segments: &[PathSegment]) -> Option<Value> {
    for segment in segments {
        current = match (current, segment) {
            (Value::Object(mut map), PathSegment::Field(name)) => map.remove(name)?,
            (Value::Array(mut array), PathSegment::Index(index)) if *index < array.len() => {
                array.swap_remove(*index)
            }
            _ => return None,
        };
    }
    Some(current)
}
