extern crate self as mestier_core;

pub mod application;
pub(crate) mod domain;
pub mod infrastructure;

pub use application::*;
pub use domain::{
    Customer, CustomerContext, CustomerContextId, CustomerId, Employee, EmployeeId, Equipment,
    EquipmentId, FileObject, Member, MemberId, Organization, OrganizationId, Permissions, Quote,
    QuoteId, QuoteLine, QuoteLineId, QuoteStatus, Role, RoleId, ServiceRate, ServiceRateId,
    ServiceRateUnit, StoredFile, User, UserId,
    customer::commands::{CreateCustomerCommand, UpdateCustomerCommand},
    customer_context::commands::{CreateCustomerContextCommand, UpdateCustomerContextCommand},
    employee::commands::{CreateEmployeeCommand, LinkEmployeeUserCommand, UpdateEmployeeCommand},
    equipment::commands::{CreateEquipmentCommand, UpdateEquipmentCommand},
    file_storage::commands::UploadFileCommand,
    organization::commands::{CreateOrganizationCommand, UpdateOrganizationCommand},
    quote::commands::{
        CreateQuoteCommand, QuoteLineCommand, UpdateQuoteCommand, UpdateQuoteStatusCommand,
    },
    service_rate::commands::{CreateServiceRateCommand, UpdateServiceRateCommand},
    user::commands::{CreateUserCommand, UpsertUserBySubCommand},
};
