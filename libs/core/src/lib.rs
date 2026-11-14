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
    ACTION_FAMILY, ActionSpec, AuthRequirement, ConnectorCatalogue, ConnectorCatalogueError,
    ConnectorDescriptor, ExecutionMode, Field, FieldKind, SelectOption, VisibleWhen,
    action_descriptors, action_specs, connector_catalogue, find_action,
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
    OauthCallback, OauthOutcome, RedirectRefusal, check_redirect_uri, is_valid_return_to,
    return_origins, with_outcome as with_oauth_outcome,
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
pub use domain::quote::service::{QuoteChange, QuoteLookup, QuoteLookups, QuoteService};

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

pub use domain::{
    Answer, Answers, Campaign, CampaignId, CampaignStatus, CatalogProfile, CatalogProfileInput,
    CatalogUsage, Closed, Completed, DocumentVatRow, Draft, Expense, ExpenseCategory,
    ExpenseCategoryId, ExpenseCategoryService, ExpenseDisplayState, ExpenseError, ExpenseField,
    ExpenseId, ExpenseOutcome, ExpenseService, ExpenseStatus, GraphPath, InProgress, Ledger,
    LedgerEntry, LedgerError, LedgerKind, LedgerMonth, LedgerMonthError, LedgerOutcome,
    LedgerService, LedgerStatus, LedgerSummary, Metadata, MonthTrend, Opened, Payables, QuestionId,
    QuestionKind, Receipt, ReceiptId, ReceiptRow, ReceiptService, Response, ResponseId,
    ResponseRecord, ResumeToken, SupplierAddress, SupplierAddressInput, SupplierContact,
    SupplierContactInput, SupplierError, SupplierField, SupplierOutcome, SupplierProfile,
    SupplierProfileInput, Survey, SurveyError, SurveyGraph, SurveyId, SurveyRecord, SurveyWindow,
    VatSummary, Visibility, build_ledger,
};

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
    absence_type::ports::MockAbsenceTypeRepository,
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
    day_override::ports::MockDayOverrideRepository,
    deal::ports::MockDealRepository,
    doe::ports::MockDoeRepository,
    doe_model::ports::{MockDoeModelRepository, MockDoeSettingsRepository},
    employee::ports::{MockEmployeeCostBasisRepository, MockEmployeeRepository},
    equipment::ports::MockEquipmentRepository,
    expense::ports::{MockExpenseCategoryRepository, MockExpenseRepository},
    extra_period::ports::MockExtraPeriodRepository,
    invitation::ports::MockInvitationRepository,
    invoice::ports::{MockDocumentFormat, MockInvoiceRepository},
    knowledge::ports::{MockKnowledgeArticleRepository, MockKnowledgeFolderRepository},
    knowledge::tables::ports::{
        MockCellReferenceCheck, MockKnowledgeTableRepository, MockKnowledgeTableRowRepository,
    },
    leave_account::ports::MockLeaveAccountRepository,
    leave_adjustment::ports::MockLeaveAdjustmentRepository,
    ledger::ports::{MockLedgerRepository, MockReceiptRepository},
    maintenance::ports::{MockInterventionRepository, MockMaintenanceContractRepository},
    member::ports::MockMemberRepository,
    member_contract::ports::MockMemberContractRepository,
    member_document::ports::MockMemberDocumentRepository,
    member_profile::ports::MockMemberProfileRepository,
    news::ports::MockNewsRepository,
    organization::ports::MockOrganizationRepository,
    photo_library::ports::MockLibraryPhotoRepository,
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
    surcharge::ports::MockSurchargeRepository,
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
    survey::events::{
        SurveyCampaignEnded, SurveyCampaignStarted, SurveyOpened, SurveyResponseCompleted,
        SurveyResponseSubmitted,
    },
    survey::ports::{CampaignRepository, ResponseRepository, SurveyRepository},
    validate_api_key_name,
};

pub use domain::maintenance::overview::{
    ContractSite, Grid, GridContract, GridLine, GridVisit, Recap, RecapGroup, RecapLine,
    SiteContact,
};

