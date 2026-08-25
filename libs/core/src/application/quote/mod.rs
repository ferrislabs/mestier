use common::CoreError;
use mestier_macros::transactional;

use crate::{
    OrganizationId, Quote, QuoteId, TaskProposal,
    application::MestierUseCase,
    domain::quote::{
        commands::{CreateQuoteCommand, UpdateQuoteCommand, UpdateQuoteStatusCommand},
        service::{QuoteService, propose_tasks_from_quote, require_quote_accepted},
    },
};

impl MestierUseCase {
    /// `organization` reads the organization's VAT status and quote number
    /// prefix — see `QuoteService::create_quote`. Listed on every use case
    /// that can allocate a number or recompute totals; `get_quote`,
    /// `list_quotes` and `soft_delete_quote` stay unaware of the
    /// organization aggregate.
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

    #[transactional(quote, organization, emitter)]
    pub async fn update_quote_status(
        &self,
        command: UpdateQuoteStatusCommand,
    ) -> Result<Quote, CoreError> {
        let mut service = QuoteService::new(quote_repository, emitter);
        service
            .update_quote_status(command, organization_repository)
            .await
    }

    #[transactional(quote, emitter)]
    pub async fn soft_delete_quote(&self, id: QuoteId) -> Result<(), CoreError> {
        let mut service = QuoteService::new(quote_repository, emitter);
        service.soft_delete_quote(id).await
    }

    /// `GET .../plan-proposal`: one suggested task per quote line, and
    /// nothing more — a read the caller reviews before `POST .../plan`
    /// confirms anything (#298). Refuses a quote that isn't accepted with
    /// its own conflict rather than a 500.
    #[transactional(quote, emitter)]
    pub async fn get_quote_plan_proposal(
        &self,
        id: QuoteId,
    ) -> Result<(Quote, Vec<TaskProposal>), CoreError> {
        let mut service = QuoteService::new(quote_repository, emitter);
        let quote = service.get_quote(id).await?;
        require_quote_accepted(&quote)?;
        let proposal = propose_tasks_from_quote(&quote);

        Ok((quote, proposal))
    }
}
