use serde_json::Value;

use super::ast::{BinOp, Expr, Path, PathRoot, PathSegment, Template, TemplateKind, UnaryOp};
use super::context::ExpressionContext;
use super::error::ExpressionError;

impl Template {
    pub fn evaluate(&self, ctx: &ExpressionContext<'_>) -> Result<Value, ExpressionError> {
        match self.kind() {
            TemplateKind::Literal(value) => Ok(value.clone()),
            TemplateKind::Whole(expr) => eval_expr(expr, ctx),
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
    value.as_bool().ok_or_else(|| ExpressionError::TypeMismatch {
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
                _ => {
                    return Err(ExpressionError::MissingPath {
                        path: full_path,
                    })
                }
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
            (Value::Array(mut array), PathSegment::Index(index)) => {
                if *index < array.len() {
                    array.swap_remove(*index)
                } else {
                    return None;
                }
            }
            _ => return None,
        };
    }
    Some(current)
}