pub use domain::{
    CreateNewsCommand, MAX_NEWS_ITEMS, MAX_SEEN_IDS, News, NewsContent, NewsContentInput, NewsCta,
    NewsCtaInput, NewsError, NewsIcon, NewsId, NewsItem, NewsItemInput, NewsOutcome,
    UpdateNewsCommand, check_seen_ids,
};

pub use domain::{
    CreationCounts, FeatureActivation, FeatureChange, FeatureFlag, FlagAdoption, HISTORY_LIMIT,
    OrganizationDetail, OrganizationFeature, OrganizationMemberSummary, OrganizationOverview,
    OrganizationSummary, PlatformAdmin, PlatformStats, RECENT_DAYS, STATS_WEEKS,
    UnknownFeatureFlag, UserOrganization, UserSummary, WeeklyCount,
    feature_flag::events::{FEATURE_DISABLED, FEATURE_ENABLED, FeatureDisabled, FeatureEnabled},
    feature_flag::ports::FeatureFlagRepository,
    platform::ports::PlatformAdminRepository,
};

pub use domain::{
    BackfillReport, BrowseKnowledgeFolderCommand, CreateKnowledgeFolderCommand,
    CreateKnowledgeNoteCommand, DeleteKnowledgeCommand, DeleteKnowledgeFolderCommand,
    GetKnowledgeCommand, InsertOutcome, KnowledgeArticle, KnowledgeArticleId,
    KnowledgeArticleWithLinks, KnowledgeError, KnowledgeFile, KnowledgeFilter, KnowledgeFolder,
    KnowledgeFolderId, KnowledgeFolderScope, KnowledgeFolderView, KnowledgeKind, KnowledgeLink,
    KnowledgeOutcome, KnowledgeResource, KnowledgeResourceType, LibraryPhoto, LibraryPhotoId,
    LinkKnowledgeCommand, ListKnowledgeCommand, ListKnowledgeFoldersCommand,
    MoveKnowledgeArticlesCommand, PhotoFilter, PhotoLibraryError, PhotoLibraryOutcome,
    PlacementReport, PlacementResult, PlacementStatus, UpdateKnowledgeCommand,
    UpdateKnowledgeFolderCommand, UploadKnowledgeFileCommand, UploadedPhoto,
    doe::service::PHOTOS_SECTION_TITLE,
    knowledge::service::MAX_PAGE_SIZE as KNOWLEDGE_MAX_PAGE_SIZE,
    knowledge::text::{
        MAX_EXTRACTED_CHARS, MAX_FILE_BYTES, MAX_MOVED_ARTICLES,
        check_file as check_knowledge_file, extract_text as extract_knowledge_text,
        normalize_markdown as normalize_knowledge_markdown,
        normalize_tags as normalize_knowledge_tags, normalize_title as normalize_knowledge_title,
    },
    photo_library::commands::{
        DeleteLibraryPhotoCommand, GetLibraryPhotoCommand, ListLibraryPhotosCommand,
        MAX_PLACEMENTS, PlaceLibraryPhotosCommand, PlacementRequest, PlacementTargetKind,
        UpdateLibraryPhotoCommand, UploadLibraryPhotoCommand,
    },
    photo_library::normalize::{NormalizedImage, normalize_image},
};

pub use domain::knowledge::documents::{
    ContentError as DocumentContentError, CreateKnowledgeDocumentCommand, DocumentContent,
    DocumentLockCommand, DocumentLockGrant, DocumentOutcome, DocumentRefusal, EditLock,
    SaveDocumentContentCommand, SavedDocument, content::MAX_JSON_BYTES as MAX_DOCUMENT_JSON_BYTES,
    from_markdown as document_from_markdown, render as render_document,
};

