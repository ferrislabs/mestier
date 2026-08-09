use serde_json::Value;

use super::ast::{Template, TemplateKind};
use super::context::ExpressionContext;
use super::error::ExpressionError;

impl Template {
    pub fn evaluate(&self, ctx: &ExpressionContext<'_>) -> Result<Value, ExpressionError> {
        let _ = ctx;
        match self.kind() {
            TemplateKind::Literal(value) => Ok(value.clone()),
        }
    }
}
