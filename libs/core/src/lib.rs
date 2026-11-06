extern crate self as mestier_core;

pub mod application;
pub(crate) mod domain;
pub mod infrastructure;

pub use application::automation::oauth::StartOauthCommand;
pub use application::automation::trigger::WebhookAuthentication;
pub use application::supplier_invoice::{ImportSupplierInvoiceOutcome, TotalsMismatch};
pub use application::*;
pub use domain::automation::collection::{
    ApiKeyLocation, Collection, CollectionDetail, CollectionDraft, CollectionDraftInput,
    CollectionError, CollectionSource, CollectionSummary, CollectionVersion, HttpMethod,
    ImportCollectionCommand, MAX_COLLECTION_BYTES, Operation, Parameter, ParameterLocation,
    ReplaceCollectionCommand, RequestBody, SecurityKind, SecurityScheme,
};
pub use domain::automation::connector::{
    AuthRequirement, ConnectorCatalogue, ConnectorCatalogueError, ConnectorDescriptor, Field,
    FieldKind, SelectOption, VisibleWhen, connector_catalogue,
};
pub use domain::automation::credential::{
    CreateCredentialCommand, Credential, CredentialError, CredentialOrigin, CredentialStatus,
    MAX_CREDENTIAL_DESCRIPTION_CHARS, UpdateCredentialCommand,
};
pub use domain::automation::credential_kind::{
    CredentialCategory, CredentialField, CredentialFieldKind, CredentialKind, CredentialOption,
    CredentialUsage, HTTP_CREDENTIAL_KINDS, ShowWhen, WEBHOOK_CREDENTIAL_KINDS, credential_kind,
    credential_kinds,
};
pub use domain::automation::event_catalogue;
pub use domain::automation::expression::{
    ExpressionContext, ExpressionError, LoopFrame, Template, parse_template,
};
pub use domain::automation::folder::{
    CreateFolderCommand, FolderName, FolderNameError, FolderRefusal, FolderTree, MAX_FOLDER_DEPTH,
    MAX_FOLDER_NAME_LENGTH, MAX_FOLDERS_PER_ORGANIZATION, UpdateFolderCommand, WorkflowFolder,
};
pub use domain::automation::oauth::{
    OauthCallback, OauthOutcome, is_valid_redirect_uri, is_valid_return_to,
    with_outcome as with_oauth_outcome,
};
pub use domain::automation::ports::DispatchOutcome;
pub use domain::automation::run::{
    Connector, ConnectorInput, ConnectorOutcome, DryRunDispatch, DryRunError, DryRunInput,
    DryRunLimit, DryRunStatus, DryRunStep, DueRun, MAX_DRY_RUN_BYTES, MAX_DRY_RUN_DURATION,
    MAX_DRY_RUN_EDGES, MAX_DRY_RUN_FIELD_BYTES, MAX_DRY_RUN_LOOP_ITEMS, MAX_DRY_RUN_NODE_BYTES,
    MAX_DRY_RUN_NODES, MAX_DRY_RUN_PAYLOAD_BYTES, MAX_DRY_RUN_PENDING, MAX_DRY_RUN_STEPS, Run,
    RunOverview, RunSettlement, RunStatus, RunStep, StepStatus, StoredOperation,
};
pub use domain::automation::secret::SealedSecret;
pub use domain::automation::settings::{AutomationSettings, SettingsBounds};
pub use domain::automation::workflow::{
    ArchiveScope, Branch, ContractKey, ContractKind, CreateWorkflowCommand, Edge, Frequency, Graph,
    GraphError, MAX_LAYOUT_ENTRIES, MAX_NODE_DESCRIPTION_LENGTH, MAX_NODE_NAME_LENGTH,
    MAX_RAW_TAGS, MAX_TAG_LENGTH, MAX_TAGS, MAX_WEBHOOK_BODY_BYTES, MAX_WEBHOOK_PATH_LENGTH,
    MAX_WORKFLOW_DESCRIPTION_LENGTH, MAX_WORKFLOW_NAME_LENGTH, NodePosition, NodeRef, OnInvalid,
    OperationRef, PlacedConnector, PlacedTrigger, ResolvedWebhook, SaveWorkflowVersionCommand,
    TriggerKind, UpdateWorkflowCommand, WEBHOOK_PREFIX, WebhookMethod, WebhookPathIssue, Weekday,
    Workflow, WorkflowDescription, WorkflowDescriptionError, WorkflowInitialState, WorkflowLayout,
    WorkflowName, WorkflowNameError, WorkflowRefusal, WorkflowTags, WorkflowTagsError,
    WorkflowVersion, check_webhook_path, contract_violations, holds_nul, is_json_content_type,
    validate_graph,
};
pub use infrastructure::automation::connectors::{
    ConnectorDryRun, ConnectorRegistry, UnknownConnectorKind,
};
pub use infrastructure::automation::webhook::{
    address_policy::PrivateNetworkAccess,
    inbound::{SignatureAlgorithm, SignatureSpec, path_fingerprint},
    secret::SecretCipher,
};
pub use infrastructure::automation::worker::{WorkerSchedule, run_automation_worker};
pub use infrastructure::invoice::facturx::FacturXDocumentFormat;
pub use infrastructure::realtime::EventHub;
pub use infrastructure::supplier_invoice::facturx::FacturXParser;

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