pub use domain::knowledge::tables::csv::{
    ImportReport as TableImportReport, InvalidCsvCell, MAX_CSV_BYTES, MAX_CSV_ROWS,
};
pub use domain::knowledge::tables::import::ImportedTable;
pub use domain::knowledge::tables::rows::{KnowledgeTableRowService, TableExport};
pub use domain::knowledge::tables::service::KnowledgeTableService;
pub use domain::knowledge::tables::{
    CellError, CellLabel, CellReference, CellValue, ColumnConfig, ColumnKind, ColumnType,
    CreateKnowledgeTableCommand, CreateTableColumnCommand, CreateTableRowCommand,
    CreateTableViewCommand, Decimals, DeleteKnowledgeTableCommand, DeleteTableColumnCommand,
    DeleteTableRowsCommand, DeleteTableViewCommand, ExportTableViewCommand, FilterOperator,
    GetKnowledgeTableCommand, GetTableRowCommand, ImportKnowledgeTableCommand,
    ImportTableRowsCommand, KnowledgeTable, KnowledgeTableId, ListKnowledgeTablesCommand,
    ListTableRowsCommand, MAX_COLUMNS as MAX_TABLE_COLUMNS,
    MAX_DELETED_ROWS as MAX_DELETED_TABLE_ROWS, MAX_ROW_PAGE as MAX_TABLE_ROW_PAGE,
    MAX_ROWS as MAX_TABLE_ROWS, ReorderTableColumnsCommand, ResolvedRow, RowBatch, RowFilter,
    RowPage, RowPatch, RowSelection, RowSort, RowValues, SelectColor,
    SelectOption as TableSelectOption, SelectOptionId, SelectOptionInput, SelectOptions,
    SortDirection, TableColumn, TableColumnId, TableDescription, TableError, TableItemName,
    TableOutcome, TableRow, TableRowId, TableSchema, TableView, TableViewId,
    UpdateKnowledgeTableCommand, UpdateTableColumnCommand, UpdateTableRowCommand,
    UpdateTableViewCommand, ViewConfig, ViewKind,
};

pub use domain::{
    BlankPageCount, CoverBlock, CoverLayout, DecorationLayer, DecorationPages, DecorationShape,
    Doe, DoeDecoration, DoeFile, DoeFont, DoeId, DoeLabel, DoeModel, DoeModelId, DoeModelSection,
    DoeModelSectionKind, DoeParty, DoeSection, DoeSectionContent, DoeSettings, DoeStyle,
    DoeSummary, DoeSupply, DoeTemplate, HexColor, ImageFit, LogoPosition, LogoSize, MakerLabel,
    Millimeters, Opacity, PartyRole, PhotosPerPage, ProjectDoeStyle, Revision, Rotation,
    StarterModel, SupplierSnapshot, SupplyPhotoColumns,
    doe::commands::{
        CreateDoeCommand, DeleteDoeCommand, DraftSupplyFromProductCommand, UpdateDoeCommand,
    },
    doe::render::{
        BundledFile, DocumentStyle, DoeDocument, DoeRenderError, DoeRenderer, FileBundle,
        NumberedContent, NumberedSection, NumberedSupply,
    },
    doe_model::commands::{
        CreateDoeModelCommand, CreateDoeModelFromStarterCommand, PreviewDoeStyleCommand,
        ProjectDoeStyleCommand, UpdateDoeModelCommand, UpdateDoeSettingsCommand,
        UpdateProjectDoeStyleCommand,
    },
};

pub use domain::{
    DayOverride, DayOverrideError, DayOverrideId, DayOverrideInput, DayOverrideOutcome,
    DayOverrideService, DayShape, FeedMember, HrPlanningFeed, OverrideKind, PlanningPeriod,
    PlanningPeriodError,
};

pub use domain::{
    DefaultSurcharge, ExtraPeriod, ExtraPeriodDraft, ExtraPeriodError, ExtraPeriodFilter,
    ExtraPeriodId, ExtraPeriodOutcome, ExtraPeriodService, ExtraReason, ExtraStatus, MAX_RATE_BP,
    RateBp, Surcharge, SurchargeError, SurchargeId, SurchargeInput, SurchargeKind, SurchargeName,
    SurchargeOutcome, SurchargeService, default_surcharges,
};

