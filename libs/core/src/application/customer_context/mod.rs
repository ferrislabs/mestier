use authz::Subject;
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
    #[transactional(customer_context, customer, role, member, authz)]
    pub async fn create_customer_context(
        &self,
        command: CreateCustomerContextCommand,
    ) -> Result<CustomerContext, CoreError> {
        let mut service = CustomerContextService::new(
            customer_context_repository,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.create_customer_context(command).await
    }

    #[transactional(customer_context, customer, role, member, authz)]
    pub async fn get_customer_context(
        &self,
        id: CustomerContextId,
    ) -> Result<CustomerContext, CoreError> {
        let mut service = CustomerContextService::new(
            customer_context_repository,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.get_customer_context(id).await
    }

    #[transactional(customer_context, customer, role, member, authz)]
    pub async fn list_customer_contexts(
        &self,
        customer_id: CustomerId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<CustomerContext>, u64), CoreError> {
        let mut service = CustomerContextService::new(
            customer_context_repository,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service
            .list_customer_contexts(customer_id, limit, offset)
            .await
    }

    #[transactional(customer_context, customer, role, member, authz)]
    pub async fn update_customer_context(
        &self,
        command: UpdateCustomerContextCommand,
    ) -> Result<CustomerContext, CoreError> {
        let mut service = CustomerContextService::new(
            customer_context_repository,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.update_customer_context(command).await
    }

    #[transactional(customer_context, customer, role, member, authz)]
    pub async fn soft_delete_customer_context(
        &self,
        id: CustomerContextId,
        actor: Subject,
    ) -> Result<(), CoreError> {
        let mut service = CustomerContextService::new(
            customer_context_repository,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.soft_delete_customer_context(id, actor).await
    }
}
