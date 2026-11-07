use chrono::Utc;
use common::CoreError;
use mestier_macros::transactional;

use authz::Resource;

use crate::{
    DealQuoteLinks, OrganizationId, Quote, QuoteId, QuoteOutcome, TaskProposal,
    application::{MestierUseCase, policy},
    domain::quote::{
        commands::{CreateQuoteCommand, UpdateQuoteCommand, UpdateQuoteStatusCommand},
        ports::QuoteRepository,
        search::{QuoteSearch, QuoteSearchResults},
        service::{
            QuoteChange, QuoteLookups, QuoteService, propose_tasks_from_quote,
            require_quote_accepted,
        },
    },
};

impl MestierUseCase {
    /// `organization` reads the organization's VAT status and quote number
    /// prefix — see `QuoteService::create_quote`. Listed on every use case
    /// that can allocate a number or recompute totals; `get_quote`,
    /// `list_quotes` and `soft_delete_quote` stay unaware of the
    /// organization aggregate.
    ///
    /// `actor` is the caller's AuthZen-shaped subject (#305), built by the
    /// handler from the request `Identity` via `handlers::resolve_actor`.
    /// It is a plain parameter here rather than a field on
    /// `CreateQuoteCommand`: unlike `organization::UpdateOrganizationCommand`,
    /// the quote commands live outside this workstream's file allowance, so
    /// the actor is threaded through the use case's own signature instead.
    /// `role`/`member`/`authz` are added to the repository list purely to
    /// enforce `quote.manage` — nothing else about this use case changes.
    #[transactional(
        quote,
        organization,
        customer,
        customer_contact,
        customer_context,
        role,
        member,
        authz,
        emitter
    )]
    pub async fn create_quote(
        &self,
        command: CreateQuoteCommand,
        actor: authz::Subject,
    ) -> Result<QuoteOutcome<Quote>, CoreError> {
        let mut organization_repository = organization_repository;
        let mut customer_repository = customer_repository;
        let mut customer_contact_repository = customer_contact_repository;
        let mut customer_context_repository = customer_context_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let actor = policy::enrich_for_organization(
            actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            authz,
            &actor,
            "quote.manage",
            Resource::new("organization", command.organization_id.0.to_string()),
        )
        .await?;

        let mut lookups = QuoteLookups {
            organizations: &mut organization_repository,
            customers: &mut customer_repository,
            contacts: &mut customer_contact_repository,
            addresses: &mut customer_context_repository,
            members: &mut member_repository,
        };
        let mut service = QuoteService::new(quote_repository, emitter);
        service
            .create_quote(command, &mut lookups, Utc::now())
            .await
    }

    #[transactional(quote, emitter)]
    pub async fn get_quote(&self, id: QuoteId) -> Result<Quote, CoreError> {
        let mut service = QuoteService::new(quote_repository, emitter);
        service.get_quote(id).await
    }

    #[transactional(quote)]
    pub async fn search_quotes(
        &self,
        search: QuoteSearch,
    ) -> Result<QuoteSearchResults, CoreError> {
        let mut quote_repository = quote_repository;
        let found = quote_repository.search(&search).await?;
        Ok(QuoteSearchResults::bounded(found, search.limit))
    }

    #[transactional(quote, emitter)]
    pub async fn list_quotes(
        &self,
        organization_id: OrganizationId,
        deal_id: Option<crate::DealId>,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Quote>, u64), CoreError> {
        let mut service = QuoteService::new(quote_repository, emitter);
        service
            .list_quotes(organization_id, deal_id, limit, offset)
            .await
    }

    #[transactional(
        quote,
        organization,
        customer,
        customer_contact,
        customer_context,
        deal,
        role,
        member,
        authz,
        emitter
    )]
    pub async fn update_quote(
        &self,
        command: UpdateQuoteCommand,
        actor: authz::Subject,
    ) -> Result<QuoteOutcome<Quote>, CoreError> {
        let mut quote_repository = quote_repository;
        let mut organization_repository = organization_repository;
        let mut customer_repository = customer_repository;
        let mut customer_contact_repository = customer_contact_repository;
        let mut customer_context_repository = customer_context_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let existing = quote_repository
            .find_by_id(command.id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let actor = policy::enrich_for_organization(
            actor,
            existing.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            authz,
            &actor,
            "quote.manage",
            Resource::new("organization", existing.organization_id.0.to_string()),
        )
        .await?;

        let now = Utc::now();
        let outcome = {
            let mut lookups = QuoteLookups {
                organizations: &mut organization_repository,
                customers: &mut customer_repository,
                contacts: &mut customer_contact_repository,
                addresses: &mut customer_context_repository,
                members: &mut member_repository,
            };
            let mut service = QuoteService::new(quote_repository, emitter);
            service.update_quote(command, &mut lookups, now).await?
        };

        follow_deal(outcome, deal_repository, customer_repository, now).await
    }

    #[transactional(
        quote,
        organization,
        customer,
        customer_contact,
        customer_context,
        deal,
        role,
        member,
        authz,
        emitter
    )]
    pub async fn update_quote_status(
        &self,
        command: UpdateQuoteStatusCommand,
        actor: authz::Subject,
    ) -> Result<QuoteOutcome<Quote>, CoreError> {
        let mut quote_repository = quote_repository;
        let mut organization_repository = organization_repository;
        let mut customer_repository = customer_repository;
        let mut customer_contact_repository = customer_contact_repository;
        let mut customer_context_repository = customer_context_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let existing = quote_repository
            .find_by_id(command.id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let actor = policy::enrich_for_organization(
            actor,
            existing.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            authz,
            &actor,
            "quote.manage",
            Resource::new("organization", existing.organization_id.0.to_string()),
        )
        .await?;

        let now = Utc::now();
        let outcome = {
            let mut lookups = QuoteLookups {
                organizations: &mut organization_repository,
                customers: &mut customer_repository,
                contacts: &mut customer_contact_repository,
                addresses: &mut customer_context_repository,
                members: &mut member_repository,
            };
            let mut service = QuoteService::new(quote_repository, emitter);
            service
                .update_quote_status(command, &mut lookups, now)
                .await?
        };

        follow_deal(outcome, deal_repository, customer_repository, now).await
    }

    #[transactional(quote, role, member, authz, emitter)]
    pub async fn revise_quote(
        &self,
        id: QuoteId,
        actor: authz::Subject,
    ) -> Result<QuoteOutcome<Quote>, CoreError> {
        let mut quote_repository = quote_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let existing = quote_repository
            .find_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let actor = policy::enrich_for_organization(
            actor,
            existing.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            authz,
            &actor,
            "quote.manage",
            Resource::new("organization", existing.organization_id.0.to_string()),
        )
        .await?;

        let mut service = QuoteService::new(quote_repository, emitter);
        service.revise_quote(id, Utc::now()).await
    }

    #[transactional(quote, role, member, authz, emitter)]
    pub async fn soft_delete_quote(
        &self,
        id: QuoteId,
        actor: authz::Subject,
    ) -> Result<QuoteOutcome<()>, CoreError> {
        let mut quote_repository = quote_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        let existing = quote_repository
            .find_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let actor = policy::enrich_for_organization(
            actor,
            existing.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        policy::require(
            authz,
            &actor,
            "quote.manage",
            Resource::new("organization", existing.organization_id.0.to_string()),
        )
        .await?;

        let mut service = QuoteService::new(quote_repository, emitter);
        service.soft_delete_quote(id, Utc::now()).await
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

async fn follow_deal(
    outcome: QuoteOutcome<QuoteChange>,
    deals: impl crate::domain::deal::ports::DealRepository,
    customers: impl crate::domain::customer::ports::CustomerRepository,
    now: chrono::DateTime<Utc>,
) -> Result<QuoteOutcome<Quote>, CoreError> {
    let change = match outcome {
        Ok(change) => change,
        Err(refusal) => return Ok(Err(refusal)),
    };
    DealQuoteLinks::new(deals, customers)
        .follow(&change, now)
        .await?;

    Ok(Ok(change.quote))
}
