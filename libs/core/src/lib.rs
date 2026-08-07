extern crate self as mestier_core;

pub mod application;
pub(crate) mod domain;
pub mod infrastructure;

pub use application::*;
pub use infrastructure::realtime::EventHub;

pub use domain::{
    AbsenceKind, AssigneeRef, Assignment, AssignmentId, Customer, CustomerContact,
    CustomerContactId, CustomerContext, CustomerContextId, CustomerId, CustomerPipelineStage,
    CustomerStatus, DateRange, Employee, EmployeeAbsence, EmployeeAbsenceId, EmployeeId,
    EmployeeRhythm, EmployeeRhythmId, EmployeeWorkSlot, EmployeeWorkSlotId, Equipment, EquipmentId,
    FileObject, Member, MemberId, MinuteInterval, Organization, OrganizationId, Permissions,
    Product, ProductId, Quote, QuoteId, QuoteLine, QuoteLineId, QuoteStatus, RhythmSlot,
    RhythmSlotId, Role, RoleId, ServiceRate, ServiceRateId, ServiceRateUnit, StoredFile, User,
    UserId, WorkOrder, WorkOrderId, WorkOrderStatus,
    absence::commands::{CreateAbsenceCommand, PatchAbsenceCommand},
    customer::commands::{CreateCustomerCommand, UpdateCustomerCommand},
    customer_contact::commands::{CreateCustomerContactCommand, UpdateCustomerContactCommand},
    customer_context::commands::{CreateCustomerContextCommand, UpdateCustomerContextCommand},
    employee::commands::{CreateEmployeeCommand, LinkEmployeeUserCommand, UpdateEmployeeCommand},
    equipment::commands::{CreateEquipmentCommand, UpdateEquipmentCommand},
    file_storage::commands::UploadFileCommand,
    organization::commands::{CreateOrganizationCommand, UpdateOrganizationCommand},
    product::commands::{CreateProductCommand, UpdateProductCommand},
    quote::commands::{
        CreateQuoteCommand, QuoteLineCommand, UpdateQuoteCommand, UpdateQuoteStatusCommand,
    },
    service_rate::commands::{CreateServiceRateCommand, UpdateServiceRateCommand},
    user::commands::{CreateUserCommand, UpsertUserBySubCommand},
    work_order::commands::{CreateWorkOrderCommand, PatchWorkOrderCommand},
    work_time::commands::{
        ReplaceRhythmCommand, ReplaceWorkSlotsCommand, RhythmSlotInput, WorkSlotInput,
    },
    work_time::service::expand_work_slots,
};
