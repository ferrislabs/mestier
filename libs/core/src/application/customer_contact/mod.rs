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
    #[transactional(customer_contact)]
    pub async fn create_customer_contact(
        &self,
        command: CreateCustomerContactCommand,
    ) -> Result<CustomerContact, CoreError> {
        let mut service = CustomerContactService::new(customer_contact_repository);
        service.create_customer_contact(command).await
    }

    #[transactional(customer_contact)]
    pub async fn get_customer_contact(
        &self,
        id: CustomerContactId,
    ) -> Result<CustomerContact, CoreError> {
        let mut service = CustomerContactService::new(customer_contact_repository);
        service.get_customer_contact(id).await
    }

    #[transactional(customer_contact)]
    pub async fn list_customer_contacts(
        &self,
        customer_id: CustomerId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<CustomerContact>, u64), CoreError> {
        let mut service = CustomerContactService::new(customer_contact_repository);
        service
            .list_customer_contacts(customer_id, limit, offset)
            .await
    }

    #[transactional(customer_contact)]
    pub async fn update_customer_contact(
        &self,
        command: UpdateCustomerContactCommand,
    ) -> Result<CustomerContact, CoreError> {
        let mut service = CustomerContactService::new(customer_contact_repository);
        service.update_customer_contact(command).await
    }

    #[transactional(customer_contact)]
    pub async fn soft_delete_customer_contact(
        &self,
        id: CustomerContactId,
    ) -> Result<(), CoreError> {
        let mut service = CustomerContactService::new(customer_contact_repository);
        service.soft_delete_customer_contact(id).await
    }
}
