use authz::Subject;
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    Customer, CustomerId, OrganizationId,
    application::MestierUseCase,
    domain::customer::{
        commands::{CreateCustomerCommand, UpdateCustomerCommand},
        service::CustomerService,
    },
};

impl MestierUseCase {
    #[transactional(customer, role, member, authz)]
    pub async fn create_customer(
        &self,
        command: CreateCustomerCommand,
    ) -> Result<Customer, CoreError> {
        let mut service = CustomerService::new(
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.create_customer(command).await
    }

    #[transactional(customer, role, member, authz)]
    pub async fn get_customer(&self, id: CustomerId) -> Result<Customer, CoreError> {
        let mut service = CustomerService::new(
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.get_customer(id).await
    }

    #[transactional(customer, role, member, authz)]
    pub async fn list_customers(
        &self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Customer>, u64), CoreError> {
        let mut service = CustomerService::new(
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.list_customers(organization_id, limit, offset).await
    }

    #[transactional(customer, role, member, authz)]
    pub async fn update_customer(
        &self,
        command: UpdateCustomerCommand,
    ) -> Result<Customer, CoreError> {
        let mut service = CustomerService::new(
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.update_customer(command).await
    }

    #[transactional(customer, role, member, authz)]
    pub async fn soft_delete_customer(
        &self,
        id: CustomerId,
        actor: Subject,
    ) -> Result<(), CoreError> {
        let mut service = CustomerService::new(
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.soft_delete_customer(id, actor).await
    }
}
