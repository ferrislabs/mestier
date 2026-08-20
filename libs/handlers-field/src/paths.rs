use axum_extra::routing::TypedPath;
use mestier_core::{OrganizationId, TimeEntryId};
use serde::Deserialize;

/// The jobs assigned to the caller for a day.
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/field/tasks")]
pub struct FieldTasksPath {
    pub organization_id: OrganizationId,
}

/// What the caller is clocked on to right now, if anything.
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/field/current")]
pub struct FieldCurrentPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/field/time-entries")]
pub struct FieldTimeEntriesPath {
    pub organization_id: OrganizationId,
}

/// Declaring a stretch of work that was never clocked live at all.
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/field/time-entries/declare")]
pub struct FieldDeclarePath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/field/time-entries/{time_entry_id}/stop")]
pub struct FieldStopPath {
    pub time_entry_id: TimeEntryId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/field/time-entries/{time_entry_id}/photos")]
pub struct FieldPhotosPath {
    pub time_entry_id: TimeEntryId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/field/day-end")]
pub struct FieldDayEndPath {
    pub organization_id: OrganizationId,
}

/// Closing a stretch the employee forgot, at the time they now declare.
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/field/time-entries/{time_entry_id}/recover")]
pub struct FieldRecoverPath {
    pub time_entry_id: TimeEntryId,
}
