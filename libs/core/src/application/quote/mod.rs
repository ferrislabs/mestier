use common::CoreError;
use mestier_macros::transactional;

use crate::{
    OrganizationId, Quote, QuoteId,
    application::MestierUseCase,
    domain::quote::{
        commands::{CreateQuoteCommand, UpdateQuoteCommand, UpdateQuoteStatusCommand},
        service::QuoteService,
    },
};

impl MestierUseCase {
    #[transactional(quote)]
    pub async fn create_quote(&self, command: CreateQuoteCommand) -> Result<Quote, CoreError> {
        let mut service = QuoteService::new(quote_repository);
        service.create_quote(command).await
    }

    #[transactional(quote)]
    pub async fn get_quote(&self, id: QuoteId) -> Result<Quote, CoreError> {
        let mut service = QuoteService::new(quote_repository);
        service.get_quote(id).await
    }

    #[transactional(quote)]
    pub async fn list_quotes(
        &self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Quote>, u64), CoreError> {
        let mut service = QuoteService::new(quote_repository);
        service.list_quotes(organization_id, limit, offset).await
    }

    #[transactional(quote)]
    pub async fn update_quote(&self, command: UpdateQuoteCommand) -> Result<Quote, CoreError> {
        let mut service = QuoteService::new(quote_repository);
        service.update_quote(command).await
    }

    #[transactional(quote)]
    pub async fn update_quote_status(
        &self,
        command: UpdateQuoteStatusCommand,
    ) -> Result<Quote, CoreError> {
        let mut service = QuoteService::new(quote_repository);
        service.update_quote_status(command).await
    }

    #[transactional(quote)]
    pub async fn soft_delete_quote(&self, id: QuoteId) -> Result<(), CoreError> {
        let mut service = QuoteService::new(quote_repository);
        service.soft_delete_quote(id).await
    }
}
