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
pub use domain::automation::ports::DispatchOutcome;
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
    address_policy::PrivateNetworkAccess, secret::SecretCipher,
};
pub use infrastructure::automation::worker::{WorkerSchedule, run_automation_worker};
pub use infrastructure::realtime::EventHub;

/// The rule that turns a salaried person's monthly cost into an hourly one.
///
/// Exposed because three layers need the same arithmetic and none of them may
/// re-derive it: the profitability calculation costs their time with it, the
/// employee response sends the equivalent back so a form can show it, and a
/// migration comment points at it.
pub use domain::employee::service::salaried_hourly_rate_cents;

/// Exposed for the BDD suite under `tests/`, which drives the quote service
/// against a `mockall` double of its repository port. The port itself stays
/// crate-private: nothing outside implements it by hand.
pub use domain::quote::service::QuoteService;

/// The profitability calculation, and the port it reads through. Exposed so the
/// use case in `application` can drive it, and so the report types are usable
/// from the handler crates.
pub use domain::profitability::{
    ports::ProfitabilityRepository, service::ProfitabilityService, service::build_report,
};
/// Every port's `mockall` double, for test targets outside this crate: its
/// own `tests/`, and the handler crates. Gated so mockall never reaches a
/// normal build.
#[cfg(feature = "mock")]
pub use domain::{
    absence::ports::MockAbsenceRepository,
    assignment_report::ports::MockAssignmentReportRepository,
    automation::ports::{
        MockAutomationSettingsRepository, MockCredentialRepository, MockEventDispatchRepository,
        MockEventLogRepository, MockRunRepository, MockWorkflowRepository,
    },
    customer::ports::MockCustomerRepository,
    customer_contact::ports::MockCustomerContactRepository,
    customer_context::ports::MockCustomerContextRepository,
    employee::ports::{MockEmployeeCostBasisRepository, MockEmployeeRepository},
    equipment::ports::MockEquipmentRepository,
    invitation::ports::MockInvitationRepository,
    member::ports::MockMemberRepository,
    organization::ports::MockOrganizationRepository,
    planning::ports::MockPlanningRepository,
    product::ports::MockProductRepository,
    project::ports::MockProjectRepository,
    project_template::ports::MockProjectTemplateRepository,
    quote::ports::MockQuoteRepository,
    role::ports::MockRoleRepository,
    service_rate::ports::MockServiceRateRepository,
    task::ports::MockTaskRepository,
    task_comment::ports::MockTaskCommentRepository,
    task_label::ports::MockTaskLabelRepository,
    task_recurrence::ports::MockTaskRecurrenceRepository,
    user::ports::MockUserRepository,
    work_time::ports::{MockRhythmRepository, MockWorkSlotRepository},
};

