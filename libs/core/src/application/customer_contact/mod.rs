use authz::Subject;
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    CustomerContact, CustomerContactId, CustomerId,
    application::MestierUseCase,
    domain::customer_contact::{
        commands::{CreateCustomerContactCommand, UpdateCustomerContactCommand},
        service::CustomerContactService,
    },
};

impl MestierUseCase {
    #[transactional(customer_contact, customer, role, member, authz)]
    pub async fn create_customer_contact(
        &self,
        command: CreateCustomerContactCommand,
    ) -> Result<CustomerContact, CoreError> {
        let mut service = CustomerContactService::new(
            customer_contact_repository,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.create_customer_contact(command).await
    }

    #[transactional(customer_contact, customer, role, member, authz)]
    pub async fn get_customer_contact(
        &self,
        id: CustomerContactId,
    ) -> Result<CustomerContact, CoreError> {
        let mut service = CustomerContactService::new(
            customer_contact_repository,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.get_customer_contact(id).await
    }

    #[transactional(customer_contact, customer, role, member, authz)]
    pub async fn list_customer_contacts(
        &self,
        customer_id: CustomerId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<CustomerContact>, u64), CoreError> {
        let mut service = CustomerContactService::new(
            customer_contact_repository,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service
            .list_customer_contacts(customer_id, limit, offset)
            .await
    }

    #[transactional(customer_contact, customer, role, member, authz)]
    pub async fn update_customer_contact(
        &self,
        command: UpdateCustomerContactCommand,
    ) -> Result<CustomerContact, CoreError> {
        let mut service = CustomerContactService::new(
            customer_contact_repository,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.update_customer_contact(command).await
    }

    #[transactional(customer_contact, customer, role, member, authz)]
    pub async fn soft_delete_customer_contact(
        &self,
        id: CustomerContactId,
        actor: Subject,
    ) -> Result<(), CoreError> {
        let mut service = CustomerContactService::new(
            customer_contact_repository,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.soft_delete_customer_contact(id, actor).await
    }
}
