use std::collections::HashMap;

use authz::{Decision, MockAuthorizer, Subject};
use common::CoreError;
use mestier_core::{
    CustomerId, CustomerService, DealError, DealId, DealService, MockCustomerRepository,
    MockDealRepository, MockMemberRepository, MockRoleRepository, OrganizationId, PipelineSummary,
};

use crate::repository::{self, Store};

pub type Deals = DealService<
    MockDealRepository,
    MockCustomerRepository,
    MockMemberRepository,
    MockRoleRepository,
    MockAuthorizer,
>;

pub type Customers = CustomerService<
    MockCustomerRepository,
    MockDealRepository,
    MockMemberRepository,
    MockRoleRepository,
    MockAuthorizer,
>;

#[derive(Debug)]
pub enum Refusal {
    Rule(DealError),
    Core(CoreError),
}

impl Refusal {
    pub fn describe(&self) -> String {
        match self {
            Self::Rule(error) => error.to_string(),
            Self::Core(CoreError::NotFound) => "not found".to_owned(),
            Self::Core(other) => other.to_string(),
        }
    }
}

#[derive(Debug, Default, cucumber::World)]
pub struct DealWorld {
    pub store: Store,
    pub organization_id: Option<OrganizationId>,
    pub customers: HashMap<String, CustomerId>,
    pub deals: HashMap<String, DealId>,
    pub refusal: Option<Refusal>,
    pub summary: Option<PipelineSummary>,
}

fn allow_all() -> MockAuthorizer {
    let mut authz = MockAuthorizer::new();
    authz
        .expect_evaluate()
        .returning(|_| Box::pin(async { Ok(Decision::allow()) }));
    authz
}

impl DealWorld {
    pub fn deals(&self) -> Deals {
        DealService::new(
            repository::deals(&self.store),
            repository::customers(&self.store),
            repository::members(&self.store),
            repository::roles(),
            allow_all(),
        )
    }

    pub fn customer_service(&self) -> Customers {
        CustomerService::new(
            repository::customers(&self.store),
            repository::deals(&self.store),
            repository::members(&self.store),
            repository::roles(),
            allow_all(),
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
