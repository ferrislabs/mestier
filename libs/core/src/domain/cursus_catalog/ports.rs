use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{
    ClosureReason, ClosureReasonId, Establishment, EstablishmentFilter, EstablishmentId,
    OrganizationId, Promotion, PromotionFilter, PromotionId, Training, TrainingFilter, TrainingId,
};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait CursusCatalogRepository: Send {
    fn insert_establishment(
        &mut self,
        establishment: &Establishment,
    ) -> impl Future<Output = Result<Establishment, CoreError>> + Send;

    fn find_establishment(
        &mut self,
        organization_id: OrganizationId,
        id: EstablishmentId,
    ) -> impl Future<Output = Result<Option<Establishment>, CoreError>> + Send;

    fn list_establishments(
        &mut self,
        organization_id: OrganizationId,
        filter: &EstablishmentFilter,
    ) -> impl Future<Output = Result<Vec<Establishment>, CoreError>> + Send;

    fn update_establishment(
        &mut self,
        establishment: &Establishment,
    ) -> impl Future<Output = Result<Establishment, CoreError>> + Send;

    fn archive_establishment(
        &mut self,
        organization_id: OrganizationId,
        id: EstablishmentId,
        archived_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<Establishment, CoreError>> + Send;

    fn insert_training(
        &mut self,
        training: &Training,
    ) -> impl Future<Output = Result<Training, CoreError>> + Send;

    fn find_training(
        &mut self,
        organization_id: OrganizationId,
        id: TrainingId,
    ) -> impl Future<Output = Result<Option<Training>, CoreError>> + Send;

    fn list_trainings(
        &mut self,
        organization_id: OrganizationId,
        filter: &TrainingFilter,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<Training>, u64), CoreError>> + Send;

    fn update_training(
        &mut self,
        training: &Training,
    ) -> impl Future<Output = Result<Training, CoreError>> + Send;

    fn archive_training(
        &mut self,
        organization_id: OrganizationId,
        id: TrainingId,
        archived_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<Training, CoreError>> + Send;

    fn insert_promotion(
        &mut self,
        promotion: &Promotion,
    ) -> impl Future<Output = Result<Promotion, CoreError>> + Send;

    fn find_promotion(
        &mut self,
        organization_id: OrganizationId,
        id: PromotionId,
    ) -> impl Future<Output = Result<Option<Promotion>, CoreError>> + Send;

    fn list_promotions(
        &mut self,
        organization_id: OrganizationId,
        filter: &PromotionFilter,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<Promotion>, u64), CoreError>> + Send;

    fn update_promotion(
        &mut self,
        promotion: &Promotion,
    ) -> impl Future<Output = Result<Promotion, CoreError>> + Send;

    fn archive_promotion(
        &mut self,
        organization_id: OrganizationId,
        id: PromotionId,
        archived_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<Promotion, CoreError>> + Send;

    fn insert_closure_reason(
        &mut self,
        closure_reason: &ClosureReason,
    ) -> impl Future<Output = Result<ClosureReason, CoreError>> + Send;

    fn find_closure_reason(
        &mut self,
        organization_id: OrganizationId,
        id: ClosureReasonId,
    ) -> impl Future<Output = Result<Option<ClosureReason>, CoreError>> + Send;

    fn list_closure_reasons(
        &mut self,
        organization_id: OrganizationId,
        include_archived: bool,
    ) -> impl Future<Output = Result<Vec<ClosureReason>, CoreError>> + Send;

    fn update_closure_reason(
        &mut self,
        closure_reason: &ClosureReason,
    ) -> impl Future<Output = Result<ClosureReason, CoreError>> + Send;

    fn archive_closure_reason(
        &mut self,
        organization_id: OrganizationId,
        id: ClosureReasonId,
        archived_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<ClosureReason, CoreError>> + Send;
}
