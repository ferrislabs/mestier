use serde_json::Value;

/// What one field's raw JSON value compiles down to.
///
/// `Interpolated` (a string mixing text and expressions) is not built yet:
/// `parse_template` grows that case as the grammar is completed.
#[derive(Debug)]
pub(crate) enum TemplateKind {
    /// The raw value was not a string, or was a string with no `{{ }}` in
    /// it: nothing to evaluate, ever.
    Literal(Value),
    /// The string, once trimmed, is exactly one `{{ ... }}`: evaluating it
    /// keeps the expression's own JSON type, rather than stringifying it.
    Whole(Expr),
}

/// One chunk of a template string while it is being split: literal text, or
/// an embedded `{{ ... }}` expression.
#[derive(Debug)]
pub(crate) enum Part {
    Text(String),
    Expr(Expr),
}

/// A parsed `{{ ... }}` body. Only literals exist yet — paths, operators and
/// function calls are added as the grammar is built out.
#[derive(Debug)]
pub(crate) enum Expr {
    Literal(Value),
}

/// Compiled form of one field's raw JSON value. Opaque by design — evaluate
/// it, or ask the static-analysis questions #199 needs; nothing else about
/// its shape is public. `Debug` is derived only so tests can use
/// `expect_err`/`unwrap_err`; it exposes nothing callers should depend on.
#[derive(Debug)]
pub struct Template {
    kind: TemplateKind,
}

impl Template {
    pub(crate) fn literal(value: Value) -> Self {
        Self {
            kind: TemplateKind::Literal(value),
        }
    }

    pub(crate) fn whole(expr: Expr) -> Self {
        Self {
            kind: TemplateKind::Whole(expr),
        }
    }

    pub(crate) fn kind(&self) -> &TemplateKind {
        &self.kind
    }
}
