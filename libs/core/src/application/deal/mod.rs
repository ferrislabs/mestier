use authz::Subject;
use chrono::Utc;
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    Deal, DealId, DealOutcome, OrganizationId, PipelineSummary,
    application::MestierUseCase,
    domain::deal::{
        DealService,
        commands::{CreateDealCommand, DealFilter, MoveDealCommand, UpdateDealCommand},
    },
};

impl MestierUseCase {
    #[transactional(deal, customer, role, member, authz)]
    pub async fn create_deal(
        &self,
        command: CreateDealCommand,
    ) -> Result<DealOutcome<Deal>, CoreError> {
        let mut service = DealService::new(
            deal_repository,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.create_deal(command).await
    }

    #[transactional(deal, customer, role, member, authz)]
    pub async fn get_deal(&self, id: DealId) -> Result<Deal, CoreError> {
        let mut service = DealService::new(
            deal_repository,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.get_deal(id).await
    }

    #[transactional(deal, customer, role, member, authz)]
    pub async fn list_deals(
        &self,
        filter: DealFilter,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Deal>, u64), CoreError> {
        let mut service = DealService::new(
            deal_repository,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.list_deals(filter, limit, offset).await
    }

    #[transactional(deal, customer, role, member, authz)]
    pub async fn update_deal(
        &self,
        command: UpdateDealCommand,
    ) -> Result<DealOutcome<Deal>, CoreError> {
        let mut service = DealService::new(
            deal_repository,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.update_deal(command).await
    }

    #[transactional(deal, customer, role, member, authz)]
    pub async fn move_deal(
        &self,
        command: MoveDealCommand,
    ) -> Result<DealOutcome<Deal>, CoreError> {
        let mut service = DealService::new(
            deal_repository,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.move_deal(command).await
    }

    #[transactional(deal, customer, role, member, authz)]
    pub async fn delete_deal(
        &self,
        id: DealId,
        actor: Subject,
    ) -> Result<DealOutcome<()>, CoreError> {
        let mut service = DealService::new(
            deal_repository,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.delete_deal(id, actor).await
    }

    #[transactional(deal, customer, role, member, authz)]
    pub async fn deal_summary(
        &self,
        organization_id: OrganizationId,
    ) -> Result<PipelineSummary, CoreError> {
        let mut service = DealService::new(
            deal_repository,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.summary(organization_id, Utc::now()).await
    }
}
