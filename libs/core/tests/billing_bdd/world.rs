use authz::{Decision, MockAuthorizer, Subject};
use mestier_core::{
    BillingSettings, BillingSettingsError, BillingSettingsService, MockInvoiceRepository,
    MockMemberRepository, MockOrganizationRepository, MockQuoteRepository, MockRoleRepository,
    NumberingError, OrganizationId, SeriesNumbering,
};

use crate::repository::{self, Store};

pub type Service = BillingSettingsService<
    MockOrganizationRepository,
    MockQuoteRepository,
    MockInvoiceRepository,
    MockRoleRepository,
    MockMemberRepository,
    MockAuthorizer,
>;

#[derive(Debug, Default, cucumber::World)]
pub struct BillingWorld {
    pub store: Store,
    pub organization_id: Option<OrganizationId>,
    pub read: Option<BillingSettings>,
    pub save_refusal: Option<BillingSettingsError>,
    pub numbering_refusal: Option<NumberingError>,
    pub numbering: Option<SeriesNumbering>,
    pub assigned_references: Vec<String>,
}

impl BillingWorld {
    pub fn service(&self) -> Service {
        let mut authz = MockAuthorizer::new();
        authz
            .expect_evaluate()
            .returning(|_| Box::pin(async { Ok(Decision::allow()) }));

        BillingSettingsService::new(
            repository::organizations(&self.store),
            repository::quotes(&self.store),
            repository::invoices(&self.store),
            repository::roles(),
            repository::members(),
            authz,
        )
    }

    pub fn organization_id(&self) -> OrganizationId {
        self.organization_id
            .expect("a scenario states the company in its `Given`")
    }

    pub fn actor(&self) -> Subject {
        Subject::system()
    }
}