/// Turning an accepted quote into a project's proposed tasks (#298) — the
/// proposal type and the guards a handler needs to answer `GET
/// .../plan-proposal` and refuse an invalid `POST .../plan` with its own
/// error rather than a 500.
pub use domain::quote::service::{
    TaskProposal, propose_tasks_from_quote, require_quote_accepted, validate_quote_plannable,
};

/// Exposed for the BDD suite under `tests/role_bdd`, which drives both
/// services against `mockall` doubles of their repository ports — a role
/// assignment touches both. Mirrors [`QuoteService`]'s own exposure above:
/// the ports stay crate-private, only the services and commands needed to
/// drive them from outside the crate are public.
pub use domain::member::service::MemberService;
pub use domain::role::service::RoleService;

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
        MockAutomationSettingsRepository, MockCollectionRepository, MockCredentialRepository,
        MockEventDispatchRepository, MockEventLogRepository, MockRunRepository,
        MockSubscriptionRepository, MockWorkflowRepository,
    },
    custom_unit::ports::MockCustomUnitRepository,
    customer::ports::MockCustomerRepository,
    customer_contact::ports::MockCustomerContactRepository,
    customer_context::ports::MockCustomerContextRepository,
    doe::ports::MockDoeRepository,
    doe_model::ports::{MockDoeModelRepository, MockDoeSettingsRepository},
    employee::ports::{MockEmployeeCostBasisRepository, MockEmployeeRepository},
    equipment::ports::MockEquipmentRepository,
    invitation::ports::MockInvitationRepository,
    invoice::ports::{MockDocumentFormat, MockInvoiceRepository},
    maintenance::ports::{MockInterventionRepository, MockMaintenanceContractRepository},
    member::ports::MockMemberRepository,
    organization::ports::MockOrganizationRepository,
    planning::ports::MockPlanningRepository,
    product::ports::MockProductRepository,
    project::ports::MockProjectRepository,
    project_template::ports::MockProjectTemplateRepository,
    quote::ports::MockQuoteRepository,
    role::ports::MockRoleRepository,
    service_rate::ports::MockServiceRateRepository,
    supplier_invoice::ports::{
        MockSupplierInvoiceAllocationRepository, MockSupplierInvoiceRepository,
        MockSupplierRepository,
    },
    support::ports::{
        MockSupportApiKeyRepository, MockSupportSettingsRepository, MockTicketEntryRepository,
        MockTicketRepository,
    },
    task::ports::MockTaskRepository,
    task_comment::ports::MockTaskCommentRepository,
    task_label::ports::MockTaskLabelRepository,
    task_recurrence::ports::MockTaskRecurrenceRepository,
    user::ports::MockUserRepository,
    work_time::ports::{MockRhythmRepository, MockWorkSlotRepository},
};

pub use domain::{
    Cadence, ContractStatus, IntervalDays, Intervention, InterventionId, InterventionStatus,
    IsoWeekNumber, MaintenanceContract, MaintenanceContractId, MaintenanceLine, MaintenanceLineId,
    Season, Window, Windows, check_intervention_date, iso_week_of,
    maintenance::commands::{
        ArchiveMaintenanceContractCommand, CreateMaintenanceContractCommand,
        DuplicateMaintenanceContractCommand, MaintenanceLineInput, PlanMaintenanceLinesCommand,
        ProposeInterventionCommand, RecordInterventionCommand, RejectInterventionCommand,
        UpdateMaintenanceContractCommand, ValidateInterventionCommand,
    },
    maintenance::due::{Due, DueStatus, LineHistory, due_for},
    maintenance::events::{InterventionProposed, InterventionRejected, InterventionValidated},
    maintenance::ports::{
        InterventionRepository, MaintenanceContractRepository, TaskMaintenanceContext,
    },
};

