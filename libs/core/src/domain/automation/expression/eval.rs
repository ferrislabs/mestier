use serde_json::Value;

use super::ast::{Expr, Template, TemplateKind};
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
    let _ = ctx;
    match expr {
        Expr::Literal(value) => Ok(value.clone()),
    }
}
