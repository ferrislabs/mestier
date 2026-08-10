//! The run engine's domain: the durable, resumable record of one workflow
//! execution (#200).
//!
//! One row per step, not one growing JSONB document — see the migration
//! `20260810000003_create_automation_runs` for the reasoning — so a failure
//! at element 300 of a 500-element `flow.loop` resumes at 300 instead of
//! rewriting everything already done. Getting there costs the engine an
//! invariant the rest of this module leans on: **a step that already
//! succeeded is never re-executed.** Resuming means reading its persisted
//! `output` back, never recomputing it.
//!
//! There is no exactly-once here. A step is marked `in_flight` *before* it
//! runs, which is what makes the crash-recovery story ("a worker died mid
//! step") safe: the row survives even if the process does not. But nothing
//! stops a process from dying *after* a connector's real-world effect and
//! *before* the row that would have recorded it — that attempt is retried,
//! and the effect runs again. `application::automation::run` and the
//! connectors in `infrastructure::automation::connectors` inherit this the
//! same way `DeliveryHandler::deliver` does for a delivery.
//!
//! This module owns the pure pieces: the model
//! ([`Run`], [`RunStep`] and their statuses), the connector contract any
//! executable connector implements ([`Connector`], [`ConnectorInput`],
//! [`ConnectorOutcome`]), and the graph-walking helpers
//! (`graph`) the engine composes into a traversal. It does no I/O and calls
//! no connector — that is `application::automation::run`'s job, the only
//! layer allowed to mix persistence, the network (later, #202) and control
//! flow.

mod connector;
mod graph;
mod model;

pub use connector::{Connector, ConnectorInput, ConnectorOutcome};
pub use graph::{
    LoopStackEntry, branch_targets, connectors_by_id, descendants_via, format_iteration_path,
    resolve_config, roots,
};
pub use model::{DueRun, Run, RunSettlement, RunStatus, RunStep, StepStatus};
