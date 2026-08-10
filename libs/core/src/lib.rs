extern crate self as mestier_core;

pub mod application;
pub(crate) mod domain;
pub mod infrastructure;

pub use application::*;
pub use domain::automation::connector::{
    AuthRequirement, AuthScheme, ConnectorCatalogue, ConnectorCatalogueError, ConnectorDescriptor,
    Field, FieldKind, SelectOption, VisibleWhen, auth_scheme, auth_schemes, connector_catalogue,
};
pub use domain::automation::credential::{
    CreateCredentialCommand, Credential, CredentialError, CredentialOrigin,
    UpdateCredentialCommand, validate_credential_data,
};
pub use domain::automation::event_catalogue;
pub use domain::automation::expression::{
    ExpressionContext, ExpressionError, LoopFrame, Template, parse_template,
};
pub use domain::automation::ports::{
    DeliveryHandler, DeliveryOutcome, DispatchOutcome, DueDelivery,
};
pub use domain::automation::run::{
    Connector, ConnectorInput, ConnectorOutcome, DueRun, Run, RunSettlement, RunStatus, RunStep,
    StepStatus,
};
pub use domain::automation::secret::SealedSecret;
pub use domain::automation::settings::{AutomationSettings, SettingsBounds};
pub use domain::automation::workflow::{
    Branch, CreateWorkflowCommand, Edge, Graph, GraphError, PlacedConnector,
    SaveWorkflowVersionCommand, UpdateWorkflowCommand, Workflow, WorkflowReference,
    WorkflowVersion, validate_graph,
};
pub use infrastructure::automation::connectors::{ConnectorRegistry, UnknownConnectorKind};
pub use infrastructure::automation::webhook::{
    WebhookDeliveryHandler, address_policy::PrivateNetworkAccess, secret::SecretCipher,
};
pub use infrastructure::automation::worker::{WorkerSchedule, run_automation_worker};
pub use infrastructure::realtime::EventHub;

pub use domain::{
    AbsenceKind, AssigneeRef, AvailabilityReport, Conflict, ConflictKind, Customer,
    CustomerContact, CustomerContactId, CustomerContext, CustomerContextId, CustomerId,
    CustomerPipelineStage, CustomerStatus, DateRange, Employee, EmployeeAbsence, EmployeeAbsenceId,
    EmployeeId, EmployeeRhythm, EmployeeRhythmId, EmployeeWorkSlot, EmployeeWorkSlotId,
    EmployeeWorkTime, Equipment, EquipmentId, FileObject, Member, MemberId, MemberWithAccount,
    MinuteInterval, Organization, OrganizationId, Permissions, PlanningEntry, PlanningResource,
    PlanningTask, PlanningView, Product, ProductId, Quote, QuoteId, QuoteLine, QuoteLineId,
    QuoteStatus, RhythmSlot, RhythmSlotId, Role, RoleId, ServiceRate, ServiceRateId,
    ServiceRateUnit, StoredFile, Task, TaskAssignment, TaskAssignmentId, TaskComment,
    TaskCommentId, TaskId, TaskLabel, TaskLabelId, TaskStatus, TimeRange, Tz, User, UserId,
    absence::commands::{CreateAbsenceCommand, PatchAbsenceCommand},
    customer::commands::{CreateCustomerCommand, UpdateCustomerCommand},
    customer_contact::commands::{CreateCustomerContactCommand, UpdateCustomerContactCommand},
    customer_context::commands::{CreateCustomerContextCommand, UpdateCustomerContextCommand},
    employee::commands::{CreateEmployeeCommand, LinkEmployeeUserCommand, UpdateEmployeeCommand},
    equipment::commands::{CreateEquipmentCommand, UpdateEquipmentCommand},
    file_storage::commands::UploadFileCommand,
    member::commands::{
        AddMemberCommand, AssignRoleCommand, CreateMemberCommand, UpdateMemberCommand,
    },
    organization::commands::{CreateOrganizationCommand, UpdateOrganizationCommand},
    planning::service::detect_conflicts,
    product::commands::{CreateProductCommand, UpdateProductCommand},
    quote::commands::{
        CreateQuoteCommand, QuoteLineCommand, UpdateQuoteCommand, UpdateQuoteStatusCommand,
    },
    service_rate::commands::{CreateServiceRateCommand, UpdateServiceRateCommand},
    task::commands::{CreateTaskCommand, PatchTaskCommand},
    task::service::{resolve_task_window, validate_parent_depth, validate_reparenting},
    task_comment::commands::{CreateTaskCommentCommand, UpdateTaskCommentCommand},
    task_label::commands::{CreateTaskLabelCommand, UpdateTaskLabelCommand},
    user::commands::{CreateUserCommand, UpsertUserBySubCommand},
    work_time::commands::{
        ReplaceRhythmCommand, ReplaceWorkSlotsCommand, RhythmSlotInput, WorkSlotInput,
    },
    work_time::service::expand_work_slots,
};
