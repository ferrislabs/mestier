use authz::{Authorizer, Resource, Subject};
use chrono::{DateTime, Utc};
use common::{CoreError, generate_uuid_v7};
use events::EventEmitter;

use super::{
    Deal, DealAmount, DealError, DealId, DealOutcome, DealStage, DealTitle, PipelineSummary,
    commands::{CreateDealCommand, DealFilter, MoveDealCommand, UpdateDealCommand},
    events::{DealCreated, DealStageChanged},
    normalized_notes,
    ports::DealRepository,
    summary::{summarise, window_start},
};
use crate::{
    CustomerId, CustomerStatus, MemberId, OrganizationId,
    application::policy,
    domain::{
        customer::ports::CustomerRepository, member::ports::MemberRepository,
        role::ports::RoleRepository,
    },
};

pub struct DealService<D, C, M, Ro, A, E>
where
    D: DealRepository,
    C: CustomerRepository,
    M: MemberRepository,
    Ro: RoleRepository,
    A: Authorizer,
    E: EventEmitter,
{
    deals: D,
    customers: C,
    member_repository: M,
    role_repository: Ro,
    authz: A,
    emitter: E,
}

impl<D, C, M, Ro, A, E> DealService<D, C, M, Ro, A, E>
where
    D: DealRepository,
    C: CustomerRepository,
    M: MemberRepository,
    Ro: RoleRepository,
    A: Authorizer,
    E: EventEmitter,
{
    pub fn new(
        deals: D,
        customers: C,
        member_repository: M,
        role_repository: Ro,
        authz: A,
        emitter: E,
    ) -> Self {
        Self {
            deals,
            customers,
            member_repository,
            role_repository,
            authz,
            emitter,
        }
    }

    async fn authorize_write(
        &mut self,
        actor: Subject,
        organization_id: OrganizationId,
    ) -> Result<(), CoreError> {
        let actor = policy::enrich_for_organization(
            actor,
            organization_id,
            &mut self.member_repository,
            &mut self.role_repository,
        )
        .await?;
        policy::require(
            &self.authz,
            &actor,
            "customer.manage",
            Resource::new("organization", organization_id.0.to_string()),
        )
        .await
    }

    async fn require_customer(
        &mut self,
        organization_id: OrganizationId,
        customer_id: CustomerId,
    ) -> Result<(), CoreError> {
        self.customers
            .find_by_id(customer_id)
            .await?
            .filter(|customer| customer.organization_id == organization_id)
            .map(|_| ())
            .ok_or(CoreError::NotFound)
    }

    async fn owner_refusal(
        &mut self,
        organization_id: OrganizationId,
        owner: Option<MemberId>,
        current_owner: Option<MemberId>,
    ) -> Result<Option<DealError>, CoreError> {
        let Some(owner) = owner.filter(|owner| Some(*owner) != current_owner) else {
            return Ok(None);
        };
        let same_organization = self
            .member_repository
            .find_by_id(owner)
            .await?
            .is_some_and(|member| member.organization_id == organization_id);
        Ok((!same_organization).then_some(DealError::OwnerNotMember))
    }

    pub async fn create_deal(
        &mut self,
        command: CreateDealCommand,
    ) -> Result<DealOutcome<Deal>, CoreError> {
        let title = match DealTitle::try_from(command.title) {
            Ok(title) => title,
            Err(refusal) => return Ok(Err(refusal)),
        };
        let amount = match DealAmount::try_from(command.amount_cents) {
            Ok(amount) => amount,
            Err(refusal) => return Ok(Err(refusal)),
        };

        self.authorize_write(command.actor, command.organization_id)
            .await?;
        self.require_customer(command.organization_id, command.customer_id)
            .await?;
        if let Some(refusal) = self
            .owner_refusal(command.organization_id, command.owner_member_id, None)
            .await?
        {
            return Ok(Err(refusal));
        }

        let now = Utc::now();
        let deal = Deal {
            id: DealId(generate_uuid_v7()),
            organization_id: command.organization_id,
            customer_id: command.customer_id,
            title,
            amount,
            stage: DealStage::New,
            expected_close_date: command.expected_close_date,
            owner_member_id: command.owner_member_id,
            source: command.source,
            notes: normalized_notes(command.notes),
            created_at: now,
            stage_changed_at: now,
            closed_at: None,
            quote_count: 0,
        };

        let created = self.deals.insert(&deal).await?;

        self.emitter.emit(
            created.organization_id,
            &DealCreated {
                deal: created.clone(),
            },
        )?;

        Ok(Ok(created))
    }

    pub async fn get_deal(&mut self, id: DealId) -> Result<Deal, CoreError> {
        self.deals.find_by_id(id).await?.ok_or(CoreError::NotFound)
    }

    pub async fn list_deals(
        &mut self,
        filter: DealFilter,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Deal>, u64), CoreError> {
        self.deals.list(&filter, limit, offset).await
    }

    pub async fn update_deal(
        &mut self,
        command: UpdateDealCommand,
    ) -> Result<DealOutcome<Deal>, CoreError> {
        let title = match DealTitle::try_from(command.title) {
            Ok(title) => title,
            Err(refusal) => return Ok(Err(refusal)),
        };
        let amount = match DealAmount::try_from(command.amount_cents) {
            Ok(amount) => amount,
            Err(refusal) => return Ok(Err(refusal)),
        };

        let mut deal = self.get_deal(command.id).await?;
        self.authorize_write(command.actor, deal.organization_id)
            .await?;
        if command.customer_id != deal.customer_id {
            self.require_customer(deal.organization_id, command.customer_id)
                .await?;
        }
        if let Some(refusal) = self
            .owner_refusal(
                deal.organization_id,
                command.owner_member_id,
                deal.owner_member_id,
            )
            .await?
        {
            return Ok(Err(refusal));
        }

        deal.customer_id = command.customer_id;
        deal.title = title;
        deal.amount = amount;
        deal.expected_close_date = command.expected_close_date;
        deal.owner_member_id = command.owner_member_id;
        deal.source = command.source;
        deal.notes = normalized_notes(command.notes);

        self.deals.update(&deal).await.map(Ok)
    }

    pub async fn move_deal(
        &mut self,
        command: MoveDealCommand,
    ) -> Result<DealOutcome<Deal>, CoreError> {
        let mut deal = self.get_deal(command.id).await?;
        self.authorize_write(command.actor, deal.organization_id)
            .await?;

        let stage = match DealStage::try_from((command.stage, command.lost_reason.as_deref())) {
            Ok(stage) => stage,
            Err(refusal) => return Ok(Err(refusal)),
        };

        let now = Utc::now();
        let won = matches!(stage, DealStage::Won);
        let from = deal.stage.name();
        if !deal.move_to(stage, now) {
            return Ok(Ok(deal));
        }
        let saved = self.deals.update(&deal).await?;

        if won {
            convert_prospect(&mut self.customers, saved.customer_id, now).await?;
        }

        self.emitter.emit(
            saved.organization_id,
            &DealStageChanged {
                deal: saved.clone(),
                from,
            },
        )?;

        Ok(Ok(saved))
    }

    pub async fn delete_deal(
        &mut self,
        id: DealId,
        actor: Subject,
    ) -> Result<DealOutcome<()>, CoreError> {
        let deal = self.get_deal(id).await?;
        self.authorize_write(actor, deal.organization_id).await?;

        if deal.is_linked_to_a_quote() {
            return Ok(Err(DealError::HasQuote));
        }

        self.deals.delete(id).await.map(Ok)
    }

    pub async fn summary(
        &mut self,
        organization_id: OrganizationId,
        now: DateTime<Utc>,
    ) -> Result<PipelineSummary, CoreError> {
        let since = window_start(now);
        let by_stage = self.deals.stage_totals(organization_id).await?;
        let closed = self
            .deals
            .closed_totals_since(organization_id, since)
            .await?;
        let conversions = self
            .customers
            .count_converted_since(organization_id, since)
            .await?;

        Ok(summarise(&by_stage, &closed, conversions))
    }
}

pub(crate) async fn convert_prospect(
    customers: &mut impl CustomerRepository,
    customer_id: CustomerId,
    now: DateTime<Utc>,
) -> Result<(), CoreError> {
    let Some(mut customer) = customers.find_by_id(customer_id).await? else {
        return Ok(());
    };
    if customer.status != CustomerStatus::Prospect {
        return Ok(());
    }
    customer.change_status(CustomerStatus::Client, now);
    customer.updated_at = now;
    customers.update(&customer).await?;
    Ok(())
}
