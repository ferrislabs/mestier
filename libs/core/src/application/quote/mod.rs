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
    /// `organization` reads only the organization's VAT status — see
    /// `QuoteService::create_quote`. Listed here and on `update_quote` only:
    /// the other quote use cases below stay unaware of the organization
    /// aggregate.
    #[transactional(quote, organization, emitter)]
    pub async fn create_quote(&self, command: CreateQuoteCommand) -> Result<Quote, CoreError> {
        let mut service = QuoteService::new(quote_repository, emitter);
        service.create_quote(command, organization_repository).await
    }

    #[transactional(quote, emitter)]
    pub async fn get_quote(&self, id: QuoteId) -> Result<Quote, CoreError> {
        let mut service = QuoteService::new(quote_repository, emitter);
        service.get_quote(id).await
    }

    #[transactional(quote, emitter)]
    pub async fn list_quotes(
        &self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Quote>, u64), CoreError> {
        let mut service = QuoteService::new(quote_repository, emitter);
        service.list_quotes(organization_id, limit, offset).await
    }

    #[transactional(quote, organization, emitter)]
    pub async fn update_quote(&self, command: UpdateQuoteCommand) -> Result<Quote, CoreError> {
        let mut service = QuoteService::new(quote_repository, emitter);
        service.update_quote(command, organization_repository).await
    }

    #[transactional(quote, emitter)]
    pub async fn update_quote_status(
        &self,
        command: UpdateQuoteStatusCommand,
    ) -> Result<Quote, CoreError> {
        let mut service = QuoteService::new(quote_repository, emitter);
        service.update_quote_status(command).await
    }

    #[transactional(quote, emitter)]
    pub async fn soft_delete_quote(&self, id: QuoteId) -> Result<(), CoreError> {
        let mut service = QuoteService::new(quote_repository, emitter);
        service.soft_delete_quote(id).await
    }
}
