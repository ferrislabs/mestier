use common::CoreError;
use mestier_macros::transactional;

use crate::{
    OrganizationId, ServiceRate, ServiceRateId,
    application::MestierUseCase,
    domain::service_rate::{
        commands::{CreateServiceRateCommand, UpdateServiceRateCommand},
        service::ServiceRateService,
    },
};

impl MestierUseCase {
    #[transactional(service_rate)]
    pub async fn create_service_rate(
        &self,
        command: CreateServiceRateCommand,
    ) -> Result<ServiceRate, CoreError> {
        let mut service = ServiceRateService::new(service_rate_repository);
        service.create_service_rate(command).await
    }

    #[transactional(service_rate)]
    pub async fn get_service_rate(&self, id: ServiceRateId) -> Result<ServiceRate, CoreError> {
        let mut service = ServiceRateService::new(service_rate_repository);
        service.get_service_rate(id).await
    }

    #[transactional(service_rate)]
    pub async fn list_service_rates(
        &self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<ServiceRate>, u64), CoreError> {
        let mut service = ServiceRateService::new(service_rate_repository);
        service
            .list_service_rates(organization_id, limit, offset)
            .await
    }

    #[transactional(service_rate)]
    pub async fn update_service_rate(
        &self,
        command: UpdateServiceRateCommand,
    ) -> Result<ServiceRate, CoreError> {
        let mut service = ServiceRateService::new(service_rate_repository);
        service.update_service_rate(command).await
    }

    #[transactional(service_rate)]
    pub async fn soft_delete_service_rate(&self, id: ServiceRateId) -> Result<(), CoreError> {
        let mut service = ServiceRateService::new(service_rate_repository);
        service.soft_delete_service_rate(id).await
    }
}
