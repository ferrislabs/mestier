extern crate self as mestier_core;

pub mod application;
pub(crate) mod domain;
pub mod infrastructure;

pub use application::*;
pub use domain::{
    Customer, CustomerId, Employee, EmployeeId, Equipment, EquipmentId, FileObject, Member,
    MemberId, Organization, OrganizationId, Permissions, Property, PropertyId, Role, RoleId,
    ServiceRate, ServiceRateId, ServiceRateUnit, StoredFile, User, UserId,
    customer::commands::{CreateCustomerCommand, UpdateCustomerCommand},
    employee::commands::{CreateEmployeeCommand, LinkEmployeeUserCommand, UpdateEmployeeCommand},
    equipment::commands::{CreateEquipmentCommand, UpdateEquipmentCommand},
    file_storage::commands::UploadFileCommand,
    organization::commands::{CreateOrganizationCommand, UpdateOrganizationCommand},
    property::commands::{CreatePropertyCommand, UpdatePropertyCommand},
    service_rate::commands::{CreateServiceRateCommand, UpdateServiceRateCommand},
    user::commands::CreateUserCommand,
};
