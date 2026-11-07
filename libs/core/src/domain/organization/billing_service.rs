use authz::{Authorizer, Resource, Subject};
use common::CoreError;

use crate::{
    application::policy,
    domain::{
        invoice::{InvoiceNumberSeries, ports::InvoiceRepository},
        member::ports::MemberRepository,
        organization::{
            Organization, OrganizationId,
            billing_settings::{BillingSettings, NumberPadding},
            commands::{ReplaceBillingSettingsCommand, SetNextNumberCommand},
            numbering::{
                CREDIT_NOTE_NUMBER_PREFIX, NumberSeries, NumberingError, SeriesNumbering,
                forward_target,
            },
            ports::OrganizationRepository,
        },
        quote::ports::QuoteRepository,
        role::ports::RoleRepository,
    },
};

pub type NumberingOutcome = Result<SeriesNumbering, NumberingError>;

pub struct BillingSettingsService<O, Q, I, R, M, A>
where
    O: OrganizationRepository,
    Q: QuoteRepository,
    I: InvoiceRepository,
    R: RoleRepository,
    M: MemberRepository,
    A: Authorizer,
{
    organization_repository: O,
    quote_repository: Q,
    invoice_repository: I,
    role_repository: R,
    member_repository: M,
    authz: A,
}

impl<O, Q, I, R, M, A> BillingSettingsService<O, Q, I, R, M, A>
where
    O: OrganizationRepository,
    Q: QuoteRepository,
    I: InvoiceRepository,
    R: RoleRepository,
    M: MemberRepository,
    A: Authorizer,
{
    pub fn new(
        organization_repository: O,
        quote_repository: Q,
        invoice_repository: I,
        role_repository: R,
        member_repository: M,
        authz: A,
    ) -> Self {
        Self {
            organization_repository,
            quote_repository,
            invoice_repository,
            role_repository,
            member_repository,
            authz,
        }
    }

    pub async fn get_billing_settings(
        &mut self,
        id: OrganizationId,
    ) -> Result<BillingSettings, CoreError> {
        self.find_organization(id).await?;

        Ok(self
            .organization_repository
            .find_billing_settings(id)
            .await?
            .unwrap_or_default())
    }

    pub async fn replace_billing_settings(
        &mut self,
        command: ReplaceBillingSettingsCommand,
    ) -> Result<BillingSettings, CoreError> {
        let organization = self.find_organization(command.id).await?;
        self.require_update(command.actor, &organization).await?;

        self.organization_repository
            .upsert_billing_settings(organization.id, &command.settings)
            .await
    }

    pub async fn get_numbering(
        &mut self,
        id: OrganizationId,
        year: i32,
    ) -> Result<Vec<SeriesNumbering>, CoreError> {
        let organization = self.find_organization(id).await?;
        let padding = self.padding(id).await?;
        let mut series_numbering = Vec::with_capacity(NumberSeries::ALL.len());
        for series in NumberSeries::ALL {
            series_numbering.push(
                self.series_numbering(&organization, series, year, padding)
                    .await?,
            );
        }

        Ok(series_numbering)
    }

    pub async fn set_next_number(
        &mut self,
        command: SetNextNumberCommand,
    ) -> Result<NumberingOutcome, CoreError> {
        let organization = self.find_organization(command.id).await?;
        self.require_update(command.actor, &organization).await?;

        let padding = self.padding(organization.id).await?;
        let current = self
            .series_numbering(&organization, command.series, command.year, padding)
            .await?;
        let target = match forward_target(command.next_number, current.last_assigned) {
            Ok(target) => target,
            Err(refusal) => return Ok(Err(refusal)),
        };

        if target > current.last_assigned {
            self.raise(organization.id, command.series, command.year, target)
                .await?;
        }

        Ok(Ok(SeriesNumbering {
            last_assigned: target,
            ..current
        }))
    }

    async fn find_organization(&mut self, id: OrganizationId) -> Result<Organization, CoreError> {
        self.organization_repository
            .find_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    async fn require_update(
        &mut self,
        actor: Subject,
        organization: &Organization,
    ) -> Result<(), CoreError> {
        let actor = policy::enrich_for_organization(
            actor,
            organization.id,
            &mut self.member_repository,
            &mut self.role_repository,
        )
        .await?;

        policy::require(
            &self.authz,
            &actor,
            "organization.update",
            Resource::new("organization", organization.id.0.to_string())
                .with_property("mestier.slug", organization.slug.clone()),
        )
        .await
    }

    async fn padding(&mut self, id: OrganizationId) -> Result<NumberPadding, CoreError> {
        Ok(self
            .organization_repository
            .find_billing_settings(id)
            .await?
            .unwrap_or_default()
            .number_padding)
    }

    async fn series_numbering(
        &mut self,
        organization: &Organization,
        series: NumberSeries,
        year: i32,
        padding: NumberPadding,
    ) -> Result<SeriesNumbering, CoreError> {
        let (prefix, last_assigned) = match series {
            NumberSeries::Quote => (
                organization.quote_number_prefix.clone(),
                self.quote_repository
                    .last_assigned_number(organization.id, year)
                    .await?,
            ),
            NumberSeries::Invoice => (
                organization.invoice_number_prefix.clone(),
                self.invoice_repository
                    .last_assigned_number(organization.id, InvoiceNumberSeries::Invoice, year)
                    .await?,
            ),
            NumberSeries::CreditNote => (
                CREDIT_NOTE_NUMBER_PREFIX.to_owned(),
                self.invoice_repository
                    .last_assigned_number(organization.id, InvoiceNumberSeries::CreditNote, year)
                    .await?,
            ),
        };

        Ok(SeriesNumbering {
            series,
            prefix,
            year,
            last_assigned,
            padding,
        })
    }

    async fn raise(
        &mut self,
        id: OrganizationId,
        series: NumberSeries,
        year: i32,
        last_assigned: i32,
    ) -> Result<(), CoreError> {
        match series {
            NumberSeries::Quote => {
                self.quote_repository
                    .raise_last_assigned_number(id, year, last_assigned)
                    .await
            }
            NumberSeries::Invoice => {
                self.invoice_repository
                    .raise_last_assigned_number(
                        id,
                        InvoiceNumberSeries::Invoice,
                        year,
                        last_assigned,
                    )
                    .await
            }
            NumberSeries::CreditNote => {
                self.invoice_repository
                    .raise_last_assigned_number(
                        id,
                        InvoiceNumberSeries::CreditNote,
                        year,
                        last_assigned,
                    )
                    .await
            }
        }
    }
}