pub use domain::{
    Attachment, AttachmentId, Badge, BadgeTone, CreatedTicket, CustomFields, DisplayFields,
    EntryKind, GeneratedApiKey, LocatedAttachment, MAX_DISPLAY_FIELD_CHARS, MAX_DISPLAY_FIELDS,
    PresentedApiKey, SupportApiKey, SupportApiKeyId, Ticket, TicketEntry, TicketEntryId, TicketId,
    TicketNumber, TicketStatus, api_key_matches, generate_api_key,
    support::commands::{
        AddBadgeCommand, AddEntryCommand, AssignTicketCommand, AssignToLeastRecentCommand,
        AttachFileCommand, CreateApiKeyCommand, CreateTicketCommand, CreatedApiKey, DraftCommand,
        EditDraftCommand, GetDisplayFieldsCommand, GetTicketCommand, ListApiKeysCommand,
        ListThreadCommand, ListTicketsCommand, PostClientMessageCommand, RemoveBadgeCommand,
        ReplaceDisplayFieldsCommand, RevokeApiKeyCommand, TransitionTicketCommand,
    },
    support::events::{TicketCreated, TicketTransitioned},
    support::ports::{
        LastAssignment, SupportApiKeyRepository, SupportSettingsRepository, TicketEntryRepository,
        TicketFilter, TicketPage, TicketRepository,
    },
    support::thread_service::NewAttachment,
    validate_api_key_name,
};

pub use domain::maintenance::overview::{
    ContractSite, Grid, GridContract, GridLine, GridVisit, Recap, RecapGroup, RecapLine,
    SiteContact,
};

pub use domain::{
    BlankPageCount, Doe, DoeFile, DoeFont, DoeId, DoeLabel, DoeModel, DoeModelId, DoeModelSection,
    DoeModelSectionKind, DoeParty, DoeSection, DoeSectionContent, DoeSettings, DoeStyle,
    DoeSummary, DoeSupply, DoeTemplate, HexColor, LogoPosition, LogoSize, MakerLabel, PartyRole,
    PhotosPerPage, Revision, StarterModel, SupplierSnapshot,
    doe::commands::{
        CreateDoeCommand, DeleteDoeCommand, DraftSupplyFromProductCommand, UpdateDoeCommand,
    },
    doe::render::{
        BundledFile, DocumentStyle, DoeDocument, DoeRenderError, DoeRenderer, FileBundle,
        NumberedContent, NumberedSection, NumberedSupply,
    },
    doe_model::commands::{
        CreateDoeModelCommand, CreateDoeModelFromStarterCommand, PreviewDoeStyleCommand,
        UpdateDoeModelCommand, UpdateDoeSettingsCommand,
    },
};