pub use domain::{
    Absence, AbsenceId, AbsenceKind, AssigneeRef, AssignmentReport, AssignmentReportId,
    AssignmentReportResolution, AvailabilityReport, Conflict, ConflictKind, Customer,
    CustomerContact, CustomerContactId, CustomerContext, CustomerContextId, CustomerId,
    CustomerPipelineStage, CustomerStatus, DateRange, DayLog, DayLogId, DeleteScope, Employee,
    EmployeeCostBasis, EmployeeCostBasisId, EmployeeId, EmployeeRhythm, EmployeeRhythmId,
    Equipment, EquipmentId, FileObject, Invitation, InvitationId, LegalIdentity, Member, MemberId,
    MemberProfitability, MemberWithAccount, MemberWorkTime, MinuteInterval, MissingCost,
    MissingLegalIdentityField, Organization, OrganizationAddress, OrganizationId, Permissions,
    PlannedAssignment, PlanningEntry, PlanningResource, PlanningTask, PlanningView, PresignedUrl,
    Product, ProductId, ProfitabilityFacts, ProfitabilityReport, Project, ProjectId,
    ProjectProfitability, ProjectTemplate, ProjectTemplateId, ProjectTemplateTask,
    ProjectTemplateTaskId, Quote, QuoteId, QuoteLine, QuoteLineId, QuoteStatus,
    QuoteVatBreakdownLine, RecurrenceFrequency, RecurrenceRule, ReportPeriod, RhythmSlot,
    RhythmSlotId, Role, RoleId, ServiceRate, ServiceRateId, ServiceRateUnit, StoredFile, Task,
    TaskAssignment, TaskAssignmentId, TaskComment, TaskCommentId, TaskExpense, TaskId, TaskLabel,
    TaskLabelId, TaskRecurrence, TaskRecurrenceId, TaskStatus, TimeEntry, TimeEntryId,
    TimeEntryPhoto, TimeEntryPhotoId, TimeEntryPhotoPhase, TimeRange, Tz, User, UserId, VatStatus,
    WorkSlot, WorkSlotId,
    absence::commands::{CreateAbsenceCommand, PatchAbsenceCommand},
    assignment_report::commands::{
        AmendAssignmentReportCommand, ReportAssignmentCommand, ResolveAssignmentReportCommand,
        WithdrawAssignmentReportCommand,
    },
    customer::commands::{CreateCustomerCommand, UpdateCustomerCommand},
    customer_contact::commands::{CreateCustomerContactCommand, UpdateCustomerContactCommand},
    customer_context::commands::{CreateCustomerContextCommand, UpdateCustomerContextCommand},
    employee::commands::{
        CorrectEmployeeCostBasisCommand, RemoveEmployeeProfileCommand, SetEmployeeCostBasisCommand,
        UpsertEmployeeProfileCommand,
    },
    equipment::commands::{CreateEquipmentCommand, UpdateEquipmentCommand},
    file_storage::commands::UploadFileCommand,
    invitation::commands::{AcceptInvitationCommand, InviteMemberCommand, RevokeInvitationCommand},
    member::commands::{
        AddMemberCommand, AssignRoleCommand, CreateMemberCommand, UpdateMemberCommand,
    },
    organization::commands::{
        CreateOrganizationCommand, UpdateLegalIdentityCommand, UpdateOrganizationCommand,
    },
    planning::service::detect_conflicts,
    product::commands::{CreateProductCommand, UpdateProductCommand},
    project::commands::{CreateProjectCommand, UpdateProjectCommand},
    project_template::commands::{
        CreateProjectTemplateCommand, InstantiateProjectTemplateCommand,
        ProjectTemplateTaskShapeCommand, ReplaceProjectTemplateTasksCommand,
        UpdateProjectTemplateCommand,
    },
    quote::commands::{
        CreateQuoteCommand, QuoteLineCommand, UpdateQuoteCommand, UpdateQuoteStatusCommand,
    },
    service_rate::commands::{CreateServiceRateCommand, UpdateServiceRateCommand},
    task::commands::{CreateTaskCommand, PatchTaskCommand},
    task::service::{resolve_task_window, validate_parent_depth, validate_reparenting},
    task_comment::commands::{CreateTaskCommentCommand, UpdateTaskCommentCommand},
    task_label::commands::{CreateTaskLabelCommand, UpdateTaskLabelCommand},
    task_recurrence::commands::{CreateTaskRecurrenceCommand, PatchTaskRecurrenceCommand},
    task_recurrence::service::{
        DEFAULT_HORIZON_DAYS, RecurrenceOccurrence, TaskRecurrenceService, dates_in_range,
        expand_occurrences, target_horizon,
    },
    time_entry::commands::{
        AttachTimeEntryPhotoCommand, DeclareTimeEntryCommand, EndDayCommand, StartTimeEntryCommand,
        StopTimeEntryCommand,
    },
    time_entry::service::TimeEntryService,
    user::commands::{CreateUserCommand, UpsertUserBySubCommand},
    weekday_from_iso, weekday_to_iso,
    work_time::commands::{
        ReplaceRhythmCommand, ReplaceWorkSlotsCommand, RhythmSlotInput, WorkSlotInput,
    },
    work_time::service::expand_work_slots,
};
