use serde::Serialize;

/// One input a form must collect.
///
/// Shared between connector descriptors and auth schemes on purpose: a single
/// shape means a single rendering component in the frontend. If a connector
/// field and an auth field ever needed different concepts, the React editor
/// would need two form engines instead of one.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Field {
    pub name: &'static str,
    pub label: &'static str,
    pub required: bool,
    pub kind: FieldKind,
    /// This field accepts a `{{ ... }}` expression.
    pub expression: bool,
    /// Masked in the UI and never returned by the API once stored. True on
    /// auth scheme fields, almost never on a connector field.
    pub secret: bool,
    pub visible_when: Option<VisibleWhen>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum FieldKind {
    Text,
    Number,
    Bool,
    Select { options: &'static [SelectOption] },
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct SelectOption {
    pub value: &'static str,
    pub label: &'static str,
}

/// Shown only when another field of the same form equals one of `any_of`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct VisibleWhen {
    pub field: &'static str,
    pub any_of: &'static [&'static str],
}
