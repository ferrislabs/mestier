use serde_json::Value;

/// What one field's raw JSON value compiles down to.
///
/// Only [`Literal`](TemplateKind::Literal) exists yet: `parse_template`
/// grows the other cases (a whole `{{ ... }}` expression, and a string that
/// interpolates several of them) as the grammar is built out.
pub(crate) enum TemplateKind {
    /// The raw value was not a string, or was a string with no `{{ }}` in
    /// it: nothing to evaluate, ever.
    Literal(Value),
}

/// Compiled form of one field's raw JSON value. Opaque by design — evaluate
/// it, or ask the static-analysis questions #199 needs; nothing else about
/// its shape is public.
pub struct Template {
    kind: TemplateKind,
}

impl Template {
    pub(crate) fn literal(value: Value) -> Self {
        Self {
            kind: TemplateKind::Literal(value),
        }
    }

    pub(crate) fn kind(&self) -> &TemplateKind {
        &self.kind
    }
}