pub use domain::{
    Acquisition, AcquisitionDraft, AdjustmentDays, AdjustmentKind, AdjustmentReason, CarryOverMax,
    DefaultLeaveAccount, LeaveAccount, LeaveAccountError, LeaveAccountId, LeaveAccountInput,
    LeaveAccountName, LeaveAccountOutcome, LeaveAccountService, LeaveAdjustment,
    LeaveAdjustmentDraft, LeaveAdjustmentError, LeaveAdjustmentId, LeaveAdjustmentOutcome,
    LeaveAdjustmentService, LeaveDays, PeriodStartMonth, PositiveDays, default_leave_accounts,
};

pub use domain::{
    ARCHIVED_ACCOUNT_REASON, Absence, AbsenceError, AbsenceId, AbsenceKind, AbsenceOutcome,
    AbsenceSpan, AbsenceStatus, AbsenceType, AbsenceTypeError, AbsenceTypeId, AbsenceTypeInput,
    AbsenceTypeName, AbsenceTypeOutcome, AbsenceTypeService, AddressKind, AssigneeRef,
    AssignmentReport, AssignmentReportId, AssignmentReportResolution, AvailabilityReport,
    BillingFrequency, BillingSettings, BillingSettingsError, BillingSettingsField,
    BillingSettingsInput, BillingSettingsService, BlockInput, BoardRank, CREDIT_NOTE_NUMBER_PREFIX,
    ClockTime, Conflict, ConflictKind, ContactEmail, ContractPeriod, ContractSchedule,
    ContractType, CountMode, CustomUnit, CustomUnitId, Customer, CustomerContact,
    CustomerContactId, CustomerContactService, CustomerContext, CustomerContextId,
    CustomerContextService, CustomerDirectoryEntry, CustomerError, CustomerField, CustomerId,
    CustomerIndicators, CustomerInvoiceFact, CustomerKind, CustomerListRow, CustomerOutcome,
    CustomerOutstandingBalance, CustomerProfile, CustomerService, CustomerStatus, DateRange,
    DayLog, DayLogId, DayPart, DayRefusal, Deal, DealAmount, DealError, DealField, DealId,
    DealOutcome, DealQuoteLinks, DealService, DealStage, DealStageName, DealTitle,
    DefaultAbsenceType, DefaultVatRate, DeleteScope, DepositPercent, DiscountPercent, DocumentKind,
    DocumentName, DraftInvoice, ElectronicInvoicingFacts, Employee, EmployeeCostBasis,
    EmployeeCostBasisId, EmployeeId, EmployeeRhythm, EmployeeRhythmId, Equipment, EquipmentId,
    FileKey, FileObject, GeneratedInvoiceDocument, Hue, Invitation, InvitationId, Invoice,
    InvoiceCollection, InvoiceDerived, InvoiceDisplayState, InvoiceError, InvoiceEvent,
    InvoiceEventId, InvoiceEventKind, InvoiceField, InvoiceId, InvoiceKind, InvoiceLine,
    InvoiceLineId, InvoiceNumberSeries, InvoiceOutcome, InvoicePayment, InvoicePaymentId,
    InvoiceSettlement, InvoiceStatus, InvoiceVatBreakdownLine, IssuerBilling, IssuerSnapshot,
    LatePenalty, LeadSource, LegalIdentity, LostReason, LunchBreak, LunchInput, Member,
    MemberContract, MemberContractError, MemberContractId, MemberContractInput,
    MemberContractOutcome, MemberContractService, MemberDocument, MemberDocumentError,
    MemberDocumentId, MemberDocumentOutcome, MemberDocumentService, MemberId, MemberProfile,
    MemberProfileError, MemberProfileField, MemberProfileInput, MemberProfileOutcome,
    MemberProfileService, MemberProfitability, MemberWithAccount, MemberWorkTime, MinuteInterval,
    MissingCost, MissingElectronicInvoicingFact, MissingLegalIdentityField, NumberPadding,
    NumberSeries, NumberingError, NumberingOutcome, OperationNature, Organization,
    OrganizationAddress, OrganizationId, PaymentMethod, PaymentTermsDays, Permissions, PhoneNumber,
    PipelineSummary, PlannedAssignment, PlanningEntry, PlanningResource, PlanningTask,
    PlanningView, PostalAddress, PostalCode, PresignedUrl, PrimaryContact, Product, ProductId,
    ProfitabilityFacts, ProfitabilityReport, Project, ProjectBillingSummary, ProjectId,
    ProjectProfitability, ProjectSupplierCostLine, ProjectTemplate, ProjectTemplateId,
    ProjectTemplateTask, ProjectTemplateTaskId, Quote, QuoteError, QuoteField, QuoteId, QuoteLine,
    QuoteLineId, QuoteOutcome, QuoteStatus, QuoteTotals, QuoteValidityDays, QuoteVatBreakdownLine,
    RecipientSnapshot, RecoveryFeeCents, RecurrenceFrequency, RecurrenceRule, RecurringTotal,
    RegistrationNumber, ReportPeriod, RhythmSlot, RhythmSlotId, Role, RoleId, SUMMARY_WINDOW_DAYS,
    ScheduleBlock, ScheduleError, ScheduleInput, ScheduleRefusal, ScheduleUnit, SeriesNumbering,
    ServiceRate, ServiceRateId, ServiceRateUnit, SnapshotAddress, StageTotal, StoredFile, Supplier,
    SupplierId, SupplierInvoice, SupplierInvoiceId, SupplierInvoiceLine,
    SupplierInvoiceLineAllocation, SupplierInvoiceLineAllocationId, SupplierInvoiceLineId,
    SupplierInvoiceReview, SupplierInvoiceSource, SupplierInvoiceStatus,
    SupplierInvoiceVatBreakdownLine, Task, TaskAssignment, TaskAssignmentId, TaskComment,
    TaskCommentId, TaskExpense, TaskId, TaskLabel, TaskLabelId, TaskRecurrence, TaskRecurrenceId,
    TaskStatus, TimeEntry, TimeEntryId, TimeEntryPhoto, TimeEntryPhotoId, TimeEntryPhotoPhase,
    TimeRange, Tz, UnknownNumberSeries, User, UserId, VatStatus, WeekSchedule, WorkDay,
    WorkDayInput, WorkSlot, WorkSlotId,
    absence::commands::{
        AbsenceSchedule, CreateAbsenceCommand, DecideAbsenceCommand, ListAbsencesCommand,
        PatchAbsenceCommand, RejectAbsenceCommand,
    },
    absence_type::commands::{
        AbsenceTypeDraft, ArchiveAbsenceTypeCommand, CreateAbsenceTypeCommand,
        ListAbsenceTypesCommand, UpdateAbsenceTypeCommand,
    },
    assignment_report::commands::{
        AmendAssignmentReportCommand, ReportAssignmentCommand, ResolveAssignmentReportCommand,
        WithdrawAssignmentReportCommand,
    },
    custom_unit::commands::CreateCustomUnitCommand,
    customer::commands::{CreateCustomerCommand, UpdateCustomerCommand},
    customer_contact::commands::{CreateCustomerContactCommand, UpdateCustomerContactCommand},
    customer_context::DEFAULT_COUNTRY,
    customer_context::commands::{CreateCustomerContextCommand, UpdateCustomerContextCommand},
    day_override::commands::{
        DeleteDayOverrideCommand, HrPlanningFeedCommand, ListDayOverridesCommand,
        PutDayOverrideCommand,
    },
    deal::commands::{CreateDealCommand, DealFilter, MoveDealCommand, UpdateDealCommand},
    default_absence_types,
    employee::commands::{
        CorrectEmployeeCostBasisCommand, RemoveEmployeeProfileCommand, SetEmployeeCostBasisCommand,
        UpsertEmployeeProfileCommand,
    },
    equipment::commands::{CreateEquipmentCommand, UpdateEquipmentCommand},
    expense::commands::{
        CreateExpenseCategoryCommand, CreateExpenseCommand, MarkExpensePaidCommand,
        UpdateExpenseCategoryCommand, UpdateExpenseCommand,
    },
    extra_period::commands::{
        CreateExtraPeriodCommand, DecideExtraPeriodCommand, ListExtraPeriodsCommand,
        RejectExtraPeriodCommand,
    },
    file_storage::commands::UploadFileCommand,
    format_number,
    invitation::commands::{AcceptInvitationCommand, InviteMemberCommand, RevokeInvitationCommand},
    invoice::commands::{
        CancelInvoiceCommand, CreateInvoiceCommand, CreateInvoiceFromQuoteCommand,
        CreditLineRequest, CreditRequest, DeleteInvoicePaymentCommand, InvoiceLineCommand,
        IssueCreditNoteCommand, IssueDepositCommand, IssueFinalInvoiceCommand, IssueInvoiceCommand,
        NoteInvoiceCommand, QuoteInvoiceMode, RecordInvoicePaymentCommand, RemindInvoiceCommand,
        SendInvoiceCommand, UpdateInvoiceCommand,
    },
    invoice::ports::{
        DocumentFormat, DocumentFormatError, GeneratedDocument, InvoiceDocumentRequest,
    },
    invoice::rules::{
        QuoteInvoicing, QuoteInvoicingBlocker, line_net_cents as invoice_line_net_cents,
    },
    invoice::service::QuoteInvoicingView,
    leave_account::commands::{
        ArchiveLeaveAccountCommand, CreateLeaveAccountCommand, LeaveAccountDraft,
        ListLeaveAccountsCommand, UpdateLeaveAccountCommand,
    },
    leave_adjustment::commands::{
        CreateLeaveAdjustmentCommand, DeleteLeaveAdjustmentCommand, ListLeaveAdjustmentsCommand,
    },
    ledger::commands::{AllocateReceiptCommand, RecordReceiptCommand},
    member::commands::{
        AddMemberCommand, AssignRoleCommand, CreateMemberCommand, UnassignRoleCommand,
        UpdateMemberCommand,
    },
    member_contract::commands::{
        CreateMemberContractCommand, DeleteMemberContractCommand, ListMemberContractsCommand,
        UpdateMemberContractCommand,
    },
    member_document::commands::{
        AddMemberDocumentCommand, DeleteMemberDocumentCommand, ListMemberDocumentsCommand,
    },
    member_profile::commands::{
        ArchiveMemberCommand, GetMemberProfileCommand, ListMemberProfilesCommand,
        PutMemberProfileCommand, ReportsPolicy, RestoreMemberCommand,
    },
    organization::commands::{
        CreateOrganizationCommand, ReplaceBillingSettingsCommand, SetNextNumberCommand,
        UpdateLegalIdentityCommand, UpdateOrganizationCommand,
    },
    payment_terms_of,
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
        CreateQuoteCommand, QuoteLineCommand, QuoteTermsCommand, UpdateQuoteCommand,
        UpdateQuoteStatusCommand,
    },
    role::commands::{CreateRoleCommand, UpdateRoleCommand},
    service_rate::commands::{CreateServiceRateCommand, UpdateServiceRateCommand},
    siren_of,
    supplier_invoice::commands::{
        AllocateSupplierInvoiceLineCommand, ConfirmSupplierInvoiceCommand,
        CreateSupplierInvoiceCommand, LineAllocationShare, RejectSupplierInvoiceCommand,
        ReplaceSupplierInvoiceLineAllocationsCommand, SupplierInvoiceLineCommand,
        UpdateSupplierInvoiceNotesCommand,
    },
    supplier_invoice::commands::{CreateSupplierCommand, UpdateSupplierCommand},
    surcharge::commands::{
        ArchiveSurchargeCommand, CreateSurchargeCommand, ListSurchargesCommand, SurchargeDraft,
        UpdateSurchargeCommand,
    },
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
