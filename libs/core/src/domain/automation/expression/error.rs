use std::fmt;

/// Everything that can go wrong parsing or evaluating a template.
///
/// A missing path is never a silent `null`: [`ExpressionError::MissingPath`]
/// names exactly what was not found, whether that surfaces statically (#199,
/// before any run exists) or at evaluation (#200). `default(v, fallback)` is
/// the one place that catches it on purpose.
///
/// `Display`/`Error` are hand-written rather than derived with `thiserror`:
/// `mestier-core` does not depend on `thiserror` today (only `common` and
/// `events` do), and this ticket's file ownership does not extend to
/// `Cargo.toml`. This type is a drop-in equivalent of what a
/// `#[derive(thiserror::Error)]` would have produced from the frozen
/// contract — same variants, same fields, same messages.
#[derive(Debug, PartialEq, Eq)]
pub enum ExpressionError {
    Syntax {
        position: usize,
        message: String,
    },
    UnknownFunction {
        name: String,
    },
    Arity {
        function: String,
        expected: usize,
        got: usize,
    },
    MissingPath {
        path: String,
    },
    TypeMismatch {
        path: String,
        expected: &'static str,
        got: &'static str,
    },
    LoopOutsideLoop,
    TooLarge {
        limit: usize,
    },
    UnknownNode {
        name: String,
    },
    NoItem {
        name: String,
    },
    LegacyConnectorsRoot {
        position: usize,
    },
}

impl fmt::Display for ExpressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax { position, message } => {
                write!(f, "syntax error at position {position}: {message}")
            }
            Self::UnknownFunction { name } => write!(f, "unknown function `{name}`"),
            Self::Arity {
                function,
                expected,
                got,
            } => write!(f, "`{function}` expects {expected} argument(s), got {got}"),
            Self::MissingPath { path } => write!(f, "missing path `{path}`"),
            Self::TypeMismatch {
                path,
                expected,
                got,
            } => write!(f, "`{path}` expected {expected}, got {got}"),
            Self::LoopOutsideLoop => write!(f, "`loop` is only valid inside a `flow.loop`"),
            Self::TooLarge { limit } => write!(f, "the value built exceeds {limit} bytes"),
            Self::UnknownNode { name } => {
                write!(
                    f,
                    "node `{name}` has no output available to this expression"
                )
            }
            Self::NoItem { name } => write!(f, "node `{name}` produced no item"),
            Self::LegacyConnectorsRoot { position } => write!(
                f,
                "cette expression utilise l'ancienne syntaxe `connectors.<id>.output` (position {position}) : utilise $('Nom du nœud').item.json…"
            ),
        }
    }
}

impl std::error::Error for ExpressionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_syntax_error_message_names_its_position() {
        let error = ExpressionError::Syntax {
            position: 17,
            message: "unexpected token".to_string(),
        };

        assert!(
            error.to_string().contains("17"),
            "the message must name the position: {error}"
        );
    }

    #[test]
    fn a_missing_path_error_message_names_the_path() {
        let error = ExpressionError::MissingPath {
            path: "$('Devis').item.json.total".to_string(),
        };

        assert!(
            error.to_string().contains("$('Devis').item.json.total"),
            "the message must name the path: {error}"
        );
    }

    #[test]
    fn a_node_error_names_the_node() {
        let unknown = ExpressionError::UnknownNode {
            name: "Rechercher".to_string(),
        };
        let empty = ExpressionError::NoItem {
            name: "Rechercher".to_string(),
        };

        assert!(unknown.to_string().contains("Rechercher"), "{unknown}");
        assert!(empty.to_string().contains("Rechercher"), "{empty}");
    }

    #[test]
    fn the_legacy_root_error_names_the_new_syntax() {
        let error = ExpressionError::LegacyConnectorsRoot { position: 3 };

        let message = error.to_string();
        assert!(
            message.contains("cette expression utilise l'ancienne syntaxe"),
            "{message}"
        );
        assert!(message.contains("$('Nom du nœud').item.json"), "{message}");
    }
}
