use serde_json::Value;

/// What one field's raw JSON value compiles down to.
#[derive(Debug)]
pub(crate) enum TemplateKind {
    /// The raw value was not a string, or was a string with no `{{ }}` in
    /// it: nothing to evaluate, ever.
    Literal(Value),
    /// The string, once trimmed, is exactly one `{{ ... }}`: evaluating it
    /// keeps the expression's own JSON type, rather than stringifying it.
    Whole(Expr),
    /// Anything else: text and expressions are stringified and
    /// concatenated, always producing a string — an array embedded in a
    /// sentence never stays an array.
    Interpolated(Vec<Part>),
}

/// One chunk of a template string while it is being split: literal text, or
/// an embedded `{{ ... }}` expression.
#[derive(Debug)]
pub(crate) enum Part {
    Text(String),
    Expr(Expr),
}

/// A parsed `{{ ... }}` body.
#[derive(Debug)]
pub(crate) enum Expr {
    Literal(Value),
    Path(Path),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnaryOp {
    Not,
}

impl UnaryOp {
    pub(crate) fn keyword(self) -> &'static str {
        match self {
            Self::Not => "not",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BinOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

impl BinOp {
    pub(crate) fn symbol(self) -> &'static str {
        match self {
            Self::Eq => "==",
            Self::Ne => "!=",
            Self::Lt => "<",
            Self::Le => "<=",
            Self::Gt => ">",
            Self::Ge => ">=",
            Self::And => "and",
            Self::Or => "or",
        }
    }
}

/// One reference into `trigger`, `connectors` or `loop`.
///
/// Kept apart from the rest of [`Expr`] because two independent readers need
/// just this piece: the evaluator resolves it against an
/// `ExpressionContext`, and the static pass (`Template::referenced_connectors`,
/// `Template::uses_loop`) only ever looks at the root and the first segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Path {
    pub(crate) root: PathRoot,
    pub(crate) segments: Vec<PathSegment>,
}

impl Path {
    /// Reconstructs the source text, e.g. `connectors.c1.output.lines[0].total`.
    /// Used to name the path in `ExpressionError::MissingPath` and
    /// `TypeMismatch`, so both contexts name it exactly as written.
    pub(crate) fn to_source(&self) -> String {
        let mut out = match self.root {
            PathRoot::Trigger => "trigger".to_string(),
            PathRoot::Connectors => "connectors".to_string(),
            PathRoot::Loop => "loop".to_string(),
        };
        for segment in &self.segments {
            match segment {
                PathSegment::Field(name) => {
                    out.push('.');
                    out.push_str(name);
                }
                PathSegment::Index(index) => {
                    out.push('[');
                    out.push_str(&index.to_string());
                    out.push(']');
                }
            }
        }
        out
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PathRoot {
    Trigger,
    Connectors,
    Loop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PathSegment {
    Field(String),
    Index(usize),
}

/// Visits every [`Path`] reachable from `expr`. The static pass
/// (`referenced_connectors`, `uses_loop`) is built entirely on top of this:
/// it never needs to know how deep a path sits inside the expression tree.
pub(crate) fn walk_paths<'a>(expr: &'a Expr, visit: &mut impl FnMut(&'a Path)) {
    match expr {
        Expr::Literal(_) => {}
        Expr::Path(path) => visit(path),
        Expr::Unary { expr, .. } => walk_paths(expr, visit),
        Expr::Binary { left, right, .. } => {
            walk_paths(left, visit);
            walk_paths(right, visit);
        }
        Expr::Call { args, .. } => {
            for arg in args {
                walk_paths(arg, visit);
            }
        }
    }
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

    pub(crate) fn interpolated(parts: Vec<Part>) -> Self {
        Self {
            kind: TemplateKind::Interpolated(parts),
        }
    }

    pub(crate) fn kind(&self) -> &TemplateKind {
        &self.kind
    }

    /// Every top-level expression this template evaluates. The static pass
    /// walks paths starting from each of these; a plain literal has none.
    pub(crate) fn expr_roots(&self) -> Vec<&Expr> {
        match &self.kind {
            TemplateKind::Literal(_) => Vec::new(),
            TemplateKind::Whole(expr) => vec![expr],
            TemplateKind::Interpolated(parts) => parts
                .iter()
                .filter_map(|part| match part {
                    Part::Expr(expr) => Some(expr),
                    Part::Text(_) => None,
                })
                .collect(),
        }
    }
}
