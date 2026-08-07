extern crate self as mestier_core;

pub mod application;
pub(crate) mod domain;
pub mod infrastructure;

pub use application::*;
pub use infrastructure::realtime::EventHub;

pub use domain::{
    AbsenceKind, AssigneeRef, Assignment, AssignmentId, AvailabilityReport, Conflict, ConflictKind,
    Customer, CustomerContact, CustomerContactId, CustomerContext, CustomerContextId, CustomerId,
    CustomerPipelineStage, CustomerStatus, DateRange, DayLog, DayLogId, Employee, EmployeeAbsence,
    EmployeeAbsenceId, EmployeeId, EmployeeRhythm, EmployeeRhythmId, EmployeeWorkSlot,
    EmployeeWorkSlotId, EmployeeWorkTime, Equipment, EquipmentId, FileObject, Member, MemberId,
    MinuteInterval, Organization, OrganizationId, Permissions, PlanningEntry, PlanningResource,
    PlanningView, PlanningWorkOrder, Product, ProductId, Quote, QuoteId, QuoteLine, QuoteLineId,
    QuoteStatus, RhythmSlot, RhythmSlotId, Role, RoleId, ServiceRate, ServiceRateId,
    ServiceRateUnit, StoredFile, TimeEntry, TimeEntryId, TimeEntryPhotoPhase, TimeRange, Tz, User,
    UserId, WorkOrder, WorkOrderId, WorkOrderStatus,
    absence::commands::{CreateAbsenceCommand, PatchAbsenceCommand},
    customer::commands::{CreateCustomerCommand, UpdateCustomerCommand},
    customer_contact::commands::{CreateCustomerContactCommand, UpdateCustomerContactCommand},
    customer_context::commands::{CreateCustomerContextCommand, UpdateCustomerContextCommand},
    day_log::commands::CloseDayCommand,
    employee::commands::{CreateEmployeeCommand, LinkEmployeeUserCommand, UpdateEmployeeCommand},
    equipment::commands::{CreateEquipmentCommand, UpdateEquipmentCommand},
    file_storage::commands::UploadFileCommand,
    organization::commands::{CreateOrganizationCommand, UpdateOrganizationCommand},
    planning::service::detect_conflicts,
    product::commands::{CreateProductCommand, UpdateProductCommand},
    quote::commands::{
        CreateQuoteCommand, QuoteLineCommand, UpdateQuoteCommand, UpdateQuoteStatusCommand,
    },
    service_rate::commands::{CreateServiceRateCommand, UpdateServiceRateCommand},
    time_entry::commands::{
        AttachTimeEntryPhotosCommand, StartTimeEntryCommand, StopTimeEntryCommand,
    },
    user::commands::{CreateUserCommand, UpsertUserBySubCommand},
    work_order::commands::{CreateWorkOrderCommand, PatchWorkOrderCommand},
    work_time::commands::{
        ReplaceRhythmCommand, ReplaceWorkSlotsCommand, RhythmSlotInput, WorkSlotInput,
    },
    work_time::service::expand_work_slots,
};
