extern crate self as mestier_core;

pub mod application;
pub(crate) mod domain;
pub mod infrastructure;

pub use application::*;
pub use infrastructure::realtime::EventHub;

pub use domain::{
    BillingSettings,
    Customer, CustomerContact, CustomerContactId, CustomerContext, CustomerContextId, CustomerId,
    CustomerPipelineStage, CustomerStatus, Employee, EmployeeId, Equipment, EquipmentId,
    FileObject, LegalMentionTemplate, LegalMentionTemplateId, Member, MemberId, Organization,
    OrganizationId, Permissions, Product, ProductId, Quote, QuoteId, QuoteLine, QuoteLineId,
    QuoteStatus, Role, RoleId, ServiceRate, ServiceRateId, ServiceRateUnit, StoredFile, User,
    UserId,
    billing_settings::commands::UpsertBillingSettingsCommand,
    customer::commands::{CreateCustomerCommand, UpdateCustomerCommand},
    customer_contact::commands::{CreateCustomerContactCommand, UpdateCustomerContactCommand},
    customer_context::commands::{CreateCustomerContextCommand, UpdateCustomerContextCommand},
    employee::commands::{CreateEmployeeCommand, LinkEmployeeUserCommand, UpdateEmployeeCommand},
    equipment::commands::{CreateEquipmentCommand, UpdateEquipmentCommand},
    file_storage::commands::UploadFileCommand,
    legal_mention_template::commands::{
        CreateLegalMentionTemplateCommand, UpdateLegalMentionTemplateCommand,
    },
    organization::commands::{CreateOrganizationCommand, UpdateOrganizationCommand},
    product::commands::{CreateProductCommand, UpdateProductCommand},
    quote::commands::{
        CreateQuoteCommand, QuoteLineCommand, UpdateQuoteCommand, UpdateQuoteStatusCommand,
    },
    service_rate::commands::{CreateServiceRateCommand, UpdateServiceRateCommand},
    user::commands::{CreateUserCommand, UpsertUserBySubCommand},
};