pub use domain::{
    Absence, AbsenceId, AbsenceKind, AssigneeRef, AssignmentReport, AssignmentReportId,
    AssignmentReportResolution, AvailabilityReport, BoardRank, Conflict, ConflictKind, CustomUnit,
    CustomUnitId, Customer, CustomerContact, CustomerContactId, CustomerContext, CustomerContextId,
    CustomerId, CustomerOutstandingBalance, CustomerPipelineStage, CustomerStatus, DateRange,
    DayLog, DayLogId, DeleteScope, DraftInvoice, ElectronicInvoicingFacts, Employee,
    EmployeeCostBasis, EmployeeCostBasisId, EmployeeId, EmployeeRhythm, EmployeeRhythmId,
    Equipment, EquipmentId, FileObject, GeneratedInvoiceDocument, Invitation, InvitationId,
    Invoice, InvoiceId, InvoiceKind, InvoiceLine, InvoiceLineId, InvoicePayment, InvoicePaymentId,
    InvoiceStatus, InvoiceVatBreakdownLine, LegalIdentity, Member, MemberId, MemberProfitability,
    MemberWithAccount, MemberWorkTime, MinuteInterval, MissingCost, MissingElectronicInvoicingFact,
    MissingLegalIdentityField, OperationNature, Organization, OrganizationAddress, OrganizationId,
    Permissions, PlannedAssignment, PlanningEntry, PlanningResource, PlanningTask, PlanningView,
    PresignedUrl, Product, ProductId, ProfitabilityFacts, ProfitabilityReport, Project,
    ProjectBillingSummary, ProjectId, ProjectProfitability, ProjectSupplierCostLine,
    ProjectTemplate, ProjectTemplateId, ProjectTemplateTask, ProjectTemplateTaskId, Quote, QuoteId,
    QuoteLine, QuoteLineId, QuoteStatus, QuoteVatBreakdownLine, RecurrenceFrequency,
    RecurrenceRule, ReportPeriod, RhythmSlot, RhythmSlotId, Role, RoleId, ServiceRate,
    ServiceRateId, ServiceRateUnit, StoredFile, Supplier, SupplierId, SupplierInvoice,
    SupplierInvoiceId, SupplierInvoiceLine, SupplierInvoiceLineAllocation,
    SupplierInvoiceLineAllocationId, SupplierInvoiceLineId, SupplierInvoiceReview,
    SupplierInvoiceSource, SupplierInvoiceStatus, SupplierInvoiceVatBreakdownLine, Task,
    TaskAssignment, TaskAssignmentId, TaskComment, TaskCommentId, TaskExpense, TaskId, TaskLabel,
    TaskLabelId, TaskRecurrence, TaskRecurrenceId, TaskStatus, TimeEntry, TimeEntryId,
    TimeEntryPhoto, TimeEntryPhotoId, TimeEntryPhotoPhase, TimeRange, Tz, User, UserId, VatStatus,
    WorkSlot, WorkSlotId,
    absence::commands::{CreateAbsenceCommand, PatchAbsenceCommand},
    assignment_report::commands::{
        AmendAssignmentReportCommand, ReportAssignmentCommand, ResolveAssignmentReportCommand,
        WithdrawAssignmentReportCommand,
    },
    custom_unit::commands::CreateCustomUnitCommand,
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
    invoice::commands::{
        CancelInvoiceCommand, CreateInvoiceCommand, DeleteInvoicePaymentCommand,
        InvoiceLineCommand, IssueCreditNoteCommand, IssueDepositCommand, IssueFinalInvoiceCommand,
        IssueInvoiceCommand, RecordInvoicePaymentCommand, UpdateInvoiceCommand,
    },
    invoice::ports::{
        DocumentFormat, DocumentFormatError, GeneratedDocument, InvoiceDocumentRequest,
    },
    member::commands::{
        AddMemberCommand, AssignRoleCommand, CreateMemberCommand, UnassignRoleCommand,
        UpdateMemberCommand,
    },
    organization::commands::{
        CreateOrganizationCommand, UpdateLegalIdentityCommand, UpdateOrganizationCommand,
    },
    planning::service::detect_conflicts,
    product::commands::{CreateProductCommand, UpdateProductCommand},
    project::commands::{
        CreateProjectCommand, CreateProjectFromQuoteCommand, PlannedTaskCommand,
        UpdateProjectCommand,
    },
    project::service::build_planned_tasks,
    project_template::commands::{
        CreateProjectTemplateCommand, InstantiateProjectTemplateCommand,
        ProjectTemplateTaskShapeCommand, ReplaceProjectTemplateTasksCommand,
        UpdateProjectTemplateCommand,
    },
    quote::commands::{
        CreateQuoteCommand, QuoteLineCommand, UpdateQuoteCommand, UpdateQuoteStatusCommand,
    },
    role::commands::{CreateRoleCommand, UpdateRoleCommand},
    service_rate::commands::{CreateServiceRateCommand, UpdateServiceRateCommand},
    supplier_invoice::commands::{
        AllocateSupplierInvoiceLineCommand, ConfirmSupplierInvoiceCommand,
        CreateSupplierInvoiceCommand, LineAllocationShare, RejectSupplierInvoiceCommand,
        ReplaceSupplierInvoiceLineAllocationsCommand, SupplierInvoiceLineCommand,
        UpdateSupplierInvoiceNotesCommand,
    },
    supplier_invoice::commands::{CreateSupplierCommand, UpdateSupplierCommand},
    task::commands::{CreateTaskCommand, PatchTaskCommand},
    task::service::{
        resolve_task_window, sort_by_board_rank, validate_parent_depth, validate_reparenting,
    },
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
