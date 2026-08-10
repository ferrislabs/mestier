use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde_json::Value;

/// Everything an expression can read while a workflow run is in flight.
///
/// Built fresh by the run engine (#200) for each connector evaluation; this
/// module only consumes it. `now` is injected rather than read from the
/// system clock so a run stays reproducible and a test stays deterministic —
/// the evaluator must never call `Utc::now()` itself.
pub struct ExpressionContext<'a> {
    pub trigger: Option<&'a Value>,
    /// Connector id -> its raw output. `connectors.c1.output` reads
    /// `connectors["c1"]["output"]`.
    pub connectors: &'a BTreeMap<String, Value>,
    pub loop_frame: Option<LoopFrame<'a>>,
    pub now: DateTime<Utc>,
}

/// The current item and position inside an explicit `flow.loop` connector.
/// `None` outside of one, which is what turns `loop.*` into
/// `ExpressionError::LoopOutsideLoop`.
pub struct LoopFrame<'a> {
    pub item: &'a Value,
    pub index: usize,
}
