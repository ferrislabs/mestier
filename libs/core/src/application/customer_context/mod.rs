use common::CoreError;
use mestier_macros::transactional;

use crate::{
    CustomerContext, CustomerContextId, CustomerId,
    application::MestierUseCase,
    domain::customer_context::{
        commands::{CreateCustomerContextCommand, UpdateCustomerContextCommand},
        service::CustomerContextService,
    },
};

impl MestierUseCase {
    #[transactional(customer_context)]
    pub async fn create_customer_context(
        &self,
        command: CreateCustomerContextCommand,
    ) -> Result<CustomerContext, CoreError> {
        let mut service = CustomerContextService::new(customer_context_repository);
        service.create_customer_context(command).await
    }

    #[transactional(customer_context)]
    pub async fn get_customer_context(
        &self,
        id: CustomerContextId,
    ) -> Result<CustomerContext, CoreError> {
        let mut service = CustomerContextService::new(customer_context_repository);
        service.get_customer_context(id).await
    }

    #[transactional(customer_context)]
    pub async fn list_customer_contexts(
        &self,
        customer_id: CustomerId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<CustomerContext>, u64), CoreError> {
        let mut service = CustomerContextService::new(customer_context_repository);
        service
            .list_customer_contexts(customer_id, limit, offset)
            .await
    }

    #[transactional(customer_context)]
    pub async fn update_customer_context(
        &self,
        command: UpdateCustomerContextCommand,
    ) -> Result<CustomerContext, CoreError> {
        let mut service = CustomerContextService::new(customer_context_repository);
        service.update_customer_context(command).await
    }

    #[transactional(customer_context)]
    pub async fn soft_delete_customer_context(
        &self,
        id: CustomerContextId,
    ) -> Result<(), CoreError> {
        let mut service = CustomerContextService::new(customer_context_repository);
        service.soft_delete_customer_context(id).await
    }
}
