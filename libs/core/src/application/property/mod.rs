use common::CoreError;
use mestier_macros::transactional;

use crate::{
    CustomerId, Property, PropertyId,
    application::MestierUseCase,
    domain::property::{
        commands::{CreatePropertyCommand, UpdatePropertyCommand},
        service::PropertyService,
    },
};

impl MestierUseCase {
    #[transactional(property)]
    pub async fn create_property(
        &self,
        command: CreatePropertyCommand,
    ) -> Result<Property, CoreError> {
        let mut service = PropertyService::new(property_repository);
        service.create_property(command).await
    }

    #[transactional(property)]
    pub async fn get_property(&self, id: PropertyId) -> Result<Property, CoreError> {
        let mut service = PropertyService::new(property_repository);
        service.get_property(id).await
    }

    #[transactional(property)]
    pub async fn list_properties(
        &self,
        customer_id: CustomerId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Property>, u64), CoreError> {
        let mut service = PropertyService::new(property_repository);
        service.list_properties(customer_id, limit, offset).await
    }

    #[transactional(property)]
    pub async fn update_property(
        &self,
        command: UpdatePropertyCommand,
    ) -> Result<Property, CoreError> {
        let mut service = PropertyService::new(property_repository);
        service.update_property(command).await
    }

    #[transactional(property)]
    pub async fn soft_delete_property(&self, id: PropertyId) -> Result<(), CoreError> {
        let mut service = PropertyService::new(property_repository);
        service.soft_delete_property(id).await
    }
}
