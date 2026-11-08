use std::collections::HashMap;

use authz::{Decision, MockAuthorizer, Subject};
use chrono::{DateTime, Utc};
use events::testing::RecordingEmitter;
use mestier_core::{
    CustomerContactService, CustomerContextService, CustomerError, CustomerId, CustomerService,
    MockCustomerContactRepository, MockCustomerContextRepository, MockCustomerRepository,
    MockDealRepository, MockMemberRepository, MockRoleRepository, OrganizationId,
    RegistrationNumber,
};

use crate::repository::{self, Store};

pub type Customers<'a> = CustomerService<
    MockCustomerRepository,
    MockDealRepository,
    MockMemberRepository,
    MockRoleRepository,
    MockAuthorizer,
    &'a RecordingEmitter,
>;

pub type Contacts = CustomerContactService<
    MockCustomerContactRepository,
    MockCustomerRepository,
    MockMemberRepository,
    MockRoleRepository,
    MockAuthorizer,
>;

pub type Contexts = CustomerContextService<
    MockCustomerContextRepository,
    MockCustomerRepository,
    MockMemberRepository,
    MockRoleRepository,
    MockAuthorizer,
>;

#[derive(Debug, Default, cucumber::World)]
pub struct CustomerWorld {
    pub store: Store,
    pub emitter: RecordingEmitter,
    pub organization_id: Option<OrganizationId>,
    pub members: HashMap<String, mestier_core::MemberId>,
    pub customers: HashMap<String, CustomerId>,
    pub current: Option<CustomerId>,
    pub refusal: Option<CustomerError>,
    pub registration: Option<Result<RegistrationNumber, CustomerError>>,
    pub payment_terms: Option<Result<Option<mestier_core::PaymentTermsDays>, CustomerError>>,
    pub first_conversion: Option<DateTime<Utc>>,
}

fn allow_all() -> MockAuthorizer {
    let mut authz = MockAuthorizer::new();
    authz
        .expect_evaluate()
        .returning(|_| Box::pin(async { Ok(Decision::allow()) }));
    authz
}

impl CustomerWorld {
    pub fn customers(&self) -> Customers<'_> {
        CustomerService::new(
            repository::customers(&self.store),
            repository::deals(),
            repository::members(&self.store),
            repository::roles(),
            allow_all(),
            &self.emitter,
        )
    }

    pub fn contacts(&self) -> Contacts {
        CustomerContactService::new(
            repository::contacts(&self.store),
            repository::customers(&self.store),
            repository::members(&self.store),
            repository::roles(),
            allow_all(),
        )
    }

    pub fn contexts(&self) -> Contexts {
        CustomerContextService::new(
            repository::contexts(&self.store),
            repository::customers(&self.store),
            repository::members(&self.store),
            repository::roles(),
            allow_all(),
        )
    }

    pub fn organization_id(&self) -> OrganizationId {
        self.organization_id
            .expect("a scenario states the company in its `Given`")
    }

    pub fn customer_id(&self) -> CustomerId {
        self.current
            .expect("the scenario creates or names a customer first")
    }

    pub fn actor(&self) -> Subject {
        Subject::system()
    }
}
