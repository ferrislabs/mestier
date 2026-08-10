use serde_json::Value;

use super::ast::{Expr, Path, PathRoot, PathSegment, Template, TemplateKind};
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
