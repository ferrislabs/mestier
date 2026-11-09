export namespace Schemas {
  // <Schemas>
  export type AbsenceId = string;
  export type AbsenceKind = "LEAVE" | "SICK" | "UNAVAILABLE";
  export type MemberId = string;
  export type DayPart = "MORNING" | "AFTERNOON";
  export type OrganizationId = string;
  export type AbsenceStatus = "PENDING" | "APPROVED" | "REJECTED" | "CANCELLED";
  export type AbsenceTypeId = string;
  export type AbsenceResponse = {
    all_day: boolean;
    created_at: string;
    created_by_member_id?: (null | MemberId) | undefined;
    decided_at?: (string | null) | undefined;
    decided_by_member_id?: (null | MemberId) | undefined;
    decision_comment?: (string | null) | undefined;
    end_date: string;
    end_part: DayPart;
    ends_at: string;
    id: AbsenceId;
    kind: AbsenceKind;
    member_id: MemberId;
    note?: (string | null) | undefined;
    organization_id: OrganizationId;
    proof_file_key?: (string | null) | undefined;
    start_date: string;
    start_part: DayPart;
    starts_at: string;
    status: AbsenceStatus;
    type_id: AbsenceTypeId;
    updated_at: string;
  };
  export type CountMode = "WORKING" | "WORKABLE" | "CALENDAR";
  export type AbsenceTypeRequest = {
    account_id?: (string | null) | undefined;
    count_mode: CountMode;
    hue: number;
    kind: AbsenceKind;
    name: string;
    needs_approval: boolean;
    needs_proof: boolean;
    paid: boolean;
  };
  export type AbsenceTypeResponse = {
    account_id?: (string | null) | undefined;
    archived_at?: (string | null) | undefined;
    count_mode: CountMode;
    created_at: string;
    hue: number;
    id: AbsenceTypeId;
    kind: AbsenceKind;
    name: string;
    needs_approval: boolean;
    needs_proof: boolean;
    organization_id: OrganizationId;
    paid: boolean;
  };
  export type AcquisitionBody = { kind: "MONTHLY"; per_month: number } | { days: number; kind: "ANNUAL" };
  export type DocumentKind = "CONTRACT" | "IDENTITY" | "DIPLOMA" | "MEDICAL" | "OTHER";
  export type AddMemberDocumentRequest = { file_key: string; kind: DocumentKind; name: string };
  export type AddTicketBadgeRequest = { label: string; tone?: (string | null) | undefined };
  export type AddTicketEntryRequest = { body: string; kind: string };
  export type AddressKind = "BILLING" | "SITE" | "OTHER";
  export type AdjustmentKind = "ADJUSTMENT" | "CARRY_OVER";
  export type InvoiceId = string;
  export type AllocateReceiptRequest = { invoice_id: InvoiceId };
  export type AmendAssignmentReportRequest = { comment?: (string | null) | undefined; reported_minutes: number };
  export type ApiKeyLocationDto = "header" | "query" | "cookie";
  export type ArchiveMemberRequest = Partial<{ reassign_reports_to: null | MemberId }>;
  export type RoleId = string;
  export type AssignRoleRequest = { role_id: RoleId };
  export type AssignTicketRequest = Partial<{ member_id: null | MemberId }>;
  export type AssigneeRefRequest = { member_id: MemberId };
  export type AssignmentReportId = string;
  export type AssignmentReportResolution = "PENDING" | "APPLIED" | "DISMISSED";
  export type TaskAssignmentId = string;
  export type AssignmentReportResponse = {
    comment?: (string | null) | undefined;
    created_at: string;
    id: AssignmentReportId;
    organization_id: OrganizationId;
    reported_by: MemberId;
    reported_minutes: number;
    resolution: AssignmentReportResolution;
    resolution_note?: (string | null) | undefined;
    resolved_at?: (string | null) | undefined;
    resolved_by?: (null | MemberId) | undefined;
    task_assignment_id: TaskAssignmentId;
    updated_at: string;
  };
  export type TimeEntryPhotoPhase = "BEFORE" | "DURING" | "AFTER";
  export type AttachPhotoRequest = { phase: TimeEntryPhotoPhase; storage_key: string };
  export type AttachmentId = string;
  export type AttachmentResponse = { filename: string; mime_type: string; size_bytes: number; storage_key: string };
  export type AttachmentUrlResponse = { expires_at: string; url: string };
  export type AuthRequirementResponse =
    | "None"
    | { Exactly: string }
    | { AnyOf: Array<string> }
    | { Optional: Array<string> };
  export type SelectOptionResponse = { label: string; value: string };
  export type FieldKindResponse =
    | "Text"
    | "Number"
    | "Bool"
    | { Select: { options: Array<SelectOptionResponse> } }
    | "Json";
  export type VisibleWhenResponse = { any_of: Array<string>; field: string };
  export type FieldResponse = {
    expression: boolean;
    kind: FieldKindResponse;
    label: string;
    name: string;
    required: boolean;
    secret: boolean;
    visible_when?: (null | VisibleWhenResponse) | undefined;
  };
  export type AuthSchemeResponse = { fields: Array<FieldResponse>; kind: string; label: string };
  export type AuthorType = "USER" | "WEBHOOK" | "SYSTEM";
  export type AutomationSettingsBody = {
    disable_target_after?: (number | null) | undefined;
    event_retention_seconds: number;
    retry_schedule_seconds: Array<number>;
    succeeded_run_retention_seconds: number;
  };
  export type TaskId = string;
  export type ConflictResponse =
    | { ends_at: string; kind: "absence"; note?: (string | null) | undefined; reason: AbsenceKind; starts_at: string }
    | { ends_at: string; kind: "outside_work_hours"; starts_at: string }
    | { ends_at: string; kind: "overlapping_task"; starts_at: string; task_id: TaskId };
  export type AvailabilityResourceResponse = {
    available: boolean;
    conflicts: Array<ConflictResponse>;
    resource_id: string;
  };
  export type AvailabilityResponse = { resources: Array<AvailabilityResourceResponse> };
  export type BadgeTone = "neutral" | "brand" | "success" | "warning" | "error";
  export type BadgeResponse = { label: string; tone: BadgeTone };
  export type BillingFrequency = "ONE_SHOT" | "MONTHLY" | "QUARTERLY" | "YEARLY";
  export type BillingSettingsResponse = {
    bic?: (string | null) | undefined;
    default_vat_rate_bp: number;
    iban?: (string | null) | undefined;
    invoice_footer: string;
    late_penalty: string;
    number_padding: number;
    payment_terms_days: number;
    quote_validity_days: number;
    recovery_fee_cents: number;
  };
  export type BlankPageCount = number;
  export type ScheduleUnit = "weeks" | "months";
  export type LunchInput = { end: string; start: string };
  export type WorkDayInput = { end: string; lunch?: (null | LunchInput) | undefined; start: string };
  export type BlockInput = {
    label?: string | undefined;
    length: number;
    unit: ScheduleUnit;
    week: Array<null | WorkDayInput>;
  };
  export type BranchDto = "Then" | "Else" | "Each" | "After";
  export type BulkAssignTasksRequest = { assignees: Array<AssigneeRefRequest>; task_ids: Array<TaskId> };
  export type TaskAssignmentSummary = { id: TaskAssignmentId; member_id: MemberId };
  export type CustomerContextId = string;
  export type CustomerId = string;
  export type EquipmentId = string;
  export type TaskEquipmentResponse = {
    created_at: string;
    hourly_rate_cents: number;
    id: EquipmentId;
    name: string;
    organization_id: OrganizationId;
    updated_at: string;
  };
  export type TaskLabelId = string;
  export type TaskLabelResponse = {
    color: string;
    created_at: string;
    id: TaskLabelId;
    name: string;
    organization_id: OrganizationId;
    updated_at: string;
  };
  export type MaintenanceLineId = string;
  export type ProjectId = string;
  export type QuoteId = string;
  export type TaskRecurrenceId = string;
  export type TaskStatus = "BACKLOG" | "PLANNED" | "IN_PROGRESS" | "DONE" | "CANCELLED";
  export type TaskResponse = {
    all_day: boolean;
    assignments: Array<TaskAssignmentSummary>;
    blocks_availability: boolean;
    child_count?: (number | null) | undefined;
    created_at: string;
    customer_context_id?: (null | CustomerContextId) | undefined;
    customer_id?: (null | CustomerId) | undefined;
    description?: (string | null) | undefined;
    ends_at?: (string | null) | undefined;
    equipment: Array<TaskEquipmentResponse>;
    expenses_cents: number;
    expenses_label?: (string | null) | undefined;
    id: TaskId;
    labels: Array<TaskLabelResponse>;
    maintenance_line_id?: (null | MaintenanceLineId) | undefined;
    member_ids: Array<MemberId>;
    organization_id: OrganizationId;
    parent_task_id?: (null | TaskId) | undefined;
    project_id?: (null | ProjectId) | undefined;
    quote_id?: (null | QuoteId) | undefined;
    recurrence_id?: (null | TaskRecurrenceId) | undefined;
    starts_at?: (string | null) | undefined;
    status: TaskStatus;
    title: string;
    updated_at: string;
  };
  export type BulkAssignTasksResponse = { tasks: Array<TaskResponse> };
  export type ButtonStyle = "Link";
  export type IntervalDays = number;
  export type IsoWeekNumber = number;
  export type Window = { from_week: IsoWeekNumber; to_week: IsoWeekNumber };
  export type Windows = Array<Window>;
  export type Cadence =
    | { every_days: IntervalDays; kind: "ROLLING" }
    | { kind: "WINDOWS"; windows: Windows }
    | { kind: "ON_DEMAND" };
  export type WindowRequest = { from_week: number; to_week: number };
  export type CadenceRequest =
    | { every_days: number; kind: "ROLLING" }
    | { kind: "WINDOWS"; windows: Array<WindowRequest> }
    | { kind: "ON_DEMAND" };
  export type CallbackQuery = Partial<{
    code: string | null;
    error: string | null;
    oauth_token: string | null;
    oauth_verifier: string | null;
    state: string | null;
  }>;
  export type CategoryId = string;
  export type CategoryResponse = {
    created_at: string;
    id: CategoryId;
    name: string;
    organization_id: OrganizationId;
    position: number;
    updated_at: string;
  };
  export type ChannelId = string;
  export type ChannelType = "TEXT" | "THREAD" | "DIRECT" | "ANNOUNCEMENT" | "VOICE";
  export type MessageId = string;
  export type ChannelResponse = {
    archived: boolean;
    category_id?: (null | CategoryId) | undefined;
    channel_type: ChannelType;
    created_at: string;
    id: ChannelId;
    name: string;
    organization_id: OrganizationId;
    origin_message_id?: (null | MessageId) | undefined;
    parent_id?: (null | ChannelId) | undefined;
    position: number;
    project_id?: (null | ProjectId) | undefined;
    topic?: (string | null) | undefined;
    updated_at: string;
  };
  export type ChatRoleResponse = { name: string; role_id: RoleId };
  export type HttpMethodDto = "GET" | "POST" | "PUT" | "PATCH" | "DELETE";
  export type ParameterLocationDto = "path" | "query" | "header";
  export type ParameterDto = {
    description?: string | undefined;
    location: ParameterLocationDto;
    name: string;
    required: boolean;
    schema?: unknown | undefined;
  };
  export type RequestBodyDto = { schema: unknown };
  export type OperationDto = {
    id: string;
    method: HttpMethodDto;
    parameters?: Array<ParameterDto> | undefined;
    path: string;
    request_body?: (null | RequestBodyDto) | undefined;
    response?: unknown | undefined;
    response_status?: (string | null) | undefined;
    summary?: string | undefined;
    tag?: string | undefined;
  };
  export type SecurityKindDto =
    | { scheme: string; type: "http" }
    | { location: ApiKeyLocationDto; parameter_name: string; type: "api_key" }
    | { type: "oauth2" }
    | { type: "open_id_connect" }
    | { declared_type: string; type: "other" };
  export type SecuritySchemeDto = { kind: SecurityKindDto; name: string };
  export type CollectionSourceDto = { kind: "url"; url: string } | { file_name: string; kind: "file" };
  export type CollectionRequest = {
    base_url: string;
    description?: string | undefined;
    operations: Array<OperationDto>;
    schemas?: Record<string, unknown> | undefined;
    security?: Array<SecuritySchemeDto> | undefined;
    source: CollectionSourceDto;
    title: string;
    version: string;
  };
  export type HistoryEntryResponse = {
    base_url: string;
    imported_at: string;
    security: Array<SecuritySchemeDto>;
    source: CollectionSourceDto;
    version: string;
  };
  export type CollectionResponse = {
    base_url: string;
    description: string;
    history: Array<HistoryEntryResponse>;
    id: string;
    imported_at: string;
    operations: Array<OperationDto>;
    organization_id: OrganizationId;
    schemas: Record<string, unknown>;
    security: Array<SecuritySchemeDto>;
    slug: string;
    source: CollectionSourceDto;
    title: string;
    version: string;
  };
  export type CollectionSummaryResponse = {
    base_url: string;
    id: string;
    imported_at: string;
    operation_count: number;
    organization_id: OrganizationId;
    security: Array<SecuritySchemeDto>;
    slug: string;
    source: CollectionSourceDto;
    title: string;
    version: string;
  };
  export type MediaItem = { description?: (string | null) | undefined; url: string };
  export type SeparatorSpacing = "Small" | "Large";
  export type Component =
    | { accent_color?: (number | null) | undefined; children: Array<Component>; type: "CONTAINER" }
    | { accessory?: (null | Component) | undefined; children: Array<Component>; type: "SECTION" }
    | { content: string; type: "TEXT_DISPLAY" }
    | { items: Array<MediaItem>; type: "MEDIA_GALLERY" }
    | { media: MediaItem; type: "THUMBNAIL" }
    | { divider: boolean; spacing?: (null | SeparatorSpacing) | undefined; type: "SEPARATOR" }
    | { components: Array<Component>; type: "ACTION_ROW" }
    | { emoji?: (string | null) | undefined; label: string; style: ButtonStyle; type: "BUTTON"; url: string };
  export type ConfirmSupplierInvoiceRequest = Partial<{ notes: string | null }>;
  export type ExecutionModeResponse = "per_item" | "once";
  export type ConnectorDescriptorResponse = {
    auth: AuthRequirementResponse;
    branches: Array<BranchDto>;
    description?: (string | null) | undefined;
    execution: ExecutionModeResponse;
    family: string;
    fields: Array<FieldResponse>;
    group?: (string | null) | undefined;
    kind: string;
    label: string;
    output_example: unknown;
    output_mirrors_field?: (string | null) | undefined;
    version: number;
  };
  export type ConnectorsResponse = {
    auth_schemes: Array<AuthSchemeResponse>;
    connectors: Array<ConnectorDescriptorResponse>;
  };
  export type MaintenanceLineRequest = {
    cadence: CadenceRequest;
    id?: (null | MaintenanceLineId) | undefined;
    label: string;
    minutes_per_intervention?: (number | null) | undefined;
  };
  export type SeasonRequest = { first_week: number; last_week: number; year: number };
  export type ContractStatus = "DRAFT" | "ACTIVE";
  export type ContractBody = {
    amount_cents?: (number | null) | undefined;
    lines: Array<MaintenanceLineRequest>;
    manager_member_id?: (null | MemberId) | undefined;
    notes?: (string | null) | undefined;
    quote_id?: (null | QuoteId) | undefined;
    reference?: (string | null) | undefined;
    season: SeasonRequest;
    status?: (null | ContractStatus) | undefined;
  };
  export type ContractKindDto = "Text" | "Number" | "Boolean" | "Object" | "List";
  export type ContractKeyDto = { example?: string | undefined; kind: ContractKindDto; name: string; required: boolean };
  export type SiteContact = { email?: (string | null) | undefined; name: string; phone?: (string | null) | undefined };
  export type ContractSite = {
    address_line?: (string | null) | undefined;
    city?: (string | null) | undefined;
    contact?: (null | SiteContact) | undefined;
    customer_name: string;
    postal_code?: (string | null) | undefined;
    site_label: string;
  };
  export type ContractType = "CDI" | "CDD" | "APPRENTICESHIP" | "INTERIM" | "MANAGER";
  export type CorrectEmployeeCostBasisRequest = {
    effective_from: string;
    effective_to?: (string | null) | undefined;
    hourly_rate_cents?: (number | null) | undefined;
    is_salaried: boolean;
    monthly_cost_cents?: (number | null) | undefined;
    weekly_contract_minutes: number;
  };
  export type CreateAbsenceRequest = {
    all_day?: boolean | undefined;
    end_date?: (string | null) | undefined;
    end_part?: (null | DayPart) | undefined;
    ends_at?: (string | null) | undefined;
    kind?: (null | AbsenceKind) | undefined;
    member_id: MemberId;
    note?: (string | null) | undefined;
    proof_file_key?: (string | null) | undefined;
    start_date?: (string | null) | undefined;
    start_part?: (null | DayPart) | undefined;
    starts_at?: (string | null) | undefined;
    type_id?: (null | AbsenceTypeId) | undefined;
  };
  export type CreateCategoryRequest = {
    allowed_role_ids?: (Array<RoleId> | null) | undefined;
    name: string;
    position: number;
  };
  export type CreateChannelRequest = {
    allowed_role_ids?: (Array<RoleId> | null) | undefined;
    category_id?: (null | CategoryId) | undefined;
    channel_type?: (null | ChannelType) | undefined;
    name: string;
    position: number;
    topic?: (string | null) | undefined;
  };
  export type CredentialOriginRequest = "supplied" | "generated";
  export type CreateCredentialRequest = {
    data?: (Record<string, unknown> | null) | undefined;
    description?: (string | null) | undefined;
    expires_at?: (string | null) | undefined;
    kind: string;
    name: string;
    origin: CredentialOriginRequest;
  };
  export type CreateCustomUnitRequest = { code: string };
  export type CreateCustomerContactRequest = {
    email?: (string | null) | undefined;
    first_name: string;
    is_primary: boolean;
    last_name: string;
    mobile?: (string | null) | undefined;
    phone?: (string | null) | undefined;
    role?: (string | null) | undefined;
  };
  export type CreateCustomerContextRequest = {
    address_line?: (string | null) | undefined;
    address_line2?: (string | null) | undefined;
    city?: (string | null) | undefined;
    country?: string | undefined;
    kind?: AddressKind | undefined;
    label: string;
    photo_key?: (string | null) | undefined;
    postal_code?: (string | null) | undefined;
  };
  export type CustomerContactId = string;
  export type CustomerKind = "INDIVIDUAL" | "PROFESSIONAL";
  export type LeadSource = "bouche_a_oreille" | "site_web" | "recommandation" | "salon" | "architecte" | "autre";
  export type CustomerStatus = "PROSPECT" | "CLIENT" | "ARCHIVED";
  export type CreateCustomerRequest = {
    billing_address_id?: (null | CustomerContextId) | undefined;
    billing_contact_id?: (null | CustomerContactId) | undefined;
    email?: (string | null) | undefined;
    kind?: CustomerKind | undefined;
    lead_source?: (null | LeadSource) | undefined;
    name: string;
    notes?: (string | null) | undefined;
    owner_member_id?: (null | MemberId) | undefined;
    payment_terms_days?: (number | null) | undefined;
    phone?: (string | null) | undefined;
    registration_number?: (string | null) | undefined;
    status: CustomerStatus;
    vat_number?: (string | null) | undefined;
    website?: (string | null) | undefined;
  };
  export type CreateDealRequest = {
    amount_cents?: number | undefined;
    customer_id: CustomerId;
    expected_close_date?: (string | null) | undefined;
    notes?: (string | null) | undefined;
    owner_member_id?: (null | MemberId) | undefined;
    source?: (null | LeadSource) | undefined;
    title: string;
  };
  export type StarterModel = "landscaping";
  export type CreateDoeModelFromStarterRequest = { starter: StarterModel };
  export type MakerLabel = "manufacturer" | "nursery";
  export type PhotosPerPage = "one" | "two" | "four";
  export type DoeModelSectionKind =
    | { kind: "supplies"; maker_label: MakerLabel }
    | { kind: "photos"; per_page: PhotosPerPage }
    | { kind: "plans" }
    | { count: BlankPageCount; kind: "blank_pages" };
  export type DoeModelSection = { enabled: boolean; kind: DoeModelSectionKind; title: string };
  export type DoeFont = "inter" | "montserrat" | "source_serif" | "barlow";
  export type DoeLabel = string;
  export type LogoPosition = "left" | "center" | "right";
  export type LogoSize = "small" | "medium" | "large";
  export type ImageFit = "contain" | "cover";
  export type Opacity = number;
  export type HexColor = string;
  export type Millimeters = number;
  export type DecorationShape =
    | { fit?: ImageFit | undefined; key: string; kind: "image"; opacity?: Opacity | undefined }
    | { fill: HexColor; kind: "rectangle"; opacity?: Opacity | undefined; radius?: Millimeters | undefined };
  export type DecorationLayer = "behind" | "front";
  export type DecorationPages = "cover" | "inner" | "all";
  export type DoeDecoration = DecorationShape & {
    height: Millimeters;
    layer?: DecorationLayer | undefined;
    pages?: DecorationPages | undefined;
    width: Millimeters;
    x: Millimeters;
    y: Millimeters;
  };
  export type DoeTemplate = string;
  export type DoeStyle = Partial<{
    body_font: DoeFont;
    contents_title: DoeLabel;
    cover_logo_position: LogoPosition;
    cover_logo_size: LogoSize;
    cover_photo: boolean;
    cover_title: DoeLabel;
    decorations: Array<DoeDecoration>;
    footer_text: DoeTemplate;
    header_logo: boolean;
    header_text: DoeTemplate;
    heading_font: DoeFont;
    logo_key: string | null;
    on_primary: HexColor;
    parties_title: DoeLabel;
    primary: HexColor;
    secondary: HexColor;
    supply_table_title: DoeLabel;
  }>;
  export type CreateDoeModelRequest = {
    description?: (string | null) | undefined;
    name: string;
    sections: Array<DoeModelSection>;
    style?: (null | DoeStyle) | undefined;
  };
  export type DoeModelId = string;
  export type CreateDoeRequest = { model_id: DoeModelId };
  export type CreateEquipmentRequest = { hourly_rate_cents: number; name: string };
  export type ExpenseCategoryId = string;
  export type SupplierId = string;
  export type CreateExpenseRequest = {
    advanced_by_member_id?: (null | MemberId) | undefined;
    amount_ht_cents: number;
    category_id: ExpenseCategoryId;
    customer_id?: (null | CustomerId) | undefined;
    date: string;
    due_date?: (string | null) | undefined;
    label: string;
    note?: (string | null) | undefined;
    proof_file_key?: (string | null) | undefined;
    reference?: (string | null) | undefined;
    supplier_id?: (null | SupplierId) | undefined;
    vat_cents?: number | undefined;
  };
  export type SurchargeId = string;
  export type CreateExtraPeriodRequest = {
    approve?: boolean | undefined;
    date: string;
    end: string;
    lunch?: (null | LunchInput) | undefined;
    member_id: MemberId;
    rate_bp?: (number | null) | undefined;
    reason: string;
    start: string;
    surcharge_id?: (null | SurchargeId) | undefined;
  };
  export type CreateFolderRequest = { name: string; parent_id?: (string | null) | undefined };
  export type CreateIntakeTicketRequest = {
    body: string;
    custom_fields: Record<string, unknown>;
    external_id?: (string | null) | undefined;
    subject: string;
  };
  export type CreateInvitationRequest = Partial<{ expires_at: string | null; member_id: null | MemberId }>;
  export type QuoteInvoiceKind = "FULL" | "DEPOSIT";
  export type CreateInvoiceFromQuoteRequest = { mode: QuoteInvoiceKind; percent?: (string | null) | undefined };
  export type DeliveryAddressRequest = {
    city: string;
    country: string;
    line1: string;
    line2?: (string | null) | undefined;
    postal_code: string;
  };
  export type InvoiceKind = "STANDARD" | "DEPOSIT" | "FINAL" | "CREDIT_NOTE";
  export type ProductId = string;
  export type ServiceRateId = string;
  export type InvoiceLineRequest = {
    discount_percent?: (string | null) | undefined;
    label: string;
    product_id?: (null | ProductId) | undefined;
    quantity: string;
    service_rate_id?: (null | ServiceRateId) | undefined;
    unit_price_cents: number;
    vat_rate_basis_points?: (number | null) | undefined;
  };
  export type OperationNature = "GOODS" | "SERVICES" | "BOTH";
  export type CreateInvoiceRequest = {
    customer_context_id: CustomerContextId;
    customer_id: CustomerId;
    delivery_address?: (null | DeliveryAddressRequest) | undefined;
    kind: InvoiceKind;
    legal_notice?: (string | null) | undefined;
    lines: Array<InvoiceLineRequest>;
    notes?: (string | null) | undefined;
    operation_nature?: (null | OperationNature) | undefined;
    payment_terms_days?: (number | null) | undefined;
    project_id?: (null | ProjectId) | undefined;
    title?: string | undefined;
  };
  export type LeaveAccountId = string;
  export type CreateLeaveAdjustmentRequest = {
    account_id: LeaveAccountId;
    date: string;
    days: number;
    kind: AdjustmentKind;
    member_id: MemberId;
    reason: string;
  };
  export type CreateMaintenanceContractRequest = ContractBody & {
    customer_context_id: CustomerContextId;
    customer_id: CustomerId;
  };
  export type CreateMemberRequest = { first_name?: (string | null) | undefined; last_name: string };
  export type CreateMessageAttachment = {
    filename: string;
    mime_type: string;
    size_bytes: number;
    storage_key: string;
  };
  export type CreateMessageRequest = { attachments?: Array<CreateMessageAttachment> | undefined; content: string };
  export type CreateOrganizationRequest = { name: string; slug: string };
  export type CreateProductRequest = {
    archived?: (boolean | null) | undefined;
    billing_frequency?: (null | BillingFrequency) | undefined;
    category?: (string | null) | undefined;
    cost_cents?: (number | null) | undefined;
    datasheet_keys?: Array<string> | undefined;
    default_vat_rate_bp?: (number | null) | undefined;
    description?: (string | null) | undefined;
    manufacturer?: (string | null) | undefined;
    name: string;
    photo_keys: Array<string>;
    sku?: (string | null) | undefined;
    supplier_id?: (null | SupplierId) | undefined;
    technical_specs?: (string | null) | undefined;
    unit: string;
    unit_price_cents: number;
  };
  export type CreateProjectChannelRequest = Partial<{ name: string | null }>;
  export type CreateProjectRequest = {
    customer_context_id?: (null | CustomerContextId) | undefined;
    customer_id?: (null | CustomerId) | undefined;
    name: string;
    quote_id?: (null | QuoteId) | undefined;
  };
  export type ProjectTemplateTaskShapeRequest = {
    all_day?: boolean | undefined;
    blocks_availability: boolean;
    day_offset: number;
    description?: (string | null) | undefined;
    ends_minute?: (number | null) | undefined;
    expenses_cents?: number | undefined;
    expenses_label?: (string | null) | undefined;
    parent_index?: (number | null) | undefined;
    starts_minute?: (number | null) | undefined;
    title: string;
  };
  export type CreateProjectTemplateRequest = {
    description?: (string | null) | undefined;
    name: string;
    tasks?: Array<ProjectTemplateTaskShapeRequest> | undefined;
  };
  export type QuoteLineId = string;
  export type PlannedTaskRequest = {
    all_day?: boolean | undefined;
    blocks_availability: boolean;
    description?: (string | null) | undefined;
    ends_at?: (string | null) | undefined;
    expenses_cents?: number | undefined;
    expenses_label?: (string | null) | undefined;
    parent_index?: (number | null) | undefined;
    quote_line_ids?: Array<QuoteLineId> | undefined;
    starts_at?: (string | null) | undefined;
    title: string;
  };
  export type CreateQuotePlanRequest = {
    force_new?: boolean | undefined;
    name: string;
    tasks?: Array<PlannedTaskRequest> | undefined;
  };
  export type DealId = string;
  export type QuoteLineRequest = {
    billing_frequency?: (null | BillingFrequency) | undefined;
    discount_percent?: (string | null) | undefined;
    label: string;
    notes?: (string | null) | undefined;
    photo_keys: Array<string>;
    product_id?: (null | ProductId) | undefined;
    quantity: string;
    service_rate_id?: (null | ServiceRateId) | undefined;
    unit: string;
    unit_price_cents: number;
    vat_rate_bp?: (number | null) | undefined;
  };
  export type CreateQuoteRequest = {
    billing_address_id?: (null | CustomerContextId) | undefined;
    billing_contact_id?: (null | CustomerContactId) | undefined;
    customer_context_id: CustomerContextId;
    customer_id: CustomerId;
    deal_id?: (null | DealId) | undefined;
    deposit_percent?: (string | null) | undefined;
    lines: Array<QuoteLineRequest>;
    notes?: (string | null) | undefined;
    owner_member_id?: (null | MemberId) | undefined;
    title: string;
    valid_until?: (string | null) | undefined;
  };
  export type CreateRoleRequest = { name: string; permissions: Array<string> };
  export type CreateServiceRateRequest = {
    archived?: (boolean | null) | undefined;
    billing_frequency?: (null | BillingFrequency) | undefined;
    category?: (string | null) | undefined;
    cost_cents?: (number | null) | undefined;
    default_vat_rate_bp?: (number | null) | undefined;
    description?: (string | null) | undefined;
    label: string;
    rate_cents: number;
    unit: string;
  };
  export type SupplierAddressRequest = {
    city?: string | undefined;
    country?: string | undefined;
    kind: AddressKind;
    label?: string | undefined;
    line2?: string | undefined;
    postal_code?: string | undefined;
    street?: string | undefined;
  };
  export type SupplierContactRequest = Partial<{
    email: string;
    first_name: string;
    last_name: string;
    mobile: string;
    phone: string;
    role: string;
  }>;
  export type CreateSupplierRequest = {
    address?: (string | null) | undefined;
    addresses?: Array<SupplierAddressRequest> | undefined;
    contacts?: Array<SupplierContactRequest> | undefined;
    default_category_id?: (null | ExpenseCategoryId) | undefined;
    email?: (string | null) | undefined;
    iban?: (string | null) | undefined;
    name: string;
    notes?: (string | null) | undefined;
    payment_terms_days?: (number | null) | undefined;
    phone?: (string | null) | undefined;
    registration_number?: (string | null) | undefined;
    vat_number?: (string | null) | undefined;
  };
  export type CreateSupportApiKeyRequest = { name: string };
  export type CreateTaskCommentRequest = { body: string };
  export type CreateTaskLabelRequest = { color: string; name: string };
  export type RecurrenceRuleRequest =
    | { frequency: "DAILY" }
    | { frequency: "WEEKLY"; weekdays: Array<number> }
    | { day_of_month: number; frequency: "MONTHLY" };
  export type CreateTaskRecurrenceRequest = RecurrenceRuleRequest & {
    all_day?: boolean | undefined;
    assignee_member_ids?: Array<MemberId> | undefined;
    blocks_availability: boolean;
    customer_context_id?: (null | CustomerContextId) | undefined;
    customer_id?: (null | CustomerId) | undefined;
    description?: (string | null) | undefined;
    duration_minutes: number;
    ends_on?: (string | null) | undefined;
    project_id?: (null | ProjectId) | undefined;
    start_time: string;
    starts_on: string;
    timezone: string;
    title: string;
  };
  export type CreateTaskRequest = {
    all_day?: boolean | undefined;
    blocks_availability: boolean;
    customer_context_id?: (null | CustomerContextId) | undefined;
    customer_id?: (null | CustomerId) | undefined;
    description?: (string | null) | undefined;
    ends_at?: (string | null) | undefined;
    expenses_cents?: number | undefined;
    expenses_label?: (string | null) | undefined;
    parent_task_id?: (null | TaskId) | undefined;
    project_id?: (null | ProjectId) | undefined;
    quote_id?: (null | QuoteId) | undefined;
    starts_at?: (string | null) | undefined;
    status?: (null | TaskStatus) | undefined;
    title: string;
  };
  export type CreateThreadRequest = { name: string; origin_message_id?: (null | MessageId) | undefined };
  export type CreateWebhookRequest = { avatar_url?: (string | null) | undefined; name: string };
  export type CreateWorkflowRequest = {
    description?: (string | null) | undefined;
    enabled?: (boolean | null) | undefined;
    folder_id?: (string | null) | undefined;
    name: string;
    tags?: (Array<string> | null) | undefined;
  };
  export type InvitationId = string;
  export type InvitationResponse = {
    created_at: string;
    expires_at: string;
    id: InvitationId;
    member_id?: (null | MemberId) | undefined;
    organization_id: OrganizationId;
  };
  export type CreatedInvitationResponse = InvitationResponse & { token: string };
  export type SupportApiKeyId = string;
  export type CreatedSupportApiKeyResponse = {
    created_at: string;
    id: SupportApiKeyId;
    key: string;
    name: string;
    prefix: string;
  };
  export type CredentialOptionResponse = { label: string; value: string };
  export type CredentialShowWhenResponse = { field: string; one_of: Array<string> };
  export type CredentialFieldResponse = {
    default?: (string | null) | undefined;
    hint?: (string | null) | undefined;
    kind: string;
    label: string;
    name: string;
    options: Array<CredentialOptionResponse>;
    placeholder?: (string | null) | undefined;
    required: boolean;
    secret: boolean;
    show_when?: (null | CredentialShowWhenResponse) | undefined;
  };
  export type CredentialUsageResponse = { has_node: boolean; label: string; usage: string };
  export type CredentialKindResponse = {
    category: string;
    category_label: string;
    connectable: boolean;
    description: string;
    executable: boolean;
    fields: Array<CredentialFieldResponse>;
    kind: string;
    label: string;
    origin_choices: Array<string>;
    supported: boolean;
    usages: Array<CredentialUsageResponse>;
  };
  export type CredentialStatusResponse = "valid" | "expired" | "not_connected";
  export type CredentialResponse = {
    account?: (string | null) | undefined;
    created_at: string;
    data: Record<string, unknown>;
    description: string;
    expires_at?: (string | null) | undefined;
    id: string;
    kind: string;
    name: string;
    organization_id: OrganizationId;
    origin: string;
    secrets_set: Array<string>;
    status: CredentialStatusResponse;
    updated_at: string;
    used_by_workflows: number;
  };
  export type CredentialWithSecretResponse = CredentialResponse & { secret: unknown };
  export type InvoiceLineId = string;
  export type CreditLineBody = { quantity: string; source_line_id: InvoiceLineId };
  export type CustomUnitId = string;
  export type CustomUnitResponse = {
    code: string;
    created_at: string;
    id: CustomUnitId;
    organization_id: OrganizationId;
  };
  export type CustomerContactResponse = {
    created_at: string;
    customer_id: CustomerId;
    email?: (string | null) | undefined;
    first_name: string;
    id: CustomerContactId;
    is_primary: boolean;
    last_name: string;
    mobile?: (string | null) | undefined;
    phone?: (string | null) | undefined;
    role?: (string | null) | undefined;
    updated_at: string;
  };
  export type CustomerContextResponse = {
    address_line?: (string | null) | undefined;
    address_line2?: (string | null) | undefined;
    city?: (string | null) | undefined;
    country: string;
    created_at: string;
    customer_id: CustomerId;
    id: CustomerContextId;
    kind: AddressKind;
    label: string;
    photo_key?: (string | null) | undefined;
    postal_code?: (string | null) | undefined;
    updated_at: string;
  };
  export type CustomerResponse = {
    archived_at?: (string | null) | undefined;
    billing_address_id?: (null | CustomerContextId) | undefined;
    billing_contact_id?: (null | CustomerContactId) | undefined;
    converted_at?: (string | null) | undefined;
    created_at: string;
    email?: (string | null) | undefined;
    id: CustomerId;
    kind: CustomerKind;
    lead_source?: (null | LeadSource) | undefined;
    name: string;
    notes?: (string | null) | undefined;
    organization_id: OrganizationId;
    owner_member_id?: (null | MemberId) | undefined;
    payment_terms_days?: (number | null) | undefined;
    phone?: (string | null) | undefined;
    registration_number?: (string | null) | undefined;
    status: CustomerStatus;
    updated_at: string;
    vat_number?: (string | null) | undefined;
    website?: (string | null) | undefined;
  };
  export type PrimaryContactResponse = {
    email?: (string | null) | undefined;
    first_name: string;
    last_name: string;
    mobile?: (string | null) | undefined;
    phone?: (string | null) | undefined;
    role?: (string | null) | undefined;
  };
  export type CustomerListItemResponse = CustomerResponse & {
    city?: (string | null) | undefined;
    last_activity_at?: (string | null) | undefined;
    overdue_cents: number;
    primary_contact?: (null | PrimaryContactResponse) | undefined;
    revenue_12m_cents: number;
  };
  export type CustomerOutstandingBalanceResponse = {
    customer_id: CustomerId;
    oldest_due_at?: (string | null) | undefined;
    outstanding_cents: number;
  };
  export type DayLogId = string;
  export type EmployeeId = string;
  export type DayLogResponse = {
    employee_id: EmployeeId;
    ended_at: string;
    id: DayLogId;
    organization_id: OrganizationId;
    work_date: string;
  };
  export type DayOverrideId = string;
  export type OverrideKind = "WORK" | "OFF";
  export type DayOverrideResponse = {
    date: string;
    end?: (string | null) | undefined;
    id: DayOverrideId;
    kind: OverrideKind;
    lunch?: (null | LunchInput) | undefined;
    member_id: MemberId;
    note: string;
    organization_id: OrganizationId;
    start?: (string | null) | undefined;
  };
  export type DealStageName = "NEW" | "VISIT" | "QUOTE_SENT" | "NEGOTIATION" | "WON" | "LOST";
  export type DealResponse = {
    amount_cents: number;
    closed_at?: (string | null) | undefined;
    created_at: string;
    customer_id: CustomerId;
    expected_close_date?: (string | null) | undefined;
    id: DealId;
    lost_reason?: (string | null) | undefined;
    notes?: (string | null) | undefined;
    organization_id: OrganizationId;
    owner_member_id?: (null | MemberId) | undefined;
    quote_count: number;
    source?: (null | LeadSource) | undefined;
    stage: DealStageName;
    stage_changed_at: string;
    title: string;
  };
  export type StageTotalResponse = { amount_cents: number; count: number; stage: DealStageName };
  export type DealSummaryResponse = {
    conversions: number;
    lost_count: number;
    open_amount_cents: number;
    open_count: number;
    stages: Array<StageTotalResponse>;
    win_rate?: (number | null) | undefined;
    won_amount_cents: number;
    won_count: number;
  };
  export type DeclareTimeEntryRequest = { ended_at: string; started_at: string; task_id: TaskId };
  export type DeleteScopeRequest = "THIS_OCCURRENCE" | "THIS_AND_FOLLOWING";
  export type DeliveryAddressResponse = {
    city: string;
    country: string;
    line1: string;
    line2?: (string | null) | undefined;
    postal_code: string;
  };
  export type UserId = string;
  export type DirectChannelResponse = {
    channel_id: ChannelId;
    created_at: string;
    organization_id: OrganizationId;
    peer_user_id: UserId;
  };
  export type PresenceStatus = "ONLINE" | "OFFLINE" | "DND";
  export type DirectoryEntryResponse = { display_name: string; presence: PresenceStatus; user_id: UserId };
  export type DoeFile = { caption?: (string | null) | undefined; key: string };
  export type DoeId = string;
  export type DoeModelResponse = {
    archived_at?: (string | null) | undefined;
    created_at: string;
    description?: (string | null) | undefined;
    id: DoeModelId;
    name: string;
    organization_id: OrganizationId;
    sections: Array<DoeModelSection>;
    style?: (null | DoeStyle) | undefined;
    updated_at: string;
  };
  export type DoeModelStarterResponse = {
    description?: (string | null) | undefined;
    name: string;
    sections: Array<DoeModelSection>;
    starter: StarterModel;
  };
  export type PartyRole = "project_owner" | "project_manager" | "contractor";
  export type DoeParty = {
    address?: (string | null) | undefined;
    email?: (string | null) | undefined;
    logo_key?: (string | null) | undefined;
    name: string;
    phone?: (string | null) | undefined;
    role: PartyRole;
  };
  export type Revision = string;
  export type SupplierSnapshot = {
    address?: (string | null) | undefined;
    email?: (string | null) | undefined;
    name: string;
    phone?: (string | null) | undefined;
  };
  export type DoeSupply = {
    datasheet_keys: Array<string>;
    designation: string;
    destination?: (string | null) | undefined;
    dpgf_ref?: (string | null) | undefined;
    id: string;
    maker?: (string | null) | undefined;
    photos: Array<DoeFile>;
    source_product_id?: (null | ProductId) | undefined;
    specs?: (string | null) | undefined;
    supplier?: (null | SupplierSnapshot) | undefined;
    title?: (string | null) | undefined;
  };
  export type DoeSectionContent =
    | { kind: "supplies"; maker_label: MakerLabel; supplies: Array<DoeSupply> }
    | { kind: "photos"; per_page: PhotosPerPage; photos: Array<DoeFile> }
    | { kind: "plans"; plans: Array<DoeFile> }
    | { count: BlankPageCount; kind: "blank_pages" };
  export type DoeSection = { content: DoeSectionContent; enabled: boolean; id: string; title: string };
  export type DoeResponse = {
    author?: (string | null) | undefined;
    cover_photo_key?: (string | null) | undefined;
    created_at: string;
    id: DoeId;
    issued_on: string;
    lot?: (string | null) | undefined;
    model_id?: (null | DoeModelId) | undefined;
    parties: Array<DoeParty>;
    place?: (string | null) | undefined;
    project_id: ProjectId;
    revision: Revision;
    sections: Array<DoeSection>;
    title: string;
    updated_at: string;
  };
  export type DoeSettingsResponse = { style: DoeStyle };
  export type DoeStylePreviewPagesResponse = { pages: Array<string> };
  export type DoeSummaryResponse = {
    id: DoeId;
    issued_on: string;
    project_id: ProjectId;
    project_name: string;
    revision: Revision;
    title: string;
    updated_at: string;
  };
  export type DraftSupplyRequest = { product_id: ProductId };
  export type OperationRefDto = { collection_id: string; operation_id: string };
  export type PlacedConnectorDto = {
    config: Record<string, unknown>;
    credential_id?: (string | null) | undefined;
    description?: (string | null) | undefined;
    disabled?: boolean | undefined;
    id: string;
    kind: string;
    name?: (string | null) | undefined;
    operation?: (null | OperationRefDto) | undefined;
    version: number;
  };
  export type EdgeDto = { branch?: (null | BranchDto) | undefined; from: string; to: string };
  export type WebhookMethodDto = "GET" | "POST" | "PUT" | "PATCH" | "DELETE";
  export type OnInvalidDto = "Reject" | "Accept";
  export type FrequencyDto = "Hourly" | "Daily" | "Weekly";
  export type WeekdayDto = "Monday" | "Tuesday" | "Wednesday" | "Thursday" | "Friday" | "Saturday" | "Sunday";
  export type TriggerKindDto =
    | "Manual"
    | { Events: Array<string> }
    | {
        Webhook: {
          contract?: Array<ContractKeyDto> | undefined;
          credential_id?: (string | null) | undefined;
          method: WebhookMethodDto;
          on_invalid?: OnInvalidDto | undefined;
          path: string;
        };
      }
    | {
        Schedule: {
          frequency: FrequencyDto;
          time: string;
          timezone?: (string | null) | undefined;
          weekday?: (null | WeekdayDto) | undefined;
        };
      };
  export type PlacedTriggerDto = {
    description?: (string | null) | undefined;
    id: string;
    kind: TriggerKindDto;
    name?: (string | null) | undefined;
  };
  export type GraphDto = {
    connectors: Array<PlacedConnectorDto>;
    edges: Array<EdgeDto>;
    triggers: Array<PlacedTriggerDto>;
  };
  export type DryRunRequest = {
    graph: GraphDto;
    trigger_id?: (string | null) | undefined;
    trigger_payload?: (Record<string, unknown> | null) | undefined;
  };
  export type DryRunStepStatus = "succeeded" | "failed" | "skipped";
  export type DryRunStepResponse = {
    connector_id: string;
    error?: (string | null) | undefined;
    input?: unknown | undefined;
    iteration_path: string;
    output?: unknown | undefined;
    status: DryRunStepStatus;
  };
  export type DryRunResponse = { steps: Array<DryRunStepResponse> };
  export type DueStatus = "PLANNED" | "OVERDUE" | "THIS_WEEK" | "NEXT_WEEK" | "LATER" | "DONE";
  export type Due = { on?: (string | null) | undefined; status: DueStatus };
  export type EditSupportDraftRequest = { body: string };
  export type EmployeeCostBasisId = string;
  export type EmployeeCostBasisResponse = {
    created_at: string;
    effective_from: string;
    effective_hourly_rate_cents?: (number | null) | undefined;
    effective_to?: (string | null) | undefined;
    employee_id: EmployeeId;
    hourly_rate_cents?: (number | null) | undefined;
    id: EmployeeCostBasisId;
    is_salaried: boolean;
    monthly_cost_cents?: (number | null) | undefined;
    organization_id: OrganizationId;
    updated_at: string;
    weekly_contract_minutes: number;
  };
  export type EmployeeResponse = {
    created_at: string;
    effective_hourly_rate_cents?: (number | null) | undefined;
    hourly_rate_cents?: (number | null) | undefined;
    id: EmployeeId;
    is_salaried: boolean;
    member_id: MemberId;
    monthly_cost_cents?: (number | null) | undefined;
    organization_id: OrganizationId;
    updated_at: string;
    weekly_contract_minutes: number;
  };
  export type EmployeeRhythmId = string;
  export type EndDayRequest = Partial<{ ended_at: string | null }>;
  export type EntryKind = "client_message" | "reply" | "internal_note" | "draft";
  export type EquipmentResponse = {
    created_at: string;
    hourly_rate_cents: number;
    id: EquipmentId;
    name: string;
    organization_id: OrganizationId;
    updated_at: string;
  };
  export type LoopFrameBody = { index: number; item: unknown };
  export type EvaluateContextBody = Partial<{
    loop: null | LoopFrameBody;
    nodes: Record<string, Array<unknown>>;
    position: number;
    trigger: unknown;
  }>;
  export type EvaluateExpressionRequest = { context: EvaluateContextBody; template: unknown };
  export type EvaluateExpressionResponse = { value: unknown };
  export type EventDescriptorResponse = {
    label: string;
    name: string;
    payload_example: unknown;
    subject_kind: string;
    version: number;
  };
  export type ExecuteWebhookRequest = { components?: (Array<Component> | null) | undefined; content: string };
  export type ExpenseCategoryRequest = { hue?: number | undefined; name: string };
  export type ExpenseCategoryResponse = {
    created_at: string;
    hue: number;
    id: ExpenseCategoryId;
    name: string;
    organization_id: OrganizationId;
    updated_at: string;
  };
  export type ExpenseDisplayState = "TO_PAY" | "OVERDUE" | "PAID" | "TO_REIMBURSE" | "REIMBURSED";
  export type ExpenseId = string;
  export type PaymentMethod = "CARD" | "TRANSFER" | "CHECK" | "CASH" | "DIRECT_DEBIT";
  export type ExpenseStatus = "TO_PAY" | "PAID";
  export type ExpenseResponse = {
    advanced_by_member_id?: (null | MemberId) | undefined;
    amount_ht_cents: number;
    category_id: ExpenseCategoryId;
    created_at: string;
    customer_id?: (null | CustomerId) | undefined;
    customer_name?: (string | null) | undefined;
    date: string;
    days_late: number;
    display_state: ExpenseDisplayState;
    due_date?: (string | null) | undefined;
    id: ExpenseId;
    label: string;
    method?: (null | PaymentMethod) | undefined;
    note?: (string | null) | undefined;
    organization_id: OrganizationId;
    paid_on?: (string | null) | undefined;
    proof_file_key?: (string | null) | undefined;
    reference?: (string | null) | undefined;
    reimbursed_on?: (string | null) | undefined;
    status: ExpenseStatus;
    supplier_id?: (null | SupplierId) | undefined;
    supplier_name?: (string | null) | undefined;
    total_cents: number;
    updated_at: string;
    vat_cents: number;
  };
  export type ExtraPeriodId = string;
  export type ExtraStatus = "PENDING" | "APPROVED" | "REJECTED" | "CANCELLED";
  export type ExtraPeriodResponse = {
    created_at: string;
    created_by_member_id?: (null | MemberId) | undefined;
    date: string;
    decided_at?: (string | null) | undefined;
    decided_by_member_id?: (null | MemberId) | undefined;
    decision_comment?: (string | null) | undefined;
    end: string;
    id: ExtraPeriodId;
    lunch?: (null | LunchInput) | undefined;
    member_id: MemberId;
    organization_id: OrganizationId;
    rate_bp: number;
    reason: string;
    start: string;
    status: ExtraStatus;
    surcharge_id?: (null | SurchargeId) | undefined;
  };
  export type FeedMemberResponse = {
    archived: boolean;
    display_name: string;
    first_name?: (string | null) | undefined;
    id: MemberId;
    last_name: string;
  };
  export type InterventionId = string;
  export type InterventionStatus = "PROPOSED" | "VALIDATED" | "REJECTED";
  export type FieldInterventionResponse = {
    done_on: string;
    id: InterventionId;
    line_id: MaintenanceLineId;
    status: InterventionStatus;
    task_id?: (null | TaskId) | undefined;
  };
  export type FieldTaskResponse = {
    all_day: boolean;
    customer_context_id?: (null | CustomerContextId) | undefined;
    customer_id?: (null | CustomerId) | undefined;
    description?: (string | null) | undefined;
    ends_at?: (string | null) | undefined;
    id: TaskId;
    maintenance_line_id?: (null | MaintenanceLineId) | undefined;
    starts_at?: (string | null) | undefined;
    status: TaskStatus;
    task_assignment_id: TaskAssignmentId;
    title: string;
  };
  export type FileUploadResponse = { key: string; mime_type: string; size_bytes: number };
  export type FileUrlResponse = { expires_at: string; url: string };
  export type FolderResponse = {
    created_at: string;
    id: string;
    name: string;
    parent_id?: (string | null) | undefined;
  };
  export type GraphErrorResponse = {
    connector_id?: (string | null) | undefined;
    field?: (string | null) | undefined;
    message: string;
    trigger_id?: (string | null) | undefined;
  };
  export type GraphInvalidDetails = { errors: Array<GraphErrorResponse> };
  export type GraphInvalidBody = { code: string; details: GraphInvalidDetails; message: string; status: number };
  export type MaintenanceContractId = string;
  export type GridVisit = { done_on: string; id: InterventionId; status: InterventionStatus; week: IsoWeekNumber };
  export type GridLine = {
    cadence: Cadence;
    due: Due;
    id: MaintenanceLineId;
    label: string;
    minutes_per_intervention?: (number | null) | undefined;
    visits: Array<GridVisit>;
  };
  export type Season = { first_week: IsoWeekNumber; last_week: IsoWeekNumber; year: number };
  export type GridContract = {
    customer_context_id: CustomerContextId;
    customer_id: CustomerId;
    id: MaintenanceContractId;
    lines: Array<GridLine>;
    manager_member_id?: (null | MemberId) | undefined;
    reference?: (string | null) | undefined;
    season: Season;
    site: ContractSite;
    status: ContractStatus;
  };
  export type Grid = { contracts: Array<GridContract>; year: number };
  export type MemberContractId = string;
  export type ScheduleInput =
    | { kind: "weekly"; week: Array<null | WorkDayInput> }
    | { anchor: string; blocks: Array<BlockInput>; kind: "rotation" }
    | { default_day: WorkDayInput; kind: "variable" };
  export type MemberContractResponse = {
    end_date?: (string | null) | undefined;
    id: MemberContractId;
    member_id: MemberId;
    organization_id: OrganizationId;
    schedule: ScheduleInput;
    start_date: string;
    trial_end_date?: (string | null) | undefined;
    type: ContractType;
  };
  export type HrPlanningFeedResponse = {
    absence_types: Array<AbsenceTypeResponse>;
    absences: Array<AbsenceResponse>;
    contracts: Array<MemberContractResponse>;
    members: Array<FeedMemberResponse>;
    overrides: Array<DayOverrideResponse>;
  };
  export type SupplierInvoiceId = string;
  export type SupplierInvoiceLineAllocationId = string;
  export type SupplierInvoiceLineId = string;
  export type SupplierInvoiceLineAllocationResponse = {
    amount_cents: number;
    created_at: string;
    id: SupplierInvoiceLineAllocationId;
    organization_id: OrganizationId;
    project_id: ProjectId;
    supplier_invoice_line_id: SupplierInvoiceLineId;
    updated_at: string;
  };
  export type SupplierInvoiceLineResponse = {
    allocations: Array<SupplierInvoiceLineAllocationResponse>;
    created_at: string;
    id: SupplierInvoiceLineId;
    label: string;
    line_total_cents: number;
    position: number;
    quantity: string;
    supplier_invoice_id: SupplierInvoiceId;
    unit?: (string | null) | undefined;
    unit_price_cents: number;
    updated_at: string;
    vat_rate_basis_points?: (number | null) | undefined;
  };
  export type SupplierInvoiceSource = "MANUAL" | "FACTUR_X";
  export type SupplierInvoiceStatus = "RECEIVED" | "CONFIRMED" | "REJECTED";
  export type SupplierInvoiceVatBreakdownLineResponse = { rate_bp: number; vat_cents: number };
  export type SupplierInvoiceResponse = {
    created_at: string;
    currency: string;
    due_on?: (string | null) | undefined;
    gross_cents: number;
    id: SupplierInvoiceId;
    issued_on: string;
    lines: Array<SupplierInvoiceLineResponse>;
    net_cents: number;
    notes?: (string | null) | undefined;
    number: string;
    organization_id: OrganizationId;
    received_at: string;
    source: SupplierInvoiceSource;
    source_file_key?: (string | null) | undefined;
    source_file_mime_type?: (string | null) | undefined;
    status: SupplierInvoiceStatus;
    supplier_id?: (null | SupplierId) | undefined;
    supplier_name: string;
    supplier_registration_number?: (string | null) | undefined;
    supplier_vat_number?: (string | null) | undefined;
    updated_at: string;
    vat_breakdown: Array<SupplierInvoiceVatBreakdownLineResponse>;
  };
  export type TotalsMismatchResponse = {
    recomputed_gross_cents: number;
    recomputed_net_cents: number;
    stated_gross_cents: number;
    stated_net_cents: number;
  };
  export type ImportSupplierInvoiceResponse =
    | {
        invoice: SupplierInvoiceResponse;
        outcome: "created";
        totals_mismatch?: (null | TotalsMismatchResponse) | undefined;
      }
    | { outcome: "parse_failed"; reason: string };
  export type InstantiateProjectTemplateRequest = {
    customer_context_id?: (null | CustomerContextId) | undefined;
    customer_id?: (null | CustomerId) | undefined;
    name: string;
    quote_id?: (null | QuoteId) | undefined;
    start_date: string;
  };
  export type ProjectResponse = {
    archived_at?: (string | null) | undefined;
    created_at: string;
    customer_context_id?: (null | CustomerContextId) | undefined;
    customer_id?: (null | CustomerId) | undefined;
    id: ProjectId;
    is_internal: boolean;
    name: string;
    organization_id: OrganizationId;
    quote_id?: (null | QuoteId) | undefined;
    updated_at: string;
  };
  export type InstantiateProjectTemplateResponse = { project: ProjectResponse; tasks: Array<TaskResponse> };
  export type IntakeAttachmentResponse = {
    created_at: string;
    file_name: string;
    id: AttachmentId;
    mime_type: string;
    size_bytes: number;
  };
  export type IntakeAttachmentUrlResponse = { expires_at: string; url: string };
  export type IntakeEntryKind = "client_message" | "reply";
  export type TicketEntryId = string;
  export type IntakeEntryResponse = {
    attachments: Array<IntakeAttachmentResponse>;
    body: string;
    created_at: string;
    id: TicketEntryId;
    kind: IntakeEntryKind;
  };
  export type TicketId = string;
  export type TicketNumber = number;
  export type TicketStatus = "new" | "in_progress" | "waiting_customer" | "resolved";
  export type IntakeTicketResponse = {
    badges: Array<BadgeResponse>;
    body: string;
    created_at: string;
    custom_fields: Record<string, unknown>;
    entries?: (Array<IntakeEntryResponse> | null) | undefined;
    external_id?: (string | null) | undefined;
    id: TicketId;
    number: TicketNumber;
    status: TicketStatus;
    subject: string;
  };
  export type InterventionResponse = {
    created_at: string;
    done_on: string;
    id: InterventionId;
    line_id: MaintenanceLineId;
    note?: (string | null) | undefined;
    proposed_by: MemberId;
    resolved_at?: (string | null) | undefined;
    resolved_by?: (null | MemberId) | undefined;
    status: InterventionStatus;
    task_id?: (null | TaskId) | undefined;
  };
  export type InvoiceBalanceResponse = {
    credited_cents: number;
    gross_cents: number;
    paid_cents: number;
    remaining_cents: number;
  };
  export type InvoiceDisplayState = "DRAFT" | "UNPAID" | "PARTIAL" | "OVERDUE" | "PAID" | "CANCELLED" | "REFUND_DUE";
  export type InvoiceDocumentFormatQuery = "pdf" | "facturx";
  export type InvoiceEventId = string;
  export type InvoiceEventKind = "SENT" | "REMINDER" | "NOTE";
  export type InvoiceEventResponse = {
    actor?: (null | UserId) | undefined;
    at: string;
    detail: string;
    id: InvoiceEventId;
    kind: InvoiceEventKind;
  };
  export type InvoiceLineResponse = {
    created_at: string;
    discount_percent: string;
    id: InvoiceLineId;
    invoice_id: InvoiceId;
    label: string;
    organization_id: OrganizationId;
    position: number;
    product_id?: (null | ProductId) | undefined;
    quantity: string;
    service_rate_id?: (null | ServiceRateId) | undefined;
    source_line_id?: (null | InvoiceLineId) | undefined;
    unit_price_cents: number;
    updated_at: string;
    vat_rate_basis_points?: (number | null) | undefined;
  };
  export type InvoicePaymentId = string;
  export type InvoicePaymentResponse = {
    amount_cents: number;
    created_at: string;
    deleted_at?: (string | null) | undefined;
    deleted_by?: (null | UserId) | undefined;
    id: InvoicePaymentId;
    invoice_id: InvoiceId;
    method: PaymentMethod;
    note?: (string | null) | undefined;
    organization_id: OrganizationId;
    paid_on: string;
    recorded_by: UserId;
    reference?: (string | null) | undefined;
    updated_at: string;
  };
  export type InvoiceStatus = "DRAFT" | "ISSUED" | "PAID" | "PARTIALLY_PAID" | "CANCELLED";
  export type InvoiceVatBreakdownLineResponse = { rate_bp: number; vat_cents: number };
  export type InvoiceResponse = {
    balance_cents: number;
    created_at: string;
    credited_cents: number;
    customer_context_id: CustomerContextId;
    customer_id: CustomerId;
    delivery_address?: (null | DeliveryAddressResponse) | undefined;
    display_state: InvoiceDisplayState;
    due_at?: (string | null) | undefined;
    due_on?: (string | null) | undefined;
    events: Array<InvoiceEventResponse>;
    full_cancellation: boolean;
    gross_cents: number;
    id: InvoiceId;
    internal_note?: (string | null) | undefined;
    issued_at?: (string | null) | undefined;
    issued_on?: (string | null) | undefined;
    kind: InvoiceKind;
    legal_notice: string;
    lines: Array<InvoiceLineResponse>;
    net_cents: number;
    notes?: (string | null) | undefined;
    number?: (string | null) | undefined;
    operation_nature?: (null | OperationNature) | undefined;
    organization_id: OrganizationId;
    paid_cents: number;
    payment_terms_days: number;
    project_id?: (null | ProjectId) | undefined;
    quote_id?: (null | QuoteId) | undefined;
    reason?: (string | null) | undefined;
    sent_at?: (string | null) | undefined;
    source_invoice_id?: (null | InvoiceId) | undefined;
    status: InvoiceStatus;
    title: string;
    updated_at: string;
    vat_breakdown: Array<InvoiceVatBreakdownLineResponse>;
  };
  export type IssueCreditNoteRequest = {
    full?: boolean | undefined;
    lines?: Array<CreditLineBody> | undefined;
    notes?: (string | null) | undefined;
    reason: string;
  };
  export type IssueDepositRequest = {
    allow_exceeding_total?: boolean | undefined;
    notes?: (string | null) | undefined;
    payment_terms_days?: (number | null) | undefined;
    percentage_bp: number;
  };
  export type IssueFinalInvoiceRequest = Partial<{
    allow_exceeding_total: boolean;
    notes: string | null;
    payment_terms_days: number | null;
  }>;
  export type IssueInvoiceRequest = Partial<{ allow_exceeding_total: boolean; issued_on: string | null }>;
  export type SnapshotAddress = Partial<{
    city: string | null;
    country: string | null;
    line1: string | null;
    line2: string | null;
    postal_code: string | null;
  }>;
  export type IssuerSnapshot = {
    address: SnapshotAddress;
    bic?: (string | null) | undefined;
    contact_email?: (string | null) | undefined;
    contact_phone?: (string | null) | undefined;
    iban?: (string | null) | undefined;
    insurance_mention?: (string | null) | undefined;
    invoice_footer: string;
    late_penalty: string;
    legal_form?: (string | null) | undefined;
    name: string;
    recovery_fee_cents: number;
    registration_number?: (string | null) | undefined;
    share_capital_cents?: (number | null) | undefined;
    vat_exemption_basis?: (string | null) | undefined;
    vat_number?: (string | null) | undefined;
  };
  export type LeaveAccountRequest = {
    acquisition: AcquisitionBody;
    carry_over_max: number;
    name: string;
    period_start_month: number;
  };
  export type LeaveAccountResponse = {
    acquisition: AcquisitionBody;
    archived_at?: (string | null) | undefined;
    carry_over_max: number;
    created_at: string;
    id: LeaveAccountId;
    name: string;
    organization_id: OrganizationId;
    period_start_month: number;
  };
  export type LeaveAdjustmentId = string;
  export type LeaveAdjustmentResponse = {
    account_id: LeaveAccountId;
    author_member_id?: (null | MemberId) | undefined;
    created_at: string;
    date: string;
    days: number;
    id: LeaveAdjustmentId;
    kind: AdjustmentKind;
    member_id: MemberId;
    organization_id: OrganizationId;
    reason: string;
  };
  export type LedgerKind = "RECEIPT" | "EXPENSE";
  export type LedgerStatus = "DONE" | "PENDING" | "LATE" | "UNALLOCATED" | "TO_REIMBURSE";
  export type LedgerEntryResponse = {
    amount_cents: number;
    category_id?: (null | ExpenseCategoryId) | undefined;
    customer_id?: (null | CustomerId) | undefined;
    date: string;
    invoice_id?: (null | InvoiceId) | undefined;
    invoice_number?: (string | null) | undefined;
    key: string;
    kind: LedgerKind;
    label: string;
    method?: (null | PaymentMethod) | undefined;
    party: string;
    reference: string;
    source_id: string;
    status: LedgerStatus;
    supplier_id?: (null | SupplierId) | undefined;
  };
  export type PayablesResponse = { overdue_cents: number; total_cents: number };
  export type LedgerSummaryResponse = {
    expenses_cents: number;
    net_cents: number;
    pending_cents: number;
    receipts_cents: number;
    unallocated_cents: number;
  };
  export type MonthTrendResponse = { expenses_cents: number; month: string; receipts_cents: number };
  export type VatSummaryResponse = { collected_cents: number; deductible_cents: number; due_cents: number };
  export type LedgerResponse = {
    entries: Array<LedgerEntryResponse>;
    months: Array<string>;
    payables: PayablesResponse;
    summary: LedgerSummaryResponse;
    trend: Array<MonthTrendResponse>;
    vat: VatSummaryResponse;
  };
  export type LineAllocationShareRequest = { amount_cents: number; project_id: ProjectId };
  export type MaintenanceLine = {
    cadence: Cadence;
    id: MaintenanceLineId;
    label: string;
    minutes_per_intervention?: (number | null) | undefined;
  };
  export type MaintenanceContractResponse = {
    amount_cents?: (number | null) | undefined;
    archived_at?: (string | null) | undefined;
    created_at: string;
    customer_context_id: CustomerContextId;
    customer_id: CustomerId;
    id: MaintenanceContractId;
    lines: Array<MaintenanceLine>;
    manager_member_id?: (null | MemberId) | undefined;
    notes?: (string | null) | undefined;
    project_id: ProjectId;
    quote_id?: (null | QuoteId) | undefined;
    reference?: (string | null) | undefined;
    season: Season;
    status: ContractStatus;
    updated_at: string;
  };
  export type MarkChannelReadRequest = { message_id: MessageId };
  export type MemberAccountResponse = { email: string; name: string };
  export type MemberDocumentId = string;
  export type MemberDocumentResponse = {
    added_at: string;
    file_key: string;
    id: MemberDocumentId;
    kind: DocumentKind;
    member_id: MemberId;
    name: string;
    organization_id: OrganizationId;
  };
  export type MemberProfileAddressResponse = {
    city: string;
    complement: string;
    country: string;
    postal_code?: (string | null) | undefined;
    street: string;
  };
  export type MemberProfileResponse = {
    address: MemberProfileAddressResponse;
    archived_at?: (string | null) | undefined;
    job_title?: (string | null) | undefined;
    landline?: (string | null) | undefined;
    manager_member_id?: (null | MemberId) | undefined;
    member_id: MemberId;
    mobile?: (string | null) | undefined;
    organization_id: OrganizationId;
    personal_email?: (string | null) | undefined;
    work_email?: (string | null) | undefined;
    work_permit_end_date?: (string | null) | undefined;
  };
  export type MissingCost = "HOURLY_RATE" | "MONTHLY_COST" | "CONTRACTED_HOURS" | "NO_COST_BASIS";
  export type MemberProfitabilityResponse = {
    labour_cost_cents?: (number | null) | undefined;
    member_id: MemberId;
    missing_cost?: (null | MissingCost) | undefined;
    planned_minutes: number;
  };
  export type MemberResponse = {
    account?: (null | MemberAccountResponse) | undefined;
    created_at: string;
    display_name: string;
    first_name?: (string | null) | undefined;
    id: MemberId;
    joined_at?: (string | null) | undefined;
    last_name: string;
    organization_id: OrganizationId;
  };
  export type MemberRoleIdsResponse = { role_ids: Array<RoleId> };
  export type WebhookId = string;
  export type ReactionCountResponse = { count: number; emoji: string; user_ids: Array<UserId> };
  export type MessageResponse = {
    attachments: Array<AttachmentResponse>;
    author_type: AuthorType;
    author_user_id?: (null | UserId) | undefined;
    author_webhook_id?: (null | WebhookId) | undefined;
    channel_id: ChannelId;
    components?: (Array<Component> | null) | undefined;
    content: string;
    created_at: string;
    edited_at?: (string | null) | undefined;
    id: MessageId;
    mention_channel_ids: Array<ChannelId>;
    mention_everyone: boolean;
    mention_role_ids: Array<RoleId>;
    mention_user_ids: Array<UserId>;
    organization_id: OrganizationId;
    reactions: Array<ReactionCountResponse>;
  };
  export type MinuteIntervalResponse = { ends_minute: number; starts_minute: number };
  export type MoveDealRequest = { lost_reason?: (string | null) | undefined; stage: DealStageName };
  export type MyMemberResponse = { display_name: string; member_id: MemberId; organization_id: OrganizationId };
  export type MyPermissionsResponse = { permissions: Array<string> };
  export type NodePositionDto = { x: number; y: number };
  export type NoteInvoiceRequest = { note: string };
  export type NotificationId = string;
  export type NotificationResponse = {
    channel_id: ChannelId;
    created_at: string;
    id: NotificationId;
    kind: string;
    message_id: MessageId;
    read_at?: (string | null) | undefined;
  };
  export type NumberSeriesKind = "QUOTE" | "INVOICE" | "CREDIT_NOTE";
  export type OpenDirectChannelRequest = { user_id: UserId };
  export type str = string;
  export type VatStatusResponse = { type: "subject"; vat_number: string } | { basis: string; type: "not_subject" };
  export type OrganizationResponse = {
    address_city?: (string | null) | undefined;
    address_country?: (string | null) | undefined;
    address_line1?: (string | null) | undefined;
    address_line2?: (string | null) | undefined;
    address_postal_code?: (string | null) | undefined;
    contact_email?: (string | null) | undefined;
    contact_phone?: (string | null) | undefined;
    created_at: string;
    field_clock_enabled: boolean;
    id: OrganizationId;
    insurance_mention?: (string | null) | undefined;
    legal_form?: (string | null) | undefined;
    legal_name?: (string | null) | undefined;
    mcp_enabled: boolean;
    missing_legal_identity_fields: Array<str>;
    name: string;
    owner_id: UserId;
    registration_number?: (string | null) | undefined;
    share_capital_cents?: (number | null) | undefined;
    slug: string;
    updated_at: string;
    vat_on_debits: boolean;
    vat_status?: (null | VatStatusResponse) | undefined;
  };
  export type OverwriteResponse = {
    allow: number;
    created_at: string;
    deny: number;
    target_id?: (string | null) | undefined;
    target_type: string;
    updated_at: string;
  };
  export type PaginationMetadata = {
    current_page: number;
    first_page: number;
    is_empty: boolean;
    last_page?: (number | null) | undefined;
    next_page?: (number | null) | undefined;
    per_page: number;
    prev_page?: (number | null) | undefined;
    total?: (number | null) | undefined;
  };
  export type PatchTaskResponse = { detached: boolean; task: TaskResponse };
  export type PayExpenseRequest = { method: PaymentMethod; paid_on: string };
  export type PlanMaintenanceLinesRequest = { line_ids: Array<MaintenanceLineId>; week: string };
  export type PlannedProjectResponse = {
    created_at: string;
    customer_context_id?: (null | CustomerContextId) | undefined;
    customer_id?: (null | CustomerId) | undefined;
    id: ProjectId;
    name: string;
    organization_id: OrganizationId;
    quote_id?: (null | QuoteId) | undefined;
    updated_at: string;
  };
  export type PlannedTaskResponse = {
    all_day: boolean;
    description?: (string | null) | undefined;
    ends_at?: (string | null) | undefined;
    expenses_cents: number;
    expenses_label?: (string | null) | undefined;
    id: TaskId;
    parent_task_id?: (null | TaskId) | undefined;
    project_id?: (null | ProjectId) | undefined;
    starts_at?: (string | null) | undefined;
    title: string;
  };
  export type PlanningEntryResponse =
    | {
        all_day: boolean;
        blocks_availability: boolean;
        child_count: number;
        context_label?: (string | null) | undefined;
        customer_name?: (string | null) | undefined;
        description?: (string | null) | undefined;
        ends_at: string;
        id: TaskId;
        kind: "task";
        labels: Array<TaskLabelResponse>;
        member_ids: Array<MemberId>;
        parent_task_id?: (null | TaskId) | undefined;
        recurrence_id?: (null | TaskRecurrenceId) | undefined;
        starts_at: string;
        status: TaskStatus;
        title: string;
      }
    | {
        absence_kind: AbsenceKind;
        all_day: boolean;
        ends_at: string;
        id: AbsenceId;
        kind: "absence";
        member_id: MemberId;
        note?: (string | null) | undefined;
        starts_at: string;
      };
  export type PlanningResourceResponse = {
    display_name: string;
    employee_id?: (null | EmployeeId) | undefined;
    hourly_rate_cents?: (number | null) | undefined;
    member_id: MemberId;
    resource_id: string;
    weekly_contract_minutes: number;
  };
  export type PlanningWorkTimeDayResponse = { date: string; intervals: Array<MinuteIntervalResponse> };
  export type PlanningWorkTimeResponse = { days: Array<PlanningWorkTimeDayResponse>; member_id: MemberId };
  export type PlanningResponse = {
    entries: Array<PlanningEntryResponse>;
    resources: Array<PlanningResourceResponse>;
    timezone: string;
    work_time: Array<PlanningWorkTimeResponse>;
  };
  export type PostIntakeMessageRequest = { body: string };
  export type PostalAddressBody = Partial<{
    city: string | null;
    complement: string | null;
    country: string | null;
    postal_code: string | null;
    street: string | null;
  }>;
  export type PresenceResponse = {
    organization_id: OrganizationId;
    status: PresenceStatus;
    updated_at: string;
    user_id: UserId;
  };
  export type PreviewDoeStyleRequest = { style: DoeStyle };
  export type ProductResponse = {
    archived_at?: (string | null) | undefined;
    billing_frequency: BillingFrequency;
    category: string;
    cost_cents?: (number | null) | undefined;
    created_at: string;
    datasheet_keys: Array<string>;
    default_vat_rate_bp?: (number | null) | undefined;
    description?: (string | null) | undefined;
    id: ProductId;
    invoice_usage_count: number;
    manufacturer?: (string | null) | undefined;
    name: string;
    organization_id: OrganizationId;
    photo_keys: Array<string>;
    quote_usage_count: number;
    sku?: (string | null) | undefined;
    supplier_id?: (null | SupplierId) | undefined;
    technical_specs?: (string | null) | undefined;
    unit: string;
    unit_price_cents: number;
    updated_at: string;
    usage_count: number;
  };
  export type ProjectProfitabilityResponse = {
    customer_id?: (null | CustomerId) | undefined;
    equipment_cost_cents?: (number | null) | undefined;
    expenses_cents?: (number | null) | undefined;
    labour_cost_cents?: (number | null) | undefined;
    margin_cents?: (number | null) | undefined;
    members_without_rate: Array<MemberId>;
    name: string;
    occupied_minutes: number;
    overlapping_minutes: number;
    planned_minutes: number;
    project_id: ProjectId;
    quoted_cents?: (number | null) | undefined;
    supplier_cost_cents?: (number | null) | undefined;
  };
  export type ProfitabilityResponse = {
    costs_redacted: boolean;
    double_booked: Array<ProjectProfitabilityResponse>;
    incomplete: Array<ProjectProfitabilityResponse>;
    least_profitable: Array<ProjectProfitabilityResponse>;
    members: Array<MemberProfitabilityResponse>;
    most_profitable: Array<ProjectProfitabilityResponse>;
    projects: Array<ProjectProfitabilityResponse>;
  };
  export type ProjectBillingSummaryResponse = {
    billed_cents: number;
    project_id: ProjectId;
    quoted_cents?: (number | null) | undefined;
    remaining_cents?: (number | null) | undefined;
  };
  export type ProjectChannelResponse = {
    archived: boolean;
    created_at: string;
    id: ChannelId;
    name: string;
    organization_id: OrganizationId;
    project_id?: (null | ProjectId) | undefined;
    topic?: (string | null) | undefined;
    updated_at: string;
  };
  export type ProjectSupplierCostLineResponse = {
    allocation_id: SupplierInvoiceLineAllocationId;
    amount_cents: number;
    created_at: string;
    line_label: string;
    supplier_invoice_id: SupplierInvoiceId;
    supplier_invoice_line_id: SupplierInvoiceLineId;
    supplier_invoice_number: string;
    supplier_name: string;
  };
  export type ProjectSupplierCostsResponse = {
    allocated_cents: number;
    lines: Array<ProjectSupplierCostLineResponse>;
    project_id: ProjectId;
  };
  export type ProjectTemplateId = string;
  export type ProjectTemplateResponse = {
    archived_at?: (string | null) | undefined;
    created_at: string;
    description?: (string | null) | undefined;
    id: ProjectTemplateId;
    name: string;
    organization_id: OrganizationId;
    tasks?: (Array<ProjectTemplateTaskResponse> | null) | undefined;
    updated_at: string;
  };
  export type ProjectTemplateTaskId = string;
  export type ProjectTemplateTaskResponse = {
    all_day: boolean;
    blocks_availability: boolean;
    day_offset: number;
    description?: (string | null) | undefined;
    ends_minute?: (number | null) | undefined;
    expenses_cents: number;
    expenses_label?: (string | null) | undefined;
    id: ProjectTemplateTaskId;
    organization_id: OrganizationId;
    parent_index?: (number | null) | undefined;
    position: number;
    starts_minute?: (number | null) | undefined;
    template_id: ProjectTemplateId;
    title: string;
  };
  export type ProposeInterventionRequest = { done_on: string };
  export type PutDayOverrideRequest = {
    end?: (string | null) | undefined;
    kind: OverrideKind;
    lunch?: (null | LunchInput) | undefined;
    note?: string | undefined;
    start?: (string | null) | undefined;
  };
  export type PutMemberContractRequest = {
    end_date?: (string | null) | undefined;
    schedule: ScheduleInput;
    start_date: string;
    trial_end_date?: (string | null) | undefined;
    type: ContractType;
  };
  export type PutMemberProfileRequest = Partial<{
    address: PostalAddressBody;
    job_title: string | null;
    landline: string | null;
    manager_member_id: null | MemberId;
    mobile: string | null;
    personal_email: string | null;
    work_email: string | null;
    work_permit_end_date: string | null;
  }>;
  export type RhythmSlotRequest = { ends_minute: number; starts_minute: number; weekday: number };
  export type PutRhythmRequest = {
    effective_from: string;
    effective_to?: (string | null) | undefined;
    slots: Array<RhythmSlotRequest>;
  };
  export type WorkSlotRequest = { ends_minute: number; starts_minute: number; work_date: string };
  export type PutWorkSlotsRequest = { slots: Array<WorkSlotRequest> };
  export type QuoteInvoicingBlockerResponse = "QUOTE_NOT_ACCEPTED" | "QUOTE_ALREADY_INVOICED" | "DEPOSITS_COVER_QUOTE";
  export type QuoteInvoicingResponse = {
    blocker?: (null | QuoteInvoicingBlockerResponse) | undefined;
    can_deposit: boolean;
    can_full: boolean;
    invoiced_percent: string;
    invoices: Array<InvoiceResponse>;
    remaining_percent: string;
  };
  export type QuoteLineResponse = {
    billing_frequency: BillingFrequency;
    created_at: string;
    discount_percent: string;
    id: QuoteLineId;
    label: string;
    net_cents: number;
    notes?: (string | null) | undefined;
    organization_id: OrganizationId;
    photo_keys: Array<string>;
    product_id?: (null | ProductId) | undefined;
    quantity: string;
    quote_id: QuoteId;
    service_rate_id?: (null | ServiceRateId) | undefined;
    unit: string;
    unit_price_cents: number;
    updated_at: string;
    vat_rate_bp?: (number | null) | undefined;
  };
  export type RecipientSnapshot = {
    address: SnapshotAddress;
    contact_email?: (string | null) | undefined;
    contact_name?: (string | null) | undefined;
    name: string;
    registration_number?: (string | null) | undefined;
    vat_number?: (string | null) | undefined;
  };
  export type RecurringTotalResponse = {
    frequency: BillingFrequency;
    gross_cents: number;
    net_cents: number;
    vat_cents: number;
  };
  export type QuoteStatus = "DRAFT" | "SENT" | "ACCEPTED" | "DECLINED" | "CANCELLED" | "SUPERSEDED";
  export type QuoteVatBreakdownLineResponse = { rate_bp: number; vat_cents: number };
  export type QuoteResponse = {
    billing_address_id?: (null | CustomerContextId) | undefined;
    billing_contact_id?: (null | CustomerContactId) | undefined;
    created_at: string;
    customer_context_id: CustomerContextId;
    customer_id: CustomerId;
    deal_id?: (null | DealId) | undefined;
    decline_reason?: (string | null) | undefined;
    deposit_percent: string;
    gross_cents: number;
    id: QuoteId;
    is_expired: boolean;
    issuer_snapshot?: (null | IssuerSnapshot) | undefined;
    lines: Array<QuoteLineResponse>;
    net_cents: number;
    notes?: (string | null) | undefined;
    organization_id: OrganizationId;
    owner_member_id?: (null | MemberId) | undefined;
    recipient_snapshot?: (null | RecipientSnapshot) | undefined;
    recurring: Array<RecurringTotalResponse>;
    reference?: (string | null) | undefined;
    responded_at?: (string | null) | undefined;
    responded_by?: (string | null) | undefined;
    revision_of_id?: (null | QuoteId) | undefined;
    status: QuoteStatus;
    superseded_by_id?: (null | QuoteId) | undefined;
    title: string;
    updated_at: string;
    valid_until?: (string | null) | undefined;
    vat_breakdown: Array<QuoteVatBreakdownLineResponse>;
  };
  export type TaskProposalResponse = {
    quote_line_id: QuoteLineId;
    suggested_minutes?: (number | null) | undefined;
    title: string;
  };
  export type QuotePlanProposalResponse = { quote: QuoteResponse; tasks: Array<TaskProposalResponse> };
  export type QuotePlanResponse = { project: PlannedProjectResponse; tasks: Array<PlannedTaskResponse> };
  export type RecapLine = {
    contract_id: MaintenanceContractId;
    due: Due;
    label: string;
    last_done_on?: (string | null) | undefined;
    line_id: MaintenanceLineId;
    minutes_per_intervention?: (number | null) | undefined;
    reference?: (string | null) | undefined;
    site: ContractSite;
  };
  export type RecapGroup = { lines: Array<RecapLine>; town?: (string | null) | undefined };
  export type Recap = { groups: Array<RecapGroup>; week: IsoWeekNumber; year: number };
  export type ReceiptId = string;
  export type ReceiptResponse = {
    amount_cents: number;
    created_at: string;
    customer_id?: (null | CustomerId) | undefined;
    id: ReceiptId;
    method: PaymentMethod;
    note?: (string | null) | undefined;
    organization_id: OrganizationId;
    paid_on: string;
    reference?: (string | null) | undefined;
  };
  export type RecordInterventionRequest = { done_on: string };
  export type RecordInvoicePaymentRequest = {
    amount_cents: number;
    method: PaymentMethod;
    note?: (string | null) | undefined;
    paid_on: string;
    reference?: (string | null) | undefined;
  };
  export type RecordReceiptRequest = {
    amount_cents: number;
    customer_id?: (null | CustomerId) | undefined;
    method: PaymentMethod;
    note?: (string | null) | undefined;
    paid_on: string;
    reference?: (string | null) | undefined;
  };
  export type RecoverTimeEntryRequest = { ended_at: string };
  export type RecurrenceRuleResponse =
    | { frequency: "DAILY" }
    | { frequency: "WEEKLY"; weekdays: Array<number> }
    | { day_of_month: number; frequency: "MONTHLY" };
  export type RejectAbsenceRequest = { comment: string };
  export type RejectExtraPeriodRequest = { comment: string };
  export type RejectInterventionRequest = Partial<{ note: string | null }>;
  export type RejectSupplierInvoiceRequest = Partial<{ notes: string | null }>;
  export type ReplaceBillingSettingsRequest = {
    bic?: (string | null) | undefined;
    default_vat_rate_bp: number;
    iban?: (string | null) | undefined;
    invoice_footer: string;
    late_penalty: string;
    number_padding: number;
    payment_terms_days: number;
    quote_validity_days: number;
    recovery_fee_cents: number;
  };
  export type ReplaceLineAllocationsRequest = { allocations: Array<LineAllocationShareRequest> };
  export type ReplaceProjectTemplateTasksRequest = { tasks: Array<ProjectTemplateTaskShapeRequest> };
  export type ReplaceSupportSettingsRequest = { display_fields: Array<string> };
  export type ReplayRunRequest = { connector_id: string };
  export type ReportAssignmentRequest = { comment?: (string | null) | undefined; reported_minutes: number };
  export type ResolveAssignmentReportRequest = {
    resolution: AssignmentReportResolution;
    resolution_note?: (string | null) | undefined;
  };
  export type RhythmSlotResponse = { ends_minute: number; starts_minute: number; weekday: number };
  export type RhythmResponse = {
    created_at: string;
    effective_from: string;
    effective_to?: (string | null) | undefined;
    employee_id: EmployeeId;
    id: EmployeeRhythmId;
    organization_id: OrganizationId;
    slots: Array<RhythmSlotResponse>;
    updated_at: string;
  };
  export type RoleResponse = {
    created_at: string;
    id: RoleId;
    is_seeded: boolean;
    name: string;
    organization_id: OrganizationId;
    permissions: Array<string>;
    updated_at: string;
  };
  export type RunResponse = {
    created_at: string;
    error?: (string | null) | undefined;
    failed_connector_id?: (string | null) | undefined;
    finished_at?: (string | null) | undefined;
    id: string;
    next_attempt_at?: (string | null) | undefined;
    number: number;
    organization_id: OrganizationId;
    started_at?: (string | null) | undefined;
    status: string;
    steps_done: number;
    steps_total: number;
    trigger_event_id?: (string | null) | undefined;
    trigger_event_name?: (string | null) | undefined;
    trigger_id?: (string | null) | undefined;
    trigger_payload?: unknown | undefined;
    workflow_id: string;
    workflow_version_created_at: string;
    workflow_version_id: string;
  };
  export type RunStepResponse = {
    attempts: number;
    connector_id: string;
    created_at: string;
    error?: (string | null) | undefined;
    finished_at?: (string | null) | undefined;
    id: string;
    input?: unknown | undefined;
    iteration_path: string;
    output?: unknown | undefined;
    resume_at?: (string | null) | undefined;
    started_at?: (string | null) | undefined;
    status: string;
  };
  export type RunDetailResponse = RunResponse & {
    graph?: (null | GraphDto) | undefined;
    steps: Array<RunStepResponse>;
  };
  export type WorkflowLayoutDto = Record<string, unknown>;
  export type SaveWorkflowVersionRequest = { graph: GraphDto; layout?: (null | WorkflowLayoutDto) | undefined };
  export type SeriesNumberingResponse = {
    kind: NumberSeriesKind;
    last_assigned_number: number;
    next_number: number;
    next_reference: string;
    padding: number;
    prefix: string;
    year: number;
  };
  export type ServiceRateResponse = {
    archived_at?: (string | null) | undefined;
    billing_frequency: BillingFrequency;
    category: string;
    cost_cents?: (number | null) | undefined;
    created_at: string;
    default_vat_rate_bp?: (number | null) | undefined;
    description?: (string | null) | undefined;
    id: ServiceRateId;
    invoice_usage_count: number;
    label: string;
    organization_id: OrganizationId;
    quote_usage_count: number;
    rate_cents: number;
    unit: string;
    updated_at: string;
    usage_count: number;
  };
  export type ServiceRateUnit = "FLAT_RATE" | "HOUR" | "DAY" | "UNIT" | "ML" | "M2" | "M3" | "KG" | "TONNE" | "LITRE";
  export type SetEmployeeCostBasisRequest = {
    effective_from: string;
    hourly_rate_cents?: (number | null) | undefined;
    is_salaried: boolean;
    monthly_cost_cents?: (number | null) | undefined;
    weekly_contract_minutes: number;
  };
  export type SetNextNumberRequest = { next_number: number };
  export type SetPresenceRequest = { status: PresenceStatus };
  export type StartOauthRequest = { redirect_uri: string; return_to: string };
  export type StartOauthResponse = { authorization_url: string };
  export type StartRunRequest = Partial<{ trigger_payload: unknown }>;
  export type StartTimeEntryRequest = { task_id: TaskId };
  export type StartedRunResponse = { run_id: string };
  export type StatusFilter = "proposed" | "validated" | "rejected";
  export type SupplierAddressResponse = {
    city: string;
    country: string;
    id: string;
    kind: AddressKind;
    label: string;
    line2: string;
    postal_code: string;
    street: string;
  };
  export type SupplierContactResponse = {
    email: string;
    first_name: string;
    id: string;
    last_name: string;
    mobile: string;
    phone: string;
    role: string;
  };
  export type SupplierResponse = {
    address?: (string | null) | undefined;
    addresses: Array<SupplierAddressResponse>;
    archived_at?: (string | null) | undefined;
    contacts: Array<SupplierContactResponse>;
    created_at: string;
    default_category_id?: (null | ExpenseCategoryId) | undefined;
    email?: (string | null) | undefined;
    iban?: (string | null) | undefined;
    id: SupplierId;
    name: string;
    notes?: (string | null) | undefined;
    organization_id: OrganizationId;
    payment_terms_days?: (number | null) | undefined;
    phone?: (string | null) | undefined;
    registration_number?: (string | null) | undefined;
    updated_at: string;
    vat_number?: (string | null) | undefined;
  };
  export type SupportApiKeyResponse = {
    created_at: string;
    id: SupportApiKeyId;
    last_used_at?: (string | null) | undefined;
    name: string;
    prefix: string;
    revoked_at?: (string | null) | undefined;
  };
  export type SupportAttachmentResponse = {
    created_at: string;
    file_name: string;
    id: AttachmentId;
    mime_type: string;
    size_bytes: number;
  };
  export type SupportSettingsResponse = { display_fields: Array<string> };
  export type SurchargeKind = "OVERTIME" | "SUNDAY" | "HOLIDAY" | "NIGHT" | "OTHER";
  export type SurchargeRequest = { kind: SurchargeKind; name: string; rate_bp: number };
  export type SurchargeResponse = {
    archived_at?: (string | null) | undefined;
    created_at: string;
    id: SurchargeId;
    kind: SurchargeKind;
    name: string;
    organization_id: OrganizationId;
    rate_bp: number;
  };
  export type TaskCommentAuthorResponse = { display_name: string; id: UserId };
  export type TaskCommentId = string;
  export type TaskCommentResponse = {
    author: TaskCommentAuthorResponse;
    author_is_self: boolean;
    body: string;
    created_at: string;
    id: TaskCommentId;
    organization_id: OrganizationId;
    task_id: TaskId;
    updated_at: string;
  };
  export type TaskRecurrenceResponse = RecurrenceRuleResponse & {
    all_day: boolean;
    assignee_member_ids: Array<MemberId>;
    blocks_availability: boolean;
    created_at: string;
    customer_context_id?: (null | CustomerContextId) | undefined;
    customer_id?: (null | CustomerId) | undefined;
    description?: (string | null) | undefined;
    duration_minutes: number;
    ends_on?: (string | null) | undefined;
    horizon_filled_to: string;
    id: TaskRecurrenceId;
    organization_id: OrganizationId;
    project_id?: (null | ProjectId) | undefined;
    start_time: string;
    starts_on: string;
    timezone: string;
    title: string;
    updated_at: string;
  };
  export type TicketEntryResponse = {
    attachments: Array<SupportAttachmentResponse>;
    author_member_id?: (null | MemberId) | undefined;
    body: string;
    created_at: string;
    id: TicketEntryId;
    kind: EntryKind;
    ticket_id: TicketId;
    updated_at: string;
  };
  export type TicketResponse = {
    assigned_at?: (string | null) | undefined;
    assignee_member_id?: (null | MemberId) | undefined;
    badges: Array<BadgeResponse>;
    body: string;
    created_at: string;
    custom_fields: Record<string, unknown>;
    external_id?: (string | null) | undefined;
    id: TicketId;
    number: TicketNumber;
    status: TicketStatus;
    subject: string;
    updated_at: string;
  };
  export type TimeEntryId = string;
  export type TimeEntryPhotoId = string;
  export type TimeEntryPhotoResponse = {
    created_at: string;
    id: TimeEntryPhotoId;
    phase: TimeEntryPhotoPhase;
    storage_key: string;
    time_entry_id: TimeEntryId;
  };
  export type TimeEntryResponse = {
    employee_id: EmployeeId;
    ended_at?: (string | null) | undefined;
    id: TimeEntryId;
    organization_id: OrganizationId;
    photos: Array<TimeEntryPhotoResponse>;
    started_at: string;
    task_id: TaskId;
    worked_minutes?: (number | null) | undefined;
  };
  export type TransitionTicketRequest = { status: string };
  export type UnreadChannelResponse = {
    channel_id: ChannelId;
    first_unread_message_id: MessageId;
    unread_count: number;
  };
  export type UnreadResponse = { channel_ids: Array<ChannelId>; channels: Array<UnreadChannelResponse> };
  export type UpdateAbsenceRequest = Partial<{
    all_day: boolean | null;
    ends_at: string | null;
    kind: null | AbsenceKind;
    note: string | null;
    proof_file_key: string | null;
    starts_at: string | null;
  }>;
  export type UpdateCategoryRequest = { name: string; position: number };
  export type UpdateChannelRequest = {
    category_id?: (null | CategoryId) | undefined;
    name: string;
    position: number;
    topic?: (string | null) | undefined;
  };
  export type UpdateCredentialRequest = Partial<{
    data: Record<string, unknown> | null;
    description: string | null;
    expires_at: string | null;
    name: string | null;
  }>;
  export type UpdateCustomerContactRequest = {
    email?: (string | null) | undefined;
    first_name: string;
    is_primary: boolean;
    last_name: string;
    mobile?: (string | null) | undefined;
    phone?: (string | null) | undefined;
    role?: (string | null) | undefined;
  };
  export type UpdateCustomerContextRequest = {
    address_line?: (string | null) | undefined;
    address_line2?: (string | null) | undefined;
    city?: (string | null) | undefined;
    country?: string | undefined;
    kind?: AddressKind | undefined;
    label: string;
    photo_key?: (string | null) | undefined;
    postal_code?: (string | null) | undefined;
  };
  export type UpdateCustomerRequest = {
    billing_address_id?: (null | CustomerContextId) | undefined;
    billing_contact_id?: (null | CustomerContactId) | undefined;
    email?: (string | null) | undefined;
    kind?: CustomerKind | undefined;
    lead_source?: (null | LeadSource) | undefined;
    name: string;
    notes?: (string | null) | undefined;
    owner_member_id?: (null | MemberId) | undefined;
    payment_terms_days?: (number | null) | undefined;
    phone?: (string | null) | undefined;
    registration_number?: (string | null) | undefined;
    status: CustomerStatus;
    vat_number?: (string | null) | undefined;
    website?: (string | null) | undefined;
  };
  export type UpdateDealRequest = {
    amount_cents: number;
    customer_id: CustomerId;
    expected_close_date?: (string | null) | undefined;
    notes?: (string | null) | undefined;
    owner_member_id?: (null | MemberId) | undefined;
    source?: (null | LeadSource) | undefined;
    title: string;
  };
  export type UpdateDoeModelRequest = {
    description?: (string | null) | undefined;
    name: string;
    sections: Array<DoeModelSection>;
    style?: (null | DoeStyle) | undefined;
  };
  export type UpdateDoeRequest = {
    author?: (string | null) | undefined;
    cover_photo_key?: (string | null) | undefined;
    expected_updated_at?: (string | null) | undefined;
    issued_on: string;
    lot?: (string | null) | undefined;
    parties: Array<DoeParty>;
    place?: (string | null) | undefined;
    revision: string;
    sections: Array<DoeSection>;
    title: string;
  };
  export type UpdateDoeSettingsRequest = { style: DoeStyle };
  export type UpdateEquipmentRequest = { hourly_rate_cents: number; name: string };
  export type UpdateExpenseRequest = {
    amount_ht_cents: number;
    category_id: ExpenseCategoryId;
    customer_id?: (null | CustomerId) | undefined;
    date: string;
    due_date?: (string | null) | undefined;
    label: string;
    note?: (string | null) | undefined;
    proof_file_key?: (string | null) | undefined;
    reference?: (string | null) | undefined;
    supplier_id?: (null | SupplierId) | undefined;
    vat_cents?: number | undefined;
  };
  export type UpdateFolderRequest = Partial<{ name: string | null; parent_id: string | null }>;
  export type UpdateInvoiceRequest = {
    customer_context_id: CustomerContextId;
    customer_id: CustomerId;
    delivery_address?: (null | DeliveryAddressRequest) | undefined;
    legal_notice?: (string | null) | undefined;
    lines: Array<InvoiceLineRequest>;
    notes?: (string | null) | undefined;
    operation_nature?: (null | OperationNature) | undefined;
    payment_terms_days?: (number | null) | undefined;
    project_id?: (null | ProjectId) | undefined;
    title?: (string | null) | undefined;
  };
  export type VatStatusRequest = { type: "subject"; vat_number: string } | { basis: string; type: "not_subject" };
  export type UpdateLegalIdentityRequest = Partial<{
    address_city: string | null;
    address_country: string | null;
    address_line1: string | null;
    address_line2: string | null;
    address_postal_code: string | null;
    contact_email: string | null;
    contact_phone: string | null;
    insurance_mention: string | null;
    legal_form: string | null;
    legal_name: string | null;
    registration_number: string | null;
    share_capital_cents: number | null;
    vat_on_debits: boolean;
    vat_status: null | VatStatusRequest;
  }>;
  export type UpdateMemberRequest = Partial<{ first_name: string | null; last_name: string | null }>;
  export type UpdateMessageRequest = { content: string };
  export type UpdateOrganizationRequest = {
    field_clock_enabled: boolean;
    mcp_enabled?: (boolean | null) | undefined;
    name: string;
    slug: string;
  };
  export type UpdateProductRequest = {
    archived?: (boolean | null) | undefined;
    billing_frequency?: (null | BillingFrequency) | undefined;
    category?: (string | null) | undefined;
    cost_cents?: (number | null) | undefined;
    datasheet_keys?: Array<string> | undefined;
    default_vat_rate_bp?: (number | null) | undefined;
    description?: (string | null) | undefined;
    manufacturer?: (string | null) | undefined;
    name: string;
    photo_keys: Array<string>;
    sku?: (string | null) | undefined;
    supplier_id?: (null | SupplierId) | undefined;
    technical_specs?: (string | null) | undefined;
    unit: string;
    unit_price_cents: number;
  };
  export type UpdateProjectRequest = {
    customer_context_id?: (null | CustomerContextId) | undefined;
    customer_id?: (null | CustomerId) | undefined;
    name: string;
    quote_id?: (null | QuoteId) | undefined;
  };
  export type UpdateProjectTemplateRequest = { description?: (string | null) | undefined; name: string };
  export type UpdateQuoteRequest = {
    billing_address_id?: (null | CustomerContextId) | undefined;
    billing_contact_id?: (null | CustomerContactId) | undefined;
    customer_context_id: CustomerContextId;
    customer_id: CustomerId;
    deal_id?: (null | DealId) | undefined;
    deposit_percent?: (string | null) | undefined;
    lines: Array<QuoteLineRequest>;
    notes?: (string | null) | undefined;
    owner_member_id?: (null | MemberId) | undefined;
    status: QuoteStatus;
    title: string;
    valid_until?: (string | null) | undefined;
  };
  export type UpdateQuoteStatusRequest = {
    decline_reason?: (string | null) | undefined;
    responded_by?: (string | null) | undefined;
    status: QuoteStatus;
  };
  export type UpdateRoleRequest = { name: string; permissions: Array<string> };
  export type UpdateServiceRateRequest = {
    archived?: (boolean | null) | undefined;
    billing_frequency?: (null | BillingFrequency) | undefined;
    category?: (string | null) | undefined;
    cost_cents?: (number | null) | undefined;
    default_vat_rate_bp?: (number | null) | undefined;
    description?: (string | null) | undefined;
    label: string;
    rate_cents: number;
    unit: string;
  };
  export type UpdateSupplierInvoiceRequest = Partial<{ notes: string | null }>;
  export type UpdateSupplierRequest = {
    address?: (string | null) | undefined;
    addresses?: Array<SupplierAddressRequest> | undefined;
    contacts?: Array<SupplierContactRequest> | undefined;
    default_category_id?: (null | ExpenseCategoryId) | undefined;
    email?: (string | null) | undefined;
    iban?: (string | null) | undefined;
    name: string;
    notes?: (string | null) | undefined;
    payment_terms_days?: (number | null) | undefined;
    phone?: (string | null) | undefined;
    registration_number?: (string | null) | undefined;
    vat_number?: (string | null) | undefined;
  };
  export type UpdateTaskCommentRequest = { body: string };
  export type UpdateTaskLabelRequest = { color: string; name: string };
  export type UpdateTaskRecurrenceRequest = (null | RecurrenceRuleRequest) &
    Partial<{
      all_day: boolean | null;
      assignee_member_ids: Array<MemberId> | null;
      blocks_availability: boolean | null;
      description: string | null;
      duration_minutes: number | null;
      ends_on: string | null;
      project_id: null | ProjectId;
      start_time: string | null;
      title: string | null;
    }>;
  export type UpdateTaskRequest = Partial<{
    all_day: boolean | null;
    assignees: Array<AssigneeRefRequest> | null;
    blocks_availability: boolean | null;
    description: string | null;
    ends_at: string | null;
    equipment_ids: Array<EquipmentId> | null;
    expenses_cents: number | null;
    expenses_label: string | null;
    following_task_id: null | TaskId;
    label_ids: Array<TaskLabelId> | null;
    parent_task_id: null | TaskId;
    preceding_task_id: null | TaskId;
    project_id: null | ProjectId;
    starts_at: string | null;
    status: null | TaskStatus;
    title: string | null;
  }>;
  export type UpdateThreadRequest = { archived: boolean; name: string };
  export type UpdateWebhookRequest = { avatar_url?: (string | null) | undefined; name: string };
  export type UpdateWorkflowRequest = Partial<{
    archived: boolean | null;
    description: string | null;
    enabled: boolean | null;
    folder_id: string | null;
    name: string | null;
    tags: Array<string> | null;
  }>;
  export type UpsertEmployeeProfileRequest = {
    hourly_rate_cents?: (number | null) | undefined;
    is_salaried: boolean;
    monthly_cost_cents?: (number | null) | undefined;
    weekly_contract_minutes: number;
  };
  export type UpsertOverwriteRequest = { allow: number; deny: number };
  export type WebhookAccepted = { run_id: string };
  export type WebhookBodyInvalid = { code: string; keys: Array<string>; message: string; status: number };
  export type WebhookCreatedResponse = {
    avatar_url?: (string | null) | undefined;
    channel_id: ChannelId;
    created_at: string;
    created_by: UserId;
    id: WebhookId;
    name: string;
    organization_id: OrganizationId;
    token: string;
    updated_at: string;
  };
  export type WebhookResponse = {
    avatar_url?: (string | null) | undefined;
    channel_id: ChannelId;
    created_at: string;
    created_by: UserId;
    id: WebhookId;
    name: string;
    organization_id: OrganizationId;
    updated_at: string;
  };
  export type WorkSlotId = string;
  export type WorkSlotResponse = {
    ends_minute: number;
    id: WorkSlotId;
    member_id: MemberId;
    organization_id: OrganizationId;
    starts_minute: number;
    work_date: string;
  };
  export type WorkTimeResponse = { rhythms: Array<RhythmResponse>; work_slots: Array<WorkSlotResponse> };
  export type WorkedHoursRow = {
    member_id: MemberId;
    missing_cost?: (null | MissingCost) | undefined;
    planned_minutes: number;
  };
  export type WorkedHoursResponse = {
    costs_redacted: boolean;
    members: Array<WorkedHoursRow>;
    total_planned_minutes: number;
  };
  export type WorkflowVersionResponse = {
    created_at: string;
    created_by?: (string | null) | undefined;
    graph: GraphDto;
    id: string;
    version: number;
    workflow_id: string;
  };
  export type WorkflowDetailResponse = {
    archived_at?: (string | null) | undefined;
    created_at: string;
    current_version?: (null | WorkflowVersionResponse) | undefined;
    description?: (string | null) | undefined;
    enabled: boolean;
    folder_id?: (string | null) | undefined;
    id: string;
    layout?: (null | WorkflowLayoutDto) | undefined;
    name: string;
    organization_id: OrganizationId;
    tags: Array<string>;
    updated_at: string;
  };
  export type WorkflowResponse = {
    archived_at?: (string | null) | undefined;
    created_at: string;
    current_version_id?: (string | null) | undefined;
    description?: (string | null) | undefined;
    enabled: boolean;
    folder_id?: (string | null) | undefined;
    id: string;
    name: string;
    organization_id: OrganizationId;
    tags: Array<string>;
    updated_at: string;
  };

  // </Schemas>
}

export namespace Endpoints {
  // <Endpoints>

  export type patch_ResolveAssignmentReport = {
    method: "PATCH";
    path: "/api/v1/assignment-reports/{assignment_report_id}/resolution";
    requestFormat: "json";
    parameters: {
      path: { assignment_report_id: string };

      body: Schemas.ResolveAssignmentReportRequest;
    };
    responses: {
      200: {
        data: {
          comment?: (string | null) | undefined;
          created_at: string;
          id: Schemas.AssignmentReportId;
          organization_id: Schemas.OrganizationId;
          reported_by: Schemas.MemberId;
          reported_minutes: number;
          resolution: Schemas.AssignmentReportResolution;
          resolution_note?: (string | null) | undefined;
          resolved_at?: (string | null) | undefined;
          resolved_by?: (null | Schemas.MemberId) | undefined;
          task_assignment_id: Schemas.TaskAssignmentId;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type delete_DeleteCategory = {
    method: "DELETE";
    path: "/api/v1/chat/categories/{category_id}";
    requestFormat: "json";
    parameters: {
      path: { category_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type patch_UpdateCategory = {
    method: "PATCH";
    path: "/api/v1/chat/categories/{category_id}";
    requestFormat: "json";
    parameters: {
      path: { category_id: string };

      body: Schemas.UpdateCategoryRequest;
    };
    responses: {
      200: {
        data: {
          created_at: string;
          id: Schemas.CategoryId;
          name: string;
          organization_id: Schemas.OrganizationId;
          position: number;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type get_GetChannel = {
    method: "GET";
    path: "/api/v1/chat/channels/{channel_id}";
    requestFormat: "json";
    parameters: {
      path: { channel_id: string };
    };
    responses: {
      200: {
        data: {
          archived: boolean;
          category_id?: (null | Schemas.CategoryId) | undefined;
          channel_type: Schemas.ChannelType;
          created_at: string;
          id: Schemas.ChannelId;
          name: string;
          organization_id: Schemas.OrganizationId;
          origin_message_id?: (null | Schemas.MessageId) | undefined;
          parent_id?: (null | Schemas.ChannelId) | undefined;
          position: number;
          project_id?: (null | Schemas.ProjectId) | undefined;
          topic?: (string | null) | undefined;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type delete_DeleteChannel = {
    method: "DELETE";
    path: "/api/v1/chat/channels/{channel_id}";
    requestFormat: "json";
    parameters: {
      path: { channel_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type patch_UpdateChannel = {
    method: "PATCH";
    path: "/api/v1/chat/channels/{channel_id}";
    requestFormat: "json";
    parameters: {
      path: { channel_id: string };

      body: Schemas.UpdateChannelRequest;
    };
    responses: {
      200: {
        data: {
          archived: boolean;
          category_id?: (null | Schemas.CategoryId) | undefined;
          channel_type: Schemas.ChannelType;
          created_at: string;
          id: Schemas.ChannelId;
          name: string;
          organization_id: Schemas.OrganizationId;
          origin_message_id?: (null | Schemas.MessageId) | undefined;
          parent_id?: (null | Schemas.ChannelId) | undefined;
          position: number;
          project_id?: (null | Schemas.ProjectId) | undefined;
          topic?: (string | null) | undefined;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type get_ListMessages = {
    method: "GET";
    path: "/api/v1/chat/channels/{channel_id}/messages";
    requestFormat: "json";
    parameters: {
      query: Partial<{ before: string; after: string; limit: number }>;
      path: { channel_id: string };
    };
    responses: {
      200: {
        data: Array<{
          attachments: Array<Schemas.AttachmentResponse>;
          author_type: Schemas.AuthorType;
          author_user_id?: (null | Schemas.UserId) | undefined;
          author_webhook_id?: (null | Schemas.WebhookId) | undefined;
          channel_id: Schemas.ChannelId;
          components?: (Array<Schemas.Component> | null) | undefined;
          content: string;
          created_at: string;
          edited_at?: (string | null) | undefined;
          id: Schemas.MessageId;
          mention_channel_ids: Array<Schemas.ChannelId>;
          mention_everyone: boolean;
          mention_role_ids: Array<Schemas.RoleId>;
          mention_user_ids: Array<Schemas.UserId>;
          organization_id: Schemas.OrganizationId;
          reactions: Array<Schemas.ReactionCountResponse>;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateMessage = {
    method: "POST";
    path: "/api/v1/chat/channels/{channel_id}/messages";
    requestFormat: "json";
    parameters: {
      path: { channel_id: string };

      body: Schemas.CreateMessageRequest;
    };
    responses: {
      201: {
        data: {
          attachments: Array<Schemas.AttachmentResponse>;
          author_type: Schemas.AuthorType;
          author_user_id?: (null | Schemas.UserId) | undefined;
          author_webhook_id?: (null | Schemas.WebhookId) | undefined;
          channel_id: Schemas.ChannelId;
          components?: (Array<Schemas.Component> | null) | undefined;
          content: string;
          created_at: string;
          edited_at?: (string | null) | undefined;
          id: Schemas.MessageId;
          mention_channel_ids: Array<Schemas.ChannelId>;
          mention_everyone: boolean;
          mention_role_ids: Array<Schemas.RoleId>;
          mention_user_ids: Array<Schemas.UserId>;
          organization_id: Schemas.OrganizationId;
          reactions: Array<Schemas.ReactionCountResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
    };
  };
  export type get_ListChannelPermissions = {
    method: "GET";
    path: "/api/v1/chat/channels/{channel_id}/permissions";
    requestFormat: "json";
    parameters: {
      path: { channel_id: string };
    };
    responses: {
      200: {
        data: Array<{
          allow: number;
          created_at: string;
          deny: number;
          target_id?: (string | null) | undefined;
          target_type: string;
          updated_at: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type put_UpsertEveryoneOverwrite = {
    method: "PUT";
    path: "/api/v1/chat/channels/{channel_id}/permissions/everyone";
    requestFormat: "json";
    parameters: {
      path: { channel_id: string };

      body: Schemas.UpsertOverwriteRequest;
    };
    responses: {
      200: {
        data: {
          allow: number;
          created_at: string;
          deny: number;
          target_id?: (string | null) | undefined;
          target_type: string;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type delete_DeleteEveryoneOverwrite = {
    method: "DELETE";
    path: "/api/v1/chat/channels/{channel_id}/permissions/everyone";
    requestFormat: "json";
    parameters: {
      path: { channel_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type put_UpsertTargetOverwrite = {
    method: "PUT";
    path: "/api/v1/chat/channels/{channel_id}/permissions/{target_type}/{target_id}";
    requestFormat: "json";
    parameters: {
      path: { channel_id: string; target_type: string; target_id: string };

      body: Schemas.UpsertOverwriteRequest;
    };
    responses: {
      200: {
        data: {
          allow: number;
          created_at: string;
          deny: number;
          target_id?: (string | null) | undefined;
          target_type: string;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type delete_DeleteTargetOverwrite = {
    method: "DELETE";
    path: "/api/v1/chat/channels/{channel_id}/permissions/{target_type}/{target_id}";
    requestFormat: "json";
    parameters: {
      path: { channel_id: string; target_type: string; target_id: string };
    };
    responses: { 204: unknown; 400: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type put_MarkChannelRead = {
    method: "PUT";
    path: "/api/v1/chat/channels/{channel_id}/read";
    requestFormat: "json";
    parameters: {
      path: { channel_id: string };

      body: Schemas.MarkChannelReadRequest;
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type get_ListThreads = {
    method: "GET";
    path: "/api/v1/chat/channels/{channel_id}/threads";
    requestFormat: "json";
    parameters: {
      path: { channel_id: string };
    };
    responses: {
      200: {
        data: Array<{
          archived: boolean;
          category_id?: (null | Schemas.CategoryId) | undefined;
          channel_type: Schemas.ChannelType;
          created_at: string;
          id: Schemas.ChannelId;
          name: string;
          organization_id: Schemas.OrganizationId;
          origin_message_id?: (null | Schemas.MessageId) | undefined;
          parent_id?: (null | Schemas.ChannelId) | undefined;
          position: number;
          project_id?: (null | Schemas.ProjectId) | undefined;
          topic?: (string | null) | undefined;
          updated_at: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type post_CreateThread = {
    method: "POST";
    path: "/api/v1/chat/channels/{channel_id}/threads";
    requestFormat: "json";
    parameters: {
      path: { channel_id: string };

      body: Schemas.CreateThreadRequest;
    };
    responses: {
      201: {
        data: {
          archived: boolean;
          category_id?: (null | Schemas.CategoryId) | undefined;
          channel_type: Schemas.ChannelType;
          created_at: string;
          id: Schemas.ChannelId;
          name: string;
          organization_id: Schemas.OrganizationId;
          origin_message_id?: (null | Schemas.MessageId) | undefined;
          parent_id?: (null | Schemas.ChannelId) | undefined;
          position: number;
          project_id?: (null | Schemas.ProjectId) | undefined;
          topic?: (string | null) | undefined;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type post_StartTyping = {
    method: "POST";
    path: "/api/v1/chat/channels/{channel_id}/typing";
    requestFormat: "json";
    parameters: {
      path: { channel_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown };
  };
  export type get_ListWebhooks = {
    method: "GET";
    path: "/api/v1/chat/channels/{channel_id}/webhooks";
    requestFormat: "json";
    parameters: {
      path: { channel_id: string };
    };
    responses: {
      200: {
        data: Array<{
          avatar_url?: (string | null) | undefined;
          channel_id: Schemas.ChannelId;
          created_at: string;
          created_by: Schemas.UserId;
          id: Schemas.WebhookId;
          name: string;
          organization_id: Schemas.OrganizationId;
          updated_at: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type post_CreateWebhook = {
    method: "POST";
    path: "/api/v1/chat/channels/{channel_id}/webhooks";
    requestFormat: "json";
    parameters: {
      path: { channel_id: string };

      body: Schemas.CreateWebhookRequest;
    };
    responses: {
      201: {
        data: {
          avatar_url?: (string | null) | undefined;
          channel_id: Schemas.ChannelId;
          created_at: string;
          created_by: Schemas.UserId;
          id: Schemas.WebhookId;
          name: string;
          organization_id: Schemas.OrganizationId;
          token: string;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type delete_DeleteMessage = {
    method: "DELETE";
    path: "/api/v1/chat/messages/{message_id}";
    requestFormat: "json";
    parameters: {
      path: { message_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type patch_UpdateMessage = {
    method: "PATCH";
    path: "/api/v1/chat/messages/{message_id}";
    requestFormat: "json";
    parameters: {
      path: { message_id: string };

      body: Schemas.UpdateMessageRequest;
    };
    responses: {
      200: {
        data: {
          attachments: Array<Schemas.AttachmentResponse>;
          author_type: Schemas.AuthorType;
          author_user_id?: (null | Schemas.UserId) | undefined;
          author_webhook_id?: (null | Schemas.WebhookId) | undefined;
          channel_id: Schemas.ChannelId;
          components?: (Array<Schemas.Component> | null) | undefined;
          content: string;
          created_at: string;
          edited_at?: (string | null) | undefined;
          id: Schemas.MessageId;
          mention_channel_ids: Array<Schemas.ChannelId>;
          mention_everyone: boolean;
          mention_role_ids: Array<Schemas.RoleId>;
          mention_user_ids: Array<Schemas.UserId>;
          organization_id: Schemas.OrganizationId;
          reactions: Array<Schemas.ReactionCountResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type get_ListReactors = {
    method: "GET";
    path: "/api/v1/chat/messages/{message_id}/reactions/{emoji}";
    requestFormat: "json";
    parameters: {
      path: { message_id: string; emoji: string };
    };
    responses: {
      200: { data: Array<string>; pagination?: (null | Schemas.PaginationMetadata) | undefined };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type put_AddReaction = {
    method: "PUT";
    path: "/api/v1/chat/messages/{message_id}/reactions/{emoji}";
    requestFormat: "json";
    parameters: {
      path: { message_id: string; emoji: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type delete_RemoveReaction = {
    method: "DELETE";
    path: "/api/v1/chat/messages/{message_id}/reactions/{emoji}";
    requestFormat: "json";
    parameters: {
      path: { message_id: string; emoji: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type put_MarkNotificationRead = {
    method: "PUT";
    path: "/api/v1/chat/notifications/{notification_id}/read";
    requestFormat: "json";
    parameters: {
      path: { notification_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown };
  };
  export type get_ListCategories = {
    method: "GET";
    path: "/api/v1/chat/organizations/{organization_id}/categories";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          created_at: string;
          id: Schemas.CategoryId;
          name: string;
          organization_id: Schemas.OrganizationId;
          position: number;
          updated_at: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateCategory = {
    method: "POST";
    path: "/api/v1/chat/organizations/{organization_id}/categories";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateCategoryRequest;
    };
    responses: {
      201: {
        data: {
          created_at: string;
          id: Schemas.CategoryId;
          name: string;
          organization_id: Schemas.OrganizationId;
          position: number;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      409: unknown;
    };
  };
  export type get_ListChannels = {
    method: "GET";
    path: "/api/v1/chat/organizations/{organization_id}/channels";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          archived: boolean;
          category_id?: (null | Schemas.CategoryId) | undefined;
          channel_type: Schemas.ChannelType;
          created_at: string;
          id: Schemas.ChannelId;
          name: string;
          organization_id: Schemas.OrganizationId;
          origin_message_id?: (null | Schemas.MessageId) | undefined;
          parent_id?: (null | Schemas.ChannelId) | undefined;
          position: number;
          project_id?: (null | Schemas.ProjectId) | undefined;
          topic?: (string | null) | undefined;
          updated_at: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateChannel = {
    method: "POST";
    path: "/api/v1/chat/organizations/{organization_id}/channels";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateChannelRequest;
    };
    responses: {
      201: {
        data: {
          archived: boolean;
          category_id?: (null | Schemas.CategoryId) | undefined;
          channel_type: Schemas.ChannelType;
          created_at: string;
          id: Schemas.ChannelId;
          name: string;
          organization_id: Schemas.OrganizationId;
          origin_message_id?: (null | Schemas.MessageId) | undefined;
          parent_id?: (null | Schemas.ChannelId) | undefined;
          position: number;
          project_id?: (null | Schemas.ProjectId) | undefined;
          topic?: (string | null) | undefined;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      409: unknown;
    };
  };
  export type get_ListDirectChannels = {
    method: "GET";
    path: "/api/v1/chat/organizations/{organization_id}/direct-channels";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          channel_id: Schemas.ChannelId;
          created_at: string;
          organization_id: Schemas.OrganizationId;
          peer_user_id: Schemas.UserId;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_OpenDirectChannel = {
    method: "POST";
    path: "/api/v1/chat/organizations/{organization_id}/direct-channels";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.OpenDirectChannelRequest;
    };
    responses: {
      200: {
        data: {
          channel_id: Schemas.ChannelId;
          created_at: string;
          organization_id: Schemas.OrganizationId;
          peer_user_id: Schemas.UserId;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_ListChatDirectory = {
    method: "GET";
    path: "/api/v1/chat/organizations/{organization_id}/directory";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{ display_name: string; presence: Schemas.PresenceStatus; user_id: Schemas.UserId }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type get_ListNotifications = {
    method: "GET";
    path: "/api/v1/chat/organizations/{organization_id}/notifications";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; unread_only: boolean | null; before: string | null; limit: number | null };
    };
    responses: {
      200: {
        data: Array<{
          channel_id: Schemas.ChannelId;
          created_at: string;
          id: Schemas.NotificationId;
          kind: string;
          message_id: Schemas.MessageId;
          read_at?: (string | null) | undefined;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type put_MarkAllNotificationsRead = {
    method: "PUT";
    path: "/api/v1/chat/organizations/{organization_id}/notifications/read-all";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown };
  };
  export type put_SetPresence = {
    method: "PUT";
    path: "/api/v1/chat/organizations/{organization_id}/presence";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.SetPresenceRequest;
    };
    responses: {
      200: {
        data: {
          organization_id: Schemas.OrganizationId;
          status: Schemas.PresenceStatus;
          updated_at: string;
          user_id: Schemas.UserId;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
    };
  };
  export type get_ListChatRoles = {
    method: "GET";
    path: "/api/v1/chat/organizations/{organization_id}/roles";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{ name: string; role_id: Schemas.RoleId }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type get_ListUnreadChannels = {
    method: "GET";
    path: "/api/v1/chat/organizations/{organization_id}/unread";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: { 200: Schemas.UnreadResponse; 401: unknown; 403: unknown };
  };
  export type delete_DeleteThread = {
    method: "DELETE";
    path: "/api/v1/chat/threads/{channel_id}";
    requestFormat: "json";
    parameters: {
      path: { channel_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type patch_UpdateThread = {
    method: "PATCH";
    path: "/api/v1/chat/threads/{channel_id}";
    requestFormat: "json";
    parameters: {
      path: { channel_id: string };

      body: Schemas.UpdateThreadRequest;
    };
    responses: {
      200: {
        data: {
          archived: boolean;
          category_id?: (null | Schemas.CategoryId) | undefined;
          channel_type: Schemas.ChannelType;
          created_at: string;
          id: Schemas.ChannelId;
          name: string;
          organization_id: Schemas.OrganizationId;
          origin_message_id?: (null | Schemas.MessageId) | undefined;
          parent_id?: (null | Schemas.ChannelId) | undefined;
          position: number;
          project_id?: (null | Schemas.ProjectId) | undefined;
          topic?: (string | null) | undefined;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type delete_DeleteWebhook = {
    method: "DELETE";
    path: "/api/v1/chat/webhooks/{webhook_id}";
    requestFormat: "json";
    parameters: {
      path: { webhook_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type patch_UpdateWebhook = {
    method: "PATCH";
    path: "/api/v1/chat/webhooks/{webhook_id}";
    requestFormat: "json";
    parameters: {
      path: { webhook_id: string };

      body: Schemas.UpdateWebhookRequest;
    };
    responses: {
      200: {
        data: {
          avatar_url?: (string | null) | undefined;
          channel_id: Schemas.ChannelId;
          created_at: string;
          created_by: Schemas.UserId;
          id: Schemas.WebhookId;
          name: string;
          organization_id: Schemas.OrganizationId;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type post_ExecuteWebhook = {
    method: "POST";
    path: "/api/v1/chat/webhooks/{webhook_id}/{token}";
    requestFormat: "json";
    parameters: {
      path: { webhook_id: string; token: string };

      body: Schemas.ExecuteWebhookRequest;
    };
    responses: {
      201: {
        data: {
          attachments: Array<Schemas.AttachmentResponse>;
          author_type: Schemas.AuthorType;
          author_user_id?: (null | Schemas.UserId) | undefined;
          author_webhook_id?: (null | Schemas.WebhookId) | undefined;
          channel_id: Schemas.ChannelId;
          components?: (Array<Schemas.Component> | null) | undefined;
          content: string;
          created_at: string;
          edited_at?: (string | null) | undefined;
          id: Schemas.MessageId;
          mention_channel_ids: Array<Schemas.ChannelId>;
          mention_everyone: boolean;
          mention_role_ids: Array<Schemas.RoleId>;
          mention_user_ids: Array<Schemas.UserId>;
          organization_id: Schemas.OrganizationId;
          reactions: Array<Schemas.ReactionCountResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type patch_CorrectEmployeeCostBasis = {
    method: "PATCH";
    path: "/api/v1/cost-bases/{cost_basis_id}";
    requestFormat: "json";
    parameters: {
      path: { cost_basis_id: string };

      body: Schemas.CorrectEmployeeCostBasisRequest;
    };
    responses: {
      200: {
        data: {
          created_at: string;
          effective_from: string;
          effective_hourly_rate_cents?: (number | null) | undefined;
          effective_to?: (string | null) | undefined;
          employee_id: Schemas.EmployeeId;
          hourly_rate_cents?: (number | null) | undefined;
          id: Schemas.EmployeeCostBasisId;
          is_salaried: boolean;
          monthly_cost_cents?: (number | null) | undefined;
          organization_id: Schemas.OrganizationId;
          updated_at: string;
          weekly_contract_minutes: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_GetCustomerContact = {
    method: "GET";
    path: "/api/v1/customer-contacts/{customer_contact_id}";
    requestFormat: "json";
    parameters: {
      path: { customer_contact_id: string };
    };
    responses: {
      200: {
        data: {
          created_at: string;
          customer_id: Schemas.CustomerId;
          email?: (string | null) | undefined;
          first_name: string;
          id: Schemas.CustomerContactId;
          is_primary: boolean;
          last_name: string;
          mobile?: (string | null) | undefined;
          phone?: (string | null) | undefined;
          role?: (string | null) | undefined;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type delete_DeleteCustomerContact = {
    method: "DELETE";
    path: "/api/v1/customer-contacts/{customer_contact_id}";
    requestFormat: "json";
    parameters: {
      path: { customer_contact_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type patch_UpdateCustomerContact = {
    method: "PATCH";
    path: "/api/v1/customer-contacts/{customer_contact_id}";
    requestFormat: "json";
    parameters: {
      path: { customer_contact_id: string };

      body: Schemas.UpdateCustomerContactRequest;
    };
    responses: {
      200: {
        data: {
          created_at: string;
          customer_id: Schemas.CustomerId;
          email?: (string | null) | undefined;
          first_name: string;
          id: Schemas.CustomerContactId;
          is_primary: boolean;
          last_name: string;
          mobile?: (string | null) | undefined;
          phone?: (string | null) | undefined;
          role?: (string | null) | undefined;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_GetCustomerContext = {
    method: "GET";
    path: "/api/v1/customer-contexts/{customer_context_id}";
    requestFormat: "json";
    parameters: {
      path: { customer_context_id: string };
    };
    responses: {
      200: {
        data: {
          address_line?: (string | null) | undefined;
          address_line2?: (string | null) | undefined;
          city?: (string | null) | undefined;
          country: string;
          created_at: string;
          customer_id: Schemas.CustomerId;
          id: Schemas.CustomerContextId;
          kind: Schemas.AddressKind;
          label: string;
          photo_key?: (string | null) | undefined;
          postal_code?: (string | null) | undefined;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type delete_DeleteCustomerContext = {
    method: "DELETE";
    path: "/api/v1/customer-contexts/{customer_context_id}";
    requestFormat: "json";
    parameters: {
      path: { customer_context_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type patch_UpdateCustomerContext = {
    method: "PATCH";
    path: "/api/v1/customer-contexts/{customer_context_id}";
    requestFormat: "json";
    parameters: {
      path: { customer_context_id: string };

      body: Schemas.UpdateCustomerContextRequest;
    };
    responses: {
      200: {
        data: {
          address_line?: (string | null) | undefined;
          address_line2?: (string | null) | undefined;
          city?: (string | null) | undefined;
          country: string;
          created_at: string;
          customer_id: Schemas.CustomerId;
          id: Schemas.CustomerContextId;
          kind: Schemas.AddressKind;
          label: string;
          photo_key?: (string | null) | undefined;
          postal_code?: (string | null) | undefined;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_GetCustomer = {
    method: "GET";
    path: "/api/v1/customers/{customer_id}";
    requestFormat: "json";
    parameters: {
      path: { customer_id: string };
    };
    responses: {
      200: {
        data: {
          archived_at?: (string | null) | undefined;
          billing_address_id?: (null | Schemas.CustomerContextId) | undefined;
          billing_contact_id?: (null | Schemas.CustomerContactId) | undefined;
          converted_at?: (string | null) | undefined;
          created_at: string;
          email?: (string | null) | undefined;
          id: Schemas.CustomerId;
          kind: Schemas.CustomerKind;
          lead_source?: (null | Schemas.LeadSource) | undefined;
          name: string;
          notes?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          owner_member_id?: (null | Schemas.MemberId) | undefined;
          payment_terms_days?: (number | null) | undefined;
          phone?: (string | null) | undefined;
          registration_number?: (string | null) | undefined;
          status: Schemas.CustomerStatus;
          updated_at: string;
          vat_number?: (string | null) | undefined;
          website?: (string | null) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type delete_DeleteCustomer = {
    method: "DELETE";
    path: "/api/v1/customers/{customer_id}";
    requestFormat: "json";
    parameters: {
      path: { customer_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type patch_UpdateCustomer = {
    method: "PATCH";
    path: "/api/v1/customers/{customer_id}";
    requestFormat: "json";
    parameters: {
      path: { customer_id: string };

      body: Schemas.UpdateCustomerRequest;
    };
    responses: {
      200: {
        data: {
          archived_at?: (string | null) | undefined;
          billing_address_id?: (null | Schemas.CustomerContextId) | undefined;
          billing_contact_id?: (null | Schemas.CustomerContactId) | undefined;
          converted_at?: (string | null) | undefined;
          created_at: string;
          email?: (string | null) | undefined;
          id: Schemas.CustomerId;
          kind: Schemas.CustomerKind;
          lead_source?: (null | Schemas.LeadSource) | undefined;
          name: string;
          notes?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          owner_member_id?: (null | Schemas.MemberId) | undefined;
          payment_terms_days?: (number | null) | undefined;
          phone?: (string | null) | undefined;
          registration_number?: (string | null) | undefined;
          status: Schemas.CustomerStatus;
          updated_at: string;
          vat_number?: (string | null) | undefined;
          website?: (string | null) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
      422: unknown;
    };
  };
  export type get_ListCustomerContacts = {
    method: "GET";
    path: "/api/v1/customers/{customer_id}/contacts";
    requestFormat: "json";
    parameters: {
      query: Partial<{ page: number; per_page: number }>;
      path: { customer_id: string };
    };
    responses: {
      200: {
        data: Array<{
          created_at: string;
          customer_id: Schemas.CustomerId;
          email?: (string | null) | undefined;
          first_name: string;
          id: Schemas.CustomerContactId;
          is_primary: boolean;
          last_name: string;
          mobile?: (string | null) | undefined;
          phone?: (string | null) | undefined;
          role?: (string | null) | undefined;
          updated_at: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type post_CreateCustomerContact = {
    method: "POST";
    path: "/api/v1/customers/{customer_id}/contacts";
    requestFormat: "json";
    parameters: {
      path: { customer_id: string };

      body: Schemas.CreateCustomerContactRequest;
    };
    responses: {
      201: {
        data: {
          created_at: string;
          customer_id: Schemas.CustomerId;
          email?: (string | null) | undefined;
          first_name: string;
          id: Schemas.CustomerContactId;
          is_primary: boolean;
          last_name: string;
          mobile?: (string | null) | undefined;
          phone?: (string | null) | undefined;
          role?: (string | null) | undefined;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_ListCustomerContexts = {
    method: "GET";
    path: "/api/v1/customers/{customer_id}/customer-contexts";
    requestFormat: "json";
    parameters: {
      query: Partial<{ page: number; per_page: number }>;
      path: { customer_id: string };
    };
    responses: {
      200: {
        data: Array<{
          address_line?: (string | null) | undefined;
          address_line2?: (string | null) | undefined;
          city?: (string | null) | undefined;
          country: string;
          created_at: string;
          customer_id: Schemas.CustomerId;
          id: Schemas.CustomerContextId;
          kind: Schemas.AddressKind;
          label: string;
          photo_key?: (string | null) | undefined;
          postal_code?: (string | null) | undefined;
          updated_at: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type post_CreateCustomerContext = {
    method: "POST";
    path: "/api/v1/customers/{customer_id}/customer-contexts";
    requestFormat: "json";
    parameters: {
      path: { customer_id: string };

      body: Schemas.CreateCustomerContextRequest;
    };
    responses: {
      201: {
        data: {
          address_line?: (string | null) | undefined;
          address_line2?: (string | null) | undefined;
          city?: (string | null) | undefined;
          country: string;
          created_at: string;
          customer_id: Schemas.CustomerId;
          id: Schemas.CustomerContextId;
          kind: Schemas.AddressKind;
          label: string;
          photo_key?: (string | null) | undefined;
          postal_code?: (string | null) | undefined;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_GetDeal = {
    method: "GET";
    path: "/api/v1/deals/{deal_id}";
    requestFormat: "json";
    parameters: {
      path: { deal_id: string };
    };
    responses: {
      200: {
        data: {
          amount_cents: number;
          closed_at?: (string | null) | undefined;
          created_at: string;
          customer_id: Schemas.CustomerId;
          expected_close_date?: (string | null) | undefined;
          id: Schemas.DealId;
          lost_reason?: (string | null) | undefined;
          notes?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          owner_member_id?: (null | Schemas.MemberId) | undefined;
          quote_count: number;
          source?: (null | Schemas.LeadSource) | undefined;
          stage: Schemas.DealStageName;
          stage_changed_at: string;
          title: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type delete_DeleteDeal = {
    method: "DELETE";
    path: "/api/v1/deals/{deal_id}";
    requestFormat: "json";
    parameters: {
      path: { deal_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown; 409: unknown };
  };
  export type patch_UpdateDeal = {
    method: "PATCH";
    path: "/api/v1/deals/{deal_id}";
    requestFormat: "json";
    parameters: {
      path: { deal_id: string };

      body: Schemas.UpdateDealRequest;
    };
    responses: {
      200: {
        data: {
          amount_cents: number;
          closed_at?: (string | null) | undefined;
          created_at: string;
          customer_id: Schemas.CustomerId;
          expected_close_date?: (string | null) | undefined;
          id: Schemas.DealId;
          lost_reason?: (string | null) | undefined;
          notes?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          owner_member_id?: (null | Schemas.MemberId) | undefined;
          quote_count: number;
          source?: (null | Schemas.LeadSource) | undefined;
          stage: Schemas.DealStageName;
          stage_changed_at: string;
          title: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      422: unknown;
    };
  };
  export type post_MoveDealStage = {
    method: "POST";
    path: "/api/v1/deals/{deal_id}/stage";
    requestFormat: "json";
    parameters: {
      path: { deal_id: string };

      body: Schemas.MoveDealRequest;
    };
    responses: {
      200: {
        data: {
          amount_cents: number;
          closed_at?: (string | null) | undefined;
          created_at: string;
          customer_id: Schemas.CustomerId;
          expected_close_date?: (string | null) | undefined;
          id: Schemas.DealId;
          lost_reason?: (string | null) | undefined;
          notes?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          owner_member_id?: (null | Schemas.MemberId) | undefined;
          quote_count: number;
          source?: (null | Schemas.LeadSource) | undefined;
          stage: Schemas.DealStageName;
          stage_changed_at: string;
          title: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      422: unknown;
    };
  };
  export type get_ListEmployeeCostBases = {
    method: "GET";
    path: "/api/v1/employees/{employee_id}/cost-bases";
    requestFormat: "json";
    parameters: {
      path: { employee_id: string };
    };
    responses: {
      200: {
        data: Array<{
          created_at: string;
          effective_from: string;
          effective_hourly_rate_cents?: (number | null) | undefined;
          effective_to?: (string | null) | undefined;
          employee_id: Schemas.EmployeeId;
          hourly_rate_cents?: (number | null) | undefined;
          id: Schemas.EmployeeCostBasisId;
          is_salaried: boolean;
          monthly_cost_cents?: (number | null) | undefined;
          organization_id: Schemas.OrganizationId;
          updated_at: string;
          weekly_contract_minutes: number;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type post_SetEmployeeCostBasis = {
    method: "POST";
    path: "/api/v1/employees/{employee_id}/cost-bases";
    requestFormat: "json";
    parameters: {
      path: { employee_id: string };

      body: Schemas.SetEmployeeCostBasisRequest;
    };
    responses: {
      201: {
        data: {
          created_at: string;
          effective_from: string;
          effective_hourly_rate_cents?: (number | null) | undefined;
          effective_to?: (string | null) | undefined;
          employee_id: Schemas.EmployeeId;
          hourly_rate_cents?: (number | null) | undefined;
          id: Schemas.EmployeeCostBasisId;
          is_salaried: boolean;
          monthly_cost_cents?: (number | null) | undefined;
          organization_id: Schemas.OrganizationId;
          updated_at: string;
          weekly_contract_minutes: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_GetEquipment = {
    method: "GET";
    path: "/api/v1/equipment/{equipment_id}";
    requestFormat: "json";
    parameters: {
      path: { equipment_id: string };
    };
    responses: {
      200: {
        data: {
          created_at: string;
          hourly_rate_cents: number;
          id: Schemas.EquipmentId;
          name: string;
          organization_id: Schemas.OrganizationId;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type delete_DeleteEquipment = {
    method: "DELETE";
    path: "/api/v1/equipment/{equipment_id}";
    requestFormat: "json";
    parameters: {
      path: { equipment_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type patch_UpdateEquipment = {
    method: "PATCH";
    path: "/api/v1/equipment/{equipment_id}";
    requestFormat: "json";
    parameters: {
      path: { equipment_id: string };

      body: Schemas.UpdateEquipmentRequest;
    };
    responses: {
      200: {
        data: {
          created_at: string;
          hourly_rate_cents: number;
          id: Schemas.EquipmentId;
          name: string;
          organization_id: Schemas.OrganizationId;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type delete_DeleteExpenseCategory = {
    method: "DELETE";
    path: "/api/v1/expense-categories/{category_id}";
    requestFormat: "json";
    parameters: {
      path: { category_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown; 409: unknown };
  };
  export type patch_UpdateExpenseCategory = {
    method: "PATCH";
    path: "/api/v1/expense-categories/{category_id}";
    requestFormat: "json";
    parameters: {
      path: { category_id: string };

      body: Schemas.ExpenseCategoryRequest;
    };
    responses: {
      200: {
        data: {
          created_at: string;
          hue: number;
          id: Schemas.ExpenseCategoryId;
          name: string;
          organization_id: Schemas.OrganizationId;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
      422: unknown;
    };
  };
  export type get_GetExpense = {
    method: "GET";
    path: "/api/v1/expenses/{expense_id}";
    requestFormat: "json";
    parameters: {
      path: { expense_id: string };
    };
    responses: {
      200: {
        data: {
          advanced_by_member_id?: (null | Schemas.MemberId) | undefined;
          amount_ht_cents: number;
          category_id: Schemas.ExpenseCategoryId;
          created_at: string;
          customer_id?: (null | Schemas.CustomerId) | undefined;
          customer_name?: (string | null) | undefined;
          date: string;
          days_late: number;
          display_state: Schemas.ExpenseDisplayState;
          due_date?: (string | null) | undefined;
          id: Schemas.ExpenseId;
          label: string;
          method?: (null | Schemas.PaymentMethod) | undefined;
          note?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          paid_on?: (string | null) | undefined;
          proof_file_key?: (string | null) | undefined;
          reference?: (string | null) | undefined;
          reimbursed_on?: (string | null) | undefined;
          status: Schemas.ExpenseStatus;
          supplier_id?: (null | Schemas.SupplierId) | undefined;
          supplier_name?: (string | null) | undefined;
          total_cents: number;
          updated_at: string;
          vat_cents: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type delete_DeleteExpense = {
    method: "DELETE";
    path: "/api/v1/expenses/{expense_id}";
    requestFormat: "json";
    parameters: {
      path: { expense_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type patch_UpdateExpense = {
    method: "PATCH";
    path: "/api/v1/expenses/{expense_id}";
    requestFormat: "json";
    parameters: {
      path: { expense_id: string };

      body: Schemas.UpdateExpenseRequest;
    };
    responses: {
      200: {
        data: {
          advanced_by_member_id?: (null | Schemas.MemberId) | undefined;
          amount_ht_cents: number;
          category_id: Schemas.ExpenseCategoryId;
          created_at: string;
          customer_id?: (null | Schemas.CustomerId) | undefined;
          customer_name?: (string | null) | undefined;
          date: string;
          days_late: number;
          display_state: Schemas.ExpenseDisplayState;
          due_date?: (string | null) | undefined;
          id: Schemas.ExpenseId;
          label: string;
          method?: (null | Schemas.PaymentMethod) | undefined;
          note?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          paid_on?: (string | null) | undefined;
          proof_file_key?: (string | null) | undefined;
          reference?: (string | null) | undefined;
          reimbursed_on?: (string | null) | undefined;
          status: Schemas.ExpenseStatus;
          supplier_id?: (null | Schemas.SupplierId) | undefined;
          supplier_name?: (string | null) | undefined;
          total_cents: number;
          updated_at: string;
          vat_cents: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      422: unknown;
    };
  };
  export type post_PayExpense = {
    method: "POST";
    path: "/api/v1/expenses/{expense_id}/pay";
    requestFormat: "json";
    parameters: {
      path: { expense_id: string };

      body: Schemas.PayExpenseRequest;
    };
    responses: {
      200: {
        data: {
          advanced_by_member_id?: (null | Schemas.MemberId) | undefined;
          amount_ht_cents: number;
          category_id: Schemas.ExpenseCategoryId;
          created_at: string;
          customer_id?: (null | Schemas.CustomerId) | undefined;
          customer_name?: (string | null) | undefined;
          date: string;
          days_late: number;
          display_state: Schemas.ExpenseDisplayState;
          due_date?: (string | null) | undefined;
          id: Schemas.ExpenseId;
          label: string;
          method?: (null | Schemas.PaymentMethod) | undefined;
          note?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          paid_on?: (string | null) | undefined;
          proof_file_key?: (string | null) | undefined;
          reference?: (string | null) | undefined;
          reimbursed_on?: (string | null) | undefined;
          status: Schemas.ExpenseStatus;
          supplier_id?: (null | Schemas.SupplierId) | undefined;
          supplier_name?: (string | null) | undefined;
          total_cents: number;
          updated_at: string;
          vat_cents: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type post_ReimburseExpense = {
    method: "POST";
    path: "/api/v1/expenses/{expense_id}/reimburse";
    requestFormat: "json";
    parameters: {
      path: { expense_id: string };
    };
    responses: {
      200: {
        data: {
          advanced_by_member_id?: (null | Schemas.MemberId) | undefined;
          amount_ht_cents: number;
          category_id: Schemas.ExpenseCategoryId;
          created_at: string;
          customer_id?: (null | Schemas.CustomerId) | undefined;
          customer_name?: (string | null) | undefined;
          date: string;
          days_late: number;
          display_state: Schemas.ExpenseDisplayState;
          due_date?: (string | null) | undefined;
          id: Schemas.ExpenseId;
          label: string;
          method?: (null | Schemas.PaymentMethod) | undefined;
          note?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          paid_on?: (string | null) | undefined;
          proof_file_key?: (string | null) | undefined;
          reference?: (string | null) | undefined;
          reimbursed_on?: (string | null) | undefined;
          status: Schemas.ExpenseStatus;
          supplier_id?: (null | Schemas.SupplierId) | undefined;
          supplier_name?: (string | null) | undefined;
          total_cents: number;
          updated_at: string;
          vat_cents: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type delete_WithdrawAssignmentReport = {
    method: "DELETE";
    path: "/api/v1/field/assignment-reports/{assignment_report_id}";
    requestFormat: "json";
    parameters: {
      path: { assignment_report_id: string };
    };
    responses: { 204: unknown; 401: unknown; 404: unknown; 409: unknown };
  };
  export type patch_AmendAssignmentReport = {
    method: "PATCH";
    path: "/api/v1/field/assignment-reports/{assignment_report_id}";
    requestFormat: "json";
    parameters: {
      path: { assignment_report_id: string };

      body: Schemas.AmendAssignmentReportRequest;
    };
    responses: {
      200: {
        data: {
          comment?: (string | null) | undefined;
          created_at: string;
          id: Schemas.AssignmentReportId;
          organization_id: Schemas.OrganizationId;
          reported_by: Schemas.MemberId;
          reported_minutes: number;
          resolution: Schemas.AssignmentReportResolution;
          resolution_note?: (string | null) | undefined;
          resolved_at?: (string | null) | undefined;
          resolved_by?: (null | Schemas.MemberId) | undefined;
          task_assignment_id: Schemas.TaskAssignmentId;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type post_AttachTimeEntryPhoto = {
    method: "POST";
    path: "/api/v1/field/time-entries/{time_entry_id}/photos";
    requestFormat: "json";
    parameters: {
      path: { time_entry_id: string };

      body: Schemas.AttachPhotoRequest;
    };
    responses: {
      201: {
        data: {
          created_at: string;
          id: Schemas.TimeEntryPhotoId;
          phase: Schemas.TimeEntryPhotoPhase;
          storage_key: string;
          time_entry_id: Schemas.TimeEntryId;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type post_RecoverTimeEntry = {
    method: "POST";
    path: "/api/v1/field/time-entries/{time_entry_id}/recover";
    requestFormat: "json";
    parameters: {
      path: { time_entry_id: string };

      body: Schemas.RecoverTimeEntryRequest;
    };
    responses: {
      200: {
        data: {
          employee_id: Schemas.EmployeeId;
          ended_at?: (string | null) | undefined;
          id: Schemas.TimeEntryId;
          organization_id: Schemas.OrganizationId;
          photos: Array<Schemas.TimeEntryPhotoResponse>;
          started_at: string;
          task_id: Schemas.TaskId;
          worked_minutes?: (number | null) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type post_StopTimeEntry = {
    method: "POST";
    path: "/api/v1/field/time-entries/{time_entry_id}/stop";
    requestFormat: "json";
    parameters: {
      path: { time_entry_id: string };
    };
    responses: {
      200: {
        data: {
          employee_id: Schemas.EmployeeId;
          ended_at?: (string | null) | undefined;
          id: Schemas.TimeEntryId;
          organization_id: Schemas.OrganizationId;
          photos: Array<Schemas.TimeEntryPhotoResponse>;
          started_at: string;
          task_id: Schemas.TaskId;
          worked_minutes?: (number | null) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type post_UploadFile = {
    method: "POST";
    path: "/api/v1/files";
    requestFormat: "binary";
    parameters: {
      query: Partial<{ folder: string }>;

      body: Array<number>;
    };
    responses: {
      201: {
        data: { key: string; mime_type: string; size_bytes: number };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      413: unknown;
      500: unknown;
    };
  };
  export type get_GetFileUrl = {
    method: "GET";
    path: "/api/v1/files/url";
    requestFormat: "json";
    parameters: {
      query: { key: string };
    };
    responses: {
      200: { data: { expires_at: string; url: string }; pagination?: (null | Schemas.PaginationMetadata) | undefined };
      401: unknown;
      409: unknown;
    };
  };
  export type delete_RevokeInvitation = {
    method: "DELETE";
    path: "/api/v1/invitations/{invitation_id}";
    requestFormat: "json";
    parameters: {
      path: { invitation_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown; 409: unknown };
  };
  export type post_AcceptInvitation = {
    method: "POST";
    path: "/api/v1/invitations/{token}/accept";
    requestFormat: "json";
    parameters: {
      path: { token: string };
    };
    responses: {
      200: {
        data: {
          account?: (null | Schemas.MemberAccountResponse) | undefined;
          created_at: string;
          display_name: string;
          first_name?: (string | null) | undefined;
          id: Schemas.MemberId;
          joined_at?: (string | null) | undefined;
          last_name: string;
          organization_id: Schemas.OrganizationId;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type delete_DeleteInvoicePayment = {
    method: "DELETE";
    path: "/api/v1/invoice-payments/{invoice_payment_id}";
    requestFormat: "json";
    parameters: {
      path: { invoice_payment_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type get_GetInvoice = {
    method: "GET";
    path: "/api/v1/invoices/{invoice_id}";
    requestFormat: "json";
    parameters: {
      path: { invoice_id: string };
    };
    responses: {
      200: {
        data: {
          balance_cents: number;
          created_at: string;
          credited_cents: number;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          delivery_address?: (null | Schemas.DeliveryAddressResponse) | undefined;
          display_state: Schemas.InvoiceDisplayState;
          due_at?: (string | null) | undefined;
          due_on?: (string | null) | undefined;
          events: Array<Schemas.InvoiceEventResponse>;
          full_cancellation: boolean;
          gross_cents: number;
          id: Schemas.InvoiceId;
          internal_note?: (string | null) | undefined;
          issued_at?: (string | null) | undefined;
          issued_on?: (string | null) | undefined;
          kind: Schemas.InvoiceKind;
          legal_notice: string;
          lines: Array<Schemas.InvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number?: (string | null) | undefined;
          operation_nature?: (null | Schemas.OperationNature) | undefined;
          organization_id: Schemas.OrganizationId;
          paid_cents: number;
          payment_terms_days: number;
          project_id?: (null | Schemas.ProjectId) | undefined;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          reason?: (string | null) | undefined;
          sent_at?: (string | null) | undefined;
          source_invoice_id?: (null | Schemas.InvoiceId) | undefined;
          status: Schemas.InvoiceStatus;
          title: string;
          updated_at: string;
          vat_breakdown: Array<Schemas.InvoiceVatBreakdownLineResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type delete_DeleteInvoice = {
    method: "DELETE";
    path: "/api/v1/invoices/{invoice_id}";
    requestFormat: "json";
    parameters: {
      path: { invoice_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown; 409: unknown };
  };
  export type patch_UpdateInvoice = {
    method: "PATCH";
    path: "/api/v1/invoices/{invoice_id}";
    requestFormat: "json";
    parameters: {
      path: { invoice_id: string };

      body: Schemas.UpdateInvoiceRequest;
    };
    responses: {
      200: {
        data: {
          balance_cents: number;
          created_at: string;
          credited_cents: number;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          delivery_address?: (null | Schemas.DeliveryAddressResponse) | undefined;
          display_state: Schemas.InvoiceDisplayState;
          due_at?: (string | null) | undefined;
          due_on?: (string | null) | undefined;
          events: Array<Schemas.InvoiceEventResponse>;
          full_cancellation: boolean;
          gross_cents: number;
          id: Schemas.InvoiceId;
          internal_note?: (string | null) | undefined;
          issued_at?: (string | null) | undefined;
          issued_on?: (string | null) | undefined;
          kind: Schemas.InvoiceKind;
          legal_notice: string;
          lines: Array<Schemas.InvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number?: (string | null) | undefined;
          operation_nature?: (null | Schemas.OperationNature) | undefined;
          organization_id: Schemas.OrganizationId;
          paid_cents: number;
          payment_terms_days: number;
          project_id?: (null | Schemas.ProjectId) | undefined;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          reason?: (string | null) | undefined;
          sent_at?: (string | null) | undefined;
          source_invoice_id?: (null | Schemas.InvoiceId) | undefined;
          status: Schemas.InvoiceStatus;
          title: string;
          updated_at: string;
          vat_breakdown: Array<Schemas.InvoiceVatBreakdownLineResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
      422: unknown;
    };
  };
  export type get_GetInvoiceBalance = {
    method: "GET";
    path: "/api/v1/invoices/{invoice_id}/balance";
    requestFormat: "json";
    parameters: {
      path: { invoice_id: string };
    };
    responses: {
      200: {
        data: { credited_cents: number; gross_cents: number; paid_cents: number; remaining_cents: number };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type post_CancelInvoice = {
    method: "POST";
    path: "/api/v1/invoices/{invoice_id}/cancel";
    requestFormat: "json";
    parameters: {
      path: { invoice_id: string };
    };
    responses: {
      200: {
        data: {
          balance_cents: number;
          created_at: string;
          credited_cents: number;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          delivery_address?: (null | Schemas.DeliveryAddressResponse) | undefined;
          display_state: Schemas.InvoiceDisplayState;
          due_at?: (string | null) | undefined;
          due_on?: (string | null) | undefined;
          events: Array<Schemas.InvoiceEventResponse>;
          full_cancellation: boolean;
          gross_cents: number;
          id: Schemas.InvoiceId;
          internal_note?: (string | null) | undefined;
          issued_at?: (string | null) | undefined;
          issued_on?: (string | null) | undefined;
          kind: Schemas.InvoiceKind;
          legal_notice: string;
          lines: Array<Schemas.InvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number?: (string | null) | undefined;
          operation_nature?: (null | Schemas.OperationNature) | undefined;
          organization_id: Schemas.OrganizationId;
          paid_cents: number;
          payment_terms_days: number;
          project_id?: (null | Schemas.ProjectId) | undefined;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          reason?: (string | null) | undefined;
          sent_at?: (string | null) | undefined;
          source_invoice_id?: (null | Schemas.InvoiceId) | undefined;
          status: Schemas.InvoiceStatus;
          title: string;
          updated_at: string;
          vat_breakdown: Array<Schemas.InvoiceVatBreakdownLineResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_ListInvoiceCreditNotes = {
    method: "GET";
    path: "/api/v1/invoices/{invoice_id}/credit-notes";
    requestFormat: "json";
    parameters: {
      path: { invoice_id: string };
    };
    responses: {
      200: {
        data: Array<{
          balance_cents: number;
          created_at: string;
          credited_cents: number;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          delivery_address?: (null | Schemas.DeliveryAddressResponse) | undefined;
          display_state: Schemas.InvoiceDisplayState;
          due_at?: (string | null) | undefined;
          due_on?: (string | null) | undefined;
          events: Array<Schemas.InvoiceEventResponse>;
          full_cancellation: boolean;
          gross_cents: number;
          id: Schemas.InvoiceId;
          internal_note?: (string | null) | undefined;
          issued_at?: (string | null) | undefined;
          issued_on?: (string | null) | undefined;
          kind: Schemas.InvoiceKind;
          legal_notice: string;
          lines: Array<Schemas.InvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number?: (string | null) | undefined;
          operation_nature?: (null | Schemas.OperationNature) | undefined;
          organization_id: Schemas.OrganizationId;
          paid_cents: number;
          payment_terms_days: number;
          project_id?: (null | Schemas.ProjectId) | undefined;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          reason?: (string | null) | undefined;
          sent_at?: (string | null) | undefined;
          source_invoice_id?: (null | Schemas.InvoiceId) | undefined;
          status: Schemas.InvoiceStatus;
          title: string;
          updated_at: string;
          vat_breakdown: Array<Schemas.InvoiceVatBreakdownLineResponse>;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type post_IssueCreditNote = {
    method: "POST";
    path: "/api/v1/invoices/{invoice_id}/credit-notes";
    requestFormat: "json";
    parameters: {
      path: { invoice_id: string };

      body: Schemas.IssueCreditNoteRequest;
    };
    responses: {
      201: {
        data: {
          balance_cents: number;
          created_at: string;
          credited_cents: number;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          delivery_address?: (null | Schemas.DeliveryAddressResponse) | undefined;
          display_state: Schemas.InvoiceDisplayState;
          due_at?: (string | null) | undefined;
          due_on?: (string | null) | undefined;
          events: Array<Schemas.InvoiceEventResponse>;
          full_cancellation: boolean;
          gross_cents: number;
          id: Schemas.InvoiceId;
          internal_note?: (string | null) | undefined;
          issued_at?: (string | null) | undefined;
          issued_on?: (string | null) | undefined;
          kind: Schemas.InvoiceKind;
          legal_notice: string;
          lines: Array<Schemas.InvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number?: (string | null) | undefined;
          operation_nature?: (null | Schemas.OperationNature) | undefined;
          organization_id: Schemas.OrganizationId;
          paid_cents: number;
          payment_terms_days: number;
          project_id?: (null | Schemas.ProjectId) | undefined;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          reason?: (string | null) | undefined;
          sent_at?: (string | null) | undefined;
          source_invoice_id?: (null | Schemas.InvoiceId) | undefined;
          status: Schemas.InvoiceStatus;
          title: string;
          updated_at: string;
          vat_breakdown: Array<Schemas.InvoiceVatBreakdownLineResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
      422: unknown;
    };
  };
  export type post_IssueInvoice = {
    method: "POST";
    path: "/api/v1/invoices/{invoice_id}/issue";
    requestFormat: "json";
    parameters: {
      path: { invoice_id: string };

      body: Schemas.IssueInvoiceRequest;
    };
    responses: {
      200: {
        data: {
          balance_cents: number;
          created_at: string;
          credited_cents: number;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          delivery_address?: (null | Schemas.DeliveryAddressResponse) | undefined;
          display_state: Schemas.InvoiceDisplayState;
          due_at?: (string | null) | undefined;
          due_on?: (string | null) | undefined;
          events: Array<Schemas.InvoiceEventResponse>;
          full_cancellation: boolean;
          gross_cents: number;
          id: Schemas.InvoiceId;
          internal_note?: (string | null) | undefined;
          issued_at?: (string | null) | undefined;
          issued_on?: (string | null) | undefined;
          kind: Schemas.InvoiceKind;
          legal_notice: string;
          lines: Array<Schemas.InvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number?: (string | null) | undefined;
          operation_nature?: (null | Schemas.OperationNature) | undefined;
          organization_id: Schemas.OrganizationId;
          paid_cents: number;
          payment_terms_days: number;
          project_id?: (null | Schemas.ProjectId) | undefined;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          reason?: (string | null) | undefined;
          sent_at?: (string | null) | undefined;
          source_invoice_id?: (null | Schemas.InvoiceId) | undefined;
          status: Schemas.InvoiceStatus;
          title: string;
          updated_at: string;
          vat_breakdown: Array<Schemas.InvoiceVatBreakdownLineResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
      422: unknown;
    };
  };
  export type post_NoteInvoice = {
    method: "POST";
    path: "/api/v1/invoices/{invoice_id}/note";
    requestFormat: "json";
    parameters: {
      path: { invoice_id: string };

      body: Schemas.NoteInvoiceRequest;
    };
    responses: {
      200: {
        data: {
          balance_cents: number;
          created_at: string;
          credited_cents: number;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          delivery_address?: (null | Schemas.DeliveryAddressResponse) | undefined;
          display_state: Schemas.InvoiceDisplayState;
          due_at?: (string | null) | undefined;
          due_on?: (string | null) | undefined;
          events: Array<Schemas.InvoiceEventResponse>;
          full_cancellation: boolean;
          gross_cents: number;
          id: Schemas.InvoiceId;
          internal_note?: (string | null) | undefined;
          issued_at?: (string | null) | undefined;
          issued_on?: (string | null) | undefined;
          kind: Schemas.InvoiceKind;
          legal_notice: string;
          lines: Array<Schemas.InvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number?: (string | null) | undefined;
          operation_nature?: (null | Schemas.OperationNature) | undefined;
          organization_id: Schemas.OrganizationId;
          paid_cents: number;
          payment_terms_days: number;
          project_id?: (null | Schemas.ProjectId) | undefined;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          reason?: (string | null) | undefined;
          sent_at?: (string | null) | undefined;
          source_invoice_id?: (null | Schemas.InvoiceId) | undefined;
          status: Schemas.InvoiceStatus;
          title: string;
          updated_at: string;
          vat_breakdown: Array<Schemas.InvoiceVatBreakdownLineResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type get_ListInvoicePayments = {
    method: "GET";
    path: "/api/v1/invoices/{invoice_id}/payments";
    requestFormat: "json";
    parameters: {
      path: { invoice_id: string };
    };
    responses: {
      200: {
        data: Array<{
          amount_cents: number;
          created_at: string;
          deleted_at?: (string | null) | undefined;
          deleted_by?: (null | Schemas.UserId) | undefined;
          id: Schemas.InvoicePaymentId;
          invoice_id: Schemas.InvoiceId;
          method: Schemas.PaymentMethod;
          note?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          paid_on: string;
          recorded_by: Schemas.UserId;
          reference?: (string | null) | undefined;
          updated_at: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type post_RecordInvoicePayment = {
    method: "POST";
    path: "/api/v1/invoices/{invoice_id}/payments";
    requestFormat: "json";
    parameters: {
      path: { invoice_id: string };

      body: Schemas.RecordInvoicePaymentRequest;
    };
    responses: {
      201: {
        data: {
          amount_cents: number;
          created_at: string;
          deleted_at?: (string | null) | undefined;
          deleted_by?: (null | Schemas.UserId) | undefined;
          id: Schemas.InvoicePaymentId;
          invoice_id: Schemas.InvoiceId;
          method: Schemas.PaymentMethod;
          note?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          paid_on: string;
          recorded_by: Schemas.UserId;
          reference?: (string | null) | undefined;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
      422: unknown;
    };
  };
  export type get_ExportInvoicePdf = {
    method: "GET";
    path: "/api/v1/invoices/{invoice_id}/pdf";
    requestFormat: "json";
    parameters: {
      path: { invoice_id: string; format: "pdf" | "facturx" };
    };
    responses: { 200: unknown; 401: unknown; 403: unknown; 404: unknown; 409: unknown; 422: unknown };
  };
  export type post_RemindInvoice = {
    method: "POST";
    path: "/api/v1/invoices/{invoice_id}/remind";
    requestFormat: "json";
    parameters: {
      path: { invoice_id: string };
    };
    responses: {
      200: {
        data: {
          balance_cents: number;
          created_at: string;
          credited_cents: number;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          delivery_address?: (null | Schemas.DeliveryAddressResponse) | undefined;
          display_state: Schemas.InvoiceDisplayState;
          due_at?: (string | null) | undefined;
          due_on?: (string | null) | undefined;
          events: Array<Schemas.InvoiceEventResponse>;
          full_cancellation: boolean;
          gross_cents: number;
          id: Schemas.InvoiceId;
          internal_note?: (string | null) | undefined;
          issued_at?: (string | null) | undefined;
          issued_on?: (string | null) | undefined;
          kind: Schemas.InvoiceKind;
          legal_notice: string;
          lines: Array<Schemas.InvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number?: (string | null) | undefined;
          operation_nature?: (null | Schemas.OperationNature) | undefined;
          organization_id: Schemas.OrganizationId;
          paid_cents: number;
          payment_terms_days: number;
          project_id?: (null | Schemas.ProjectId) | undefined;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          reason?: (string | null) | undefined;
          sent_at?: (string | null) | undefined;
          source_invoice_id?: (null | Schemas.InvoiceId) | undefined;
          status: Schemas.InvoiceStatus;
          title: string;
          updated_at: string;
          vat_breakdown: Array<Schemas.InvoiceVatBreakdownLineResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type post_SendInvoice = {
    method: "POST";
    path: "/api/v1/invoices/{invoice_id}/send";
    requestFormat: "json";
    parameters: {
      path: { invoice_id: string };
    };
    responses: {
      200: {
        data: {
          balance_cents: number;
          created_at: string;
          credited_cents: number;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          delivery_address?: (null | Schemas.DeliveryAddressResponse) | undefined;
          display_state: Schemas.InvoiceDisplayState;
          due_at?: (string | null) | undefined;
          due_on?: (string | null) | undefined;
          events: Array<Schemas.InvoiceEventResponse>;
          full_cancellation: boolean;
          gross_cents: number;
          id: Schemas.InvoiceId;
          internal_note?: (string | null) | undefined;
          issued_at?: (string | null) | undefined;
          issued_on?: (string | null) | undefined;
          kind: Schemas.InvoiceKind;
          legal_notice: string;
          lines: Array<Schemas.InvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number?: (string | null) | undefined;
          operation_nature?: (null | Schemas.OperationNature) | undefined;
          organization_id: Schemas.OrganizationId;
          paid_cents: number;
          payment_terms_days: number;
          project_id?: (null | Schemas.ProjectId) | undefined;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          reason?: (string | null) | undefined;
          sent_at?: (string | null) | undefined;
          source_invoice_id?: (null | Schemas.InvoiceId) | undefined;
          status: Schemas.InvoiceStatus;
          title: string;
          updated_at: string;
          vat_breakdown: Array<Schemas.InvoiceVatBreakdownLineResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_GetMember = {
    method: "GET";
    path: "/api/v1/members/{member_id}";
    requestFormat: "json";
    parameters: {
      path: { member_id: string };
    };
    responses: {
      200: {
        data: {
          account?: (null | Schemas.MemberAccountResponse) | undefined;
          created_at: string;
          display_name: string;
          first_name?: (string | null) | undefined;
          id: Schemas.MemberId;
          joined_at?: (string | null) | undefined;
          last_name: string;
          organization_id: Schemas.OrganizationId;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type delete_DeleteMember = {
    method: "DELETE";
    path: "/api/v1/members/{member_id}";
    requestFormat: "json";
    parameters: {
      path: { member_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type patch_UpdateMember = {
    method: "PATCH";
    path: "/api/v1/members/{member_id}";
    requestFormat: "json";
    parameters: {
      path: { member_id: string };

      body: Schemas.UpdateMemberRequest;
    };
    responses: {
      200: {
        data: {
          account?: (null | Schemas.MemberAccountResponse) | undefined;
          created_at: string;
          display_name: string;
          first_name?: (string | null) | undefined;
          id: Schemas.MemberId;
          joined_at?: (string | null) | undefined;
          last_name: string;
          organization_id: Schemas.OrganizationId;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type put_UpsertEmployeeProfile = {
    method: "PUT";
    path: "/api/v1/members/{member_id}/employee-profile";
    requestFormat: "json";
    parameters: {
      path: { member_id: string };

      body: Schemas.UpsertEmployeeProfileRequest;
    };
    responses: {
      200: {
        data: {
          created_at: string;
          effective_hourly_rate_cents?: (number | null) | undefined;
          hourly_rate_cents?: (number | null) | undefined;
          id: Schemas.EmployeeId;
          is_salaried: boolean;
          member_id: Schemas.MemberId;
          monthly_cost_cents?: (number | null) | undefined;
          organization_id: Schemas.OrganizationId;
          updated_at: string;
          weekly_contract_minutes: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type delete_RemoveEmployeeProfile = {
    method: "DELETE";
    path: "/api/v1/members/{member_id}/employee-profile";
    requestFormat: "json";
    parameters: {
      path: { member_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type put_PutRhythm = {
    method: "PUT";
    path: "/api/v1/members/{member_id}/rhythm";
    requestFormat: "json";
    parameters: {
      path: { member_id: string };

      body: Schemas.PutRhythmRequest;
    };
    responses: {
      200: {
        data: {
          created_at: string;
          effective_from: string;
          effective_to?: (string | null) | undefined;
          employee_id: Schemas.EmployeeId;
          id: Schemas.EmployeeRhythmId;
          organization_id: Schemas.OrganizationId;
          slots: Array<Schemas.RhythmSlotResponse>;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      409: unknown;
    };
  };
  export type get_ListMemberRoles = {
    method: "GET";
    path: "/api/v1/members/{member_id}/roles";
    requestFormat: "json";
    parameters: {
      path: { member_id: string };
    };
    responses: {
      200: { data: { role_ids: Array<Schemas.RoleId> }; pagination?: (null | Schemas.PaginationMetadata) | undefined };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type post_AssignRole = {
    method: "POST";
    path: "/api/v1/members/{member_id}/roles";
    requestFormat: "json";
    parameters: {
      path: { member_id: string };

      body: Schemas.AssignRoleRequest;
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type delete_UnassignRole = {
    method: "DELETE";
    path: "/api/v1/members/{member_id}/roles/{role_id}";
    requestFormat: "json";
    parameters: {
      path: { member_id: string; role_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type put_PutWorkSlots = {
    method: "PUT";
    path: "/api/v1/members/{member_id}/work-slots";
    requestFormat: "json";
    parameters: {
      query: { from: string; to: string };
      path: { member_id: string };

      body: Schemas.PutWorkSlotsRequest;
    };
    responses: {
      200: {
        data: Array<{
          ends_minute: number;
          id: Schemas.WorkSlotId;
          member_id: Schemas.MemberId;
          organization_id: Schemas.OrganizationId;
          starts_minute: number;
          work_date: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      409: unknown;
    };
  };
  export type get_GetWorkTime = {
    method: "GET";
    path: "/api/v1/members/{member_id}/work-time";
    requestFormat: "json";
    parameters: {
      query: { from: string; to: string };
      path: { member_id: string };
    };
    responses: {
      200: {
        data: { rhythms: Array<Schemas.RhythmResponse>; work_slots: Array<Schemas.WorkSlotResponse> };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
    };
  };
  export type get_ListOrganizations = {
    method: "GET";
    path: "/api/v1/organizations";
    requestFormat: "json";
    parameters: {
      query: Partial<{ page: number; per_page: number }>;
    };
    responses: {
      200: {
        data: Array<{
          address_city?: (string | null) | undefined;
          address_country?: (string | null) | undefined;
          address_line1?: (string | null) | undefined;
          address_line2?: (string | null) | undefined;
          address_postal_code?: (string | null) | undefined;
          contact_email?: (string | null) | undefined;
          contact_phone?: (string | null) | undefined;
          created_at: string;
          field_clock_enabled: boolean;
          id: Schemas.OrganizationId;
          insurance_mention?: (string | null) | undefined;
          legal_form?: (string | null) | undefined;
          legal_name?: (string | null) | undefined;
          mcp_enabled: boolean;
          missing_legal_identity_fields: Array<Schemas.str>;
          name: string;
          owner_id: Schemas.UserId;
          registration_number?: (string | null) | undefined;
          share_capital_cents?: (number | null) | undefined;
          slug: string;
          updated_at: string;
          vat_on_debits: boolean;
          vat_status?: (null | Schemas.VatStatusResponse) | undefined;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
    };
  };
  export type post_CreateOrganization = {
    method: "POST";
    path: "/api/v1/organizations";
    requestFormat: "json";
    parameters: {
      body: Schemas.CreateOrganizationRequest;
    };
    responses: {
      201: {
        data: {
          address_city?: (string | null) | undefined;
          address_country?: (string | null) | undefined;
          address_line1?: (string | null) | undefined;
          address_line2?: (string | null) | undefined;
          address_postal_code?: (string | null) | undefined;
          contact_email?: (string | null) | undefined;
          contact_phone?: (string | null) | undefined;
          created_at: string;
          field_clock_enabled: boolean;
          id: Schemas.OrganizationId;
          insurance_mention?: (string | null) | undefined;
          legal_form?: (string | null) | undefined;
          legal_name?: (string | null) | undefined;
          mcp_enabled: boolean;
          missing_legal_identity_fields: Array<Schemas.str>;
          name: string;
          owner_id: Schemas.UserId;
          registration_number?: (string | null) | undefined;
          share_capital_cents?: (number | null) | undefined;
          slug: string;
          updated_at: string;
          vat_on_debits: boolean;
          vat_status?: (null | Schemas.VatStatusResponse) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      409: unknown;
    };
  };
  export type get_GetOrganization = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: {
          address_city?: (string | null) | undefined;
          address_country?: (string | null) | undefined;
          address_line1?: (string | null) | undefined;
          address_line2?: (string | null) | undefined;
          address_postal_code?: (string | null) | undefined;
          contact_email?: (string | null) | undefined;
          contact_phone?: (string | null) | undefined;
          created_at: string;
          field_clock_enabled: boolean;
          id: Schemas.OrganizationId;
          insurance_mention?: (string | null) | undefined;
          legal_form?: (string | null) | undefined;
          legal_name?: (string | null) | undefined;
          mcp_enabled: boolean;
          missing_legal_identity_fields: Array<Schemas.str>;
          name: string;
          owner_id: Schemas.UserId;
          registration_number?: (string | null) | undefined;
          share_capital_cents?: (number | null) | undefined;
          slug: string;
          updated_at: string;
          vat_on_debits: boolean;
          vat_status?: (null | Schemas.VatStatusResponse) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type delete_DeleteOrganization = {
    method: "DELETE";
    path: "/api/v1/organizations/{organization_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type patch_UpdateOrganization = {
    method: "PATCH";
    path: "/api/v1/organizations/{organization_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.UpdateOrganizationRequest;
    };
    responses: {
      200: {
        data: {
          address_city?: (string | null) | undefined;
          address_country?: (string | null) | undefined;
          address_line1?: (string | null) | undefined;
          address_line2?: (string | null) | undefined;
          address_postal_code?: (string | null) | undefined;
          contact_email?: (string | null) | undefined;
          contact_phone?: (string | null) | undefined;
          created_at: string;
          field_clock_enabled: boolean;
          id: Schemas.OrganizationId;
          insurance_mention?: (string | null) | undefined;
          legal_form?: (string | null) | undefined;
          legal_name?: (string | null) | undefined;
          mcp_enabled: boolean;
          missing_legal_identity_fields: Array<Schemas.str>;
          name: string;
          owner_id: Schemas.UserId;
          registration_number?: (string | null) | undefined;
          share_capital_cents?: (number | null) | undefined;
          slug: string;
          updated_at: string;
          vat_on_debits: boolean;
          vat_status?: (null | Schemas.VatStatusResponse) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_ListAbsenceTypes = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/absence-types";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          account_id?: (string | null) | undefined;
          archived_at?: (string | null) | undefined;
          count_mode: Schemas.CountMode;
          created_at: string;
          hue: number;
          id: Schemas.AbsenceTypeId;
          kind: Schemas.AbsenceKind;
          name: string;
          needs_approval: boolean;
          needs_proof: boolean;
          organization_id: Schemas.OrganizationId;
          paid: boolean;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateAbsenceType = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/absence-types";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.AbsenceTypeRequest;
    };
    responses: {
      201: {
        data: {
          account_id?: (string | null) | undefined;
          archived_at?: (string | null) | undefined;
          count_mode: Schemas.CountMode;
          created_at: string;
          hue: number;
          id: Schemas.AbsenceTypeId;
          kind: Schemas.AbsenceKind;
          name: string;
          needs_approval: boolean;
          needs_proof: boolean;
          organization_id: Schemas.OrganizationId;
          paid: boolean;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      422: unknown;
    };
  };
  export type put_UpdateAbsenceType = {
    method: "PUT";
    path: "/api/v1/organizations/{organization_id}/absence-types/{absence_type_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; absence_type_id: string };

      body: Schemas.AbsenceTypeRequest;
    };
    responses: {
      200: {
        data: {
          account_id?: (string | null) | undefined;
          archived_at?: (string | null) | undefined;
          count_mode: Schemas.CountMode;
          created_at: string;
          hue: number;
          id: Schemas.AbsenceTypeId;
          kind: Schemas.AbsenceKind;
          name: string;
          needs_approval: boolean;
          needs_proof: boolean;
          organization_id: Schemas.OrganizationId;
          paid: boolean;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      422: unknown;
    };
  };
  export type post_ArchiveAbsenceType = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/absence-types/{absence_type_id}/archive";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; absence_type_id: string };
    };
    responses: {
      200: {
        data: {
          account_id?: (string | null) | undefined;
          archived_at?: (string | null) | undefined;
          count_mode: Schemas.CountMode;
          created_at: string;
          hue: number;
          id: Schemas.AbsenceTypeId;
          kind: Schemas.AbsenceKind;
          name: string;
          needs_approval: boolean;
          needs_proof: boolean;
          organization_id: Schemas.OrganizationId;
          paid: boolean;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type post_RestoreAbsenceType = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/absence-types/{absence_type_id}/restore";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; absence_type_id: string };
    };
    responses: {
      200: {
        data: {
          account_id?: (string | null) | undefined;
          archived_at?: (string | null) | undefined;
          count_mode: Schemas.CountMode;
          created_at: string;
          hue: number;
          id: Schemas.AbsenceTypeId;
          kind: Schemas.AbsenceKind;
          name: string;
          needs_approval: boolean;
          needs_proof: boolean;
          organization_id: Schemas.OrganizationId;
          paid: boolean;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type get_ListAbsences = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/absences";
    requestFormat: "json";
    parameters: {
      query: Partial<{ member_id: string; page: number; per_page: number }>;
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          all_day: boolean;
          created_at: string;
          created_by_member_id?: (null | Schemas.MemberId) | undefined;
          decided_at?: (string | null) | undefined;
          decided_by_member_id?: (null | Schemas.MemberId) | undefined;
          decision_comment?: (string | null) | undefined;
          end_date: string;
          end_part: Schemas.DayPart;
          ends_at: string;
          id: Schemas.AbsenceId;
          kind: Schemas.AbsenceKind;
          member_id: Schemas.MemberId;
          note?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          proof_file_key?: (string | null) | undefined;
          start_date: string;
          start_part: Schemas.DayPart;
          starts_at: string;
          status: Schemas.AbsenceStatus;
          type_id: Schemas.AbsenceTypeId;
          updated_at: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateAbsence = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/absences";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateAbsenceRequest;
    };
    responses: {
      201: {
        data: {
          all_day: boolean;
          created_at: string;
          created_by_member_id?: (null | Schemas.MemberId) | undefined;
          decided_at?: (string | null) | undefined;
          decided_by_member_id?: (null | Schemas.MemberId) | undefined;
          decision_comment?: (string | null) | undefined;
          end_date: string;
          end_part: Schemas.DayPart;
          ends_at: string;
          id: Schemas.AbsenceId;
          kind: Schemas.AbsenceKind;
          member_id: Schemas.MemberId;
          note?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          proof_file_key?: (string | null) | undefined;
          start_date: string;
          start_part: Schemas.DayPart;
          starts_at: string;
          status: Schemas.AbsenceStatus;
          type_id: Schemas.AbsenceTypeId;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
      422: unknown;
    };
  };
  export type delete_DeleteAbsence = {
    method: "DELETE";
    path: "/api/v1/organizations/{organization_id}/absences/{absence_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; absence_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type patch_PatchAbsence = {
    method: "PATCH";
    path: "/api/v1/organizations/{organization_id}/absences/{absence_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; absence_id: string };

      body: Schemas.UpdateAbsenceRequest;
    };
    responses: {
      200: {
        data: {
          all_day: boolean;
          created_at: string;
          created_by_member_id?: (null | Schemas.MemberId) | undefined;
          decided_at?: (string | null) | undefined;
          decided_by_member_id?: (null | Schemas.MemberId) | undefined;
          decision_comment?: (string | null) | undefined;
          end_date: string;
          end_part: Schemas.DayPart;
          ends_at: string;
          id: Schemas.AbsenceId;
          kind: Schemas.AbsenceKind;
          member_id: Schemas.MemberId;
          note?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          proof_file_key?: (string | null) | undefined;
          start_date: string;
          start_part: Schemas.DayPart;
          starts_at: string;
          status: Schemas.AbsenceStatus;
          type_id: Schemas.AbsenceTypeId;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type post_ApproveAbsence = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/absences/{absence_id}/approve";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; absence_id: string };
    };
    responses: {
      200: {
        data: {
          all_day: boolean;
          created_at: string;
          created_by_member_id?: (null | Schemas.MemberId) | undefined;
          decided_at?: (string | null) | undefined;
          decided_by_member_id?: (null | Schemas.MemberId) | undefined;
          decision_comment?: (string | null) | undefined;
          end_date: string;
          end_part: Schemas.DayPart;
          ends_at: string;
          id: Schemas.AbsenceId;
          kind: Schemas.AbsenceKind;
          member_id: Schemas.MemberId;
          note?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          proof_file_key?: (string | null) | undefined;
          start_date: string;
          start_part: Schemas.DayPart;
          starts_at: string;
          status: Schemas.AbsenceStatus;
          type_id: Schemas.AbsenceTypeId;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      422: unknown;
    };
  };
  export type post_CancelAbsence = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/absences/{absence_id}/cancel";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; absence_id: string };
    };
    responses: {
      200: {
        data: {
          all_day: boolean;
          created_at: string;
          created_by_member_id?: (null | Schemas.MemberId) | undefined;
          decided_at?: (string | null) | undefined;
          decided_by_member_id?: (null | Schemas.MemberId) | undefined;
          decision_comment?: (string | null) | undefined;
          end_date: string;
          end_part: Schemas.DayPart;
          ends_at: string;
          id: Schemas.AbsenceId;
          kind: Schemas.AbsenceKind;
          member_id: Schemas.MemberId;
          note?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          proof_file_key?: (string | null) | undefined;
          start_date: string;
          start_part: Schemas.DayPart;
          starts_at: string;
          status: Schemas.AbsenceStatus;
          type_id: Schemas.AbsenceTypeId;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      422: unknown;
    };
  };
  export type post_RejectAbsence = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/absences/{absence_id}/reject";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; absence_id: string };

      body: Schemas.RejectAbsenceRequest;
    };
    responses: {
      200: {
        data: {
          all_day: boolean;
          created_at: string;
          created_by_member_id?: (null | Schemas.MemberId) | undefined;
          decided_at?: (string | null) | undefined;
          decided_by_member_id?: (null | Schemas.MemberId) | undefined;
          decision_comment?: (string | null) | undefined;
          end_date: string;
          end_part: Schemas.DayPart;
          ends_at: string;
          id: Schemas.AbsenceId;
          kind: Schemas.AbsenceKind;
          member_id: Schemas.MemberId;
          note?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          proof_file_key?: (string | null) | undefined;
          start_date: string;
          start_part: Schemas.DayPart;
          starts_at: string;
          status: Schemas.AbsenceStatus;
          type_id: Schemas.AbsenceTypeId;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      422: unknown;
    };
  };
  export type get_ListAssignmentReports = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/assignment-reports";
    requestFormat: "json";
    parameters: {
      query: Partial<{ resolution: "PENDING" | "APPLIED" | "DISMISSED"; page: number; per_page: number }>;
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          comment?: (string | null) | undefined;
          created_at: string;
          id: Schemas.AssignmentReportId;
          organization_id: Schemas.OrganizationId;
          reported_by: Schemas.MemberId;
          reported_minutes: number;
          resolution: Schemas.AssignmentReportResolution;
          resolution_note?: (string | null) | undefined;
          resolved_at?: (string | null) | undefined;
          resolved_by?: (null | Schemas.MemberId) | undefined;
          task_assignment_id: Schemas.TaskAssignmentId;
          updated_at: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type get_ListCollections = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/automation/collections";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          base_url: string;
          id: string;
          imported_at: string;
          operation_count: number;
          organization_id: Schemas.OrganizationId;
          security: Array<Schemas.SecuritySchemeDto>;
          slug: string;
          source: Schemas.CollectionSourceDto;
          title: string;
          version: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_ImportCollection = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/automation/collections";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CollectionRequest;
    };
    responses: {
      201: {
        data: {
          base_url: string;
          description: string;
          history: Array<Schemas.HistoryEntryResponse>;
          id: string;
          imported_at: string;
          operations: Array<Schemas.OperationDto>;
          organization_id: Schemas.OrganizationId;
          schemas: Record<string, unknown>;
          security: Array<Schemas.SecuritySchemeDto>;
          slug: string;
          source: Schemas.CollectionSourceDto;
          title: string;
          version: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      413: unknown;
      422: unknown;
    };
  };
  export type get_GetCollection = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/automation/collections/{collection_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; collection_id: string };
    };
    responses: {
      200: {
        data: {
          base_url: string;
          description: string;
          history: Array<Schemas.HistoryEntryResponse>;
          id: string;
          imported_at: string;
          operations: Array<Schemas.OperationDto>;
          organization_id: Schemas.OrganizationId;
          schemas: Record<string, unknown>;
          security: Array<Schemas.SecuritySchemeDto>;
          slug: string;
          source: Schemas.CollectionSourceDto;
          title: string;
          version: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type put_ReplaceCollection = {
    method: "PUT";
    path: "/api/v1/organizations/{organization_id}/automation/collections/{collection_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; collection_id: string };

      body: Schemas.CollectionRequest;
    };
    responses: {
      200: {
        data: {
          base_url: string;
          description: string;
          history: Array<Schemas.HistoryEntryResponse>;
          id: string;
          imported_at: string;
          operations: Array<Schemas.OperationDto>;
          organization_id: Schemas.OrganizationId;
          schemas: Record<string, unknown>;
          security: Array<Schemas.SecuritySchemeDto>;
          slug: string;
          source: Schemas.CollectionSourceDto;
          title: string;
          version: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      413: unknown;
      422: unknown;
    };
  };
  export type delete_DeleteCollection = {
    method: "DELETE";
    path: "/api/v1/organizations/{organization_id}/automation/collections/{collection_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; collection_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type get_ListConnectors = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/automation/connectors";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: {
          auth_schemes: Array<Schemas.AuthSchemeResponse>;
          connectors: Array<Schemas.ConnectorDescriptorResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type get_ListCredentialKinds = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/automation/credential-kinds";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          category: string;
          category_label: string;
          connectable: boolean;
          description: string;
          executable: boolean;
          fields: Array<Schemas.CredentialFieldResponse>;
          kind: string;
          label: string;
          origin_choices: Array<string>;
          supported: boolean;
          usages: Array<Schemas.CredentialUsageResponse>;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type get_ListAutomationCredentials = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/automation/credentials";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          account?: (string | null) | undefined;
          created_at: string;
          data: Record<string, unknown>;
          description: string;
          expires_at?: (string | null) | undefined;
          id: string;
          kind: string;
          name: string;
          organization_id: Schemas.OrganizationId;
          origin: string;
          secrets_set: Array<string>;
          status: Schemas.CredentialStatusResponse;
          updated_at: string;
          used_by_workflows: number;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateAutomationCredential = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/automation/credentials";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateCredentialRequest;
    };
    responses: {
      201: {
        data: Schemas.CredentialResponse & { secret: unknown };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      409: unknown;
    };
  };
  export type delete_DeleteAutomationCredential = {
    method: "DELETE";
    path: "/api/v1/organizations/{organization_id}/automation/credentials/{credential_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; credential_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type patch_UpdateAutomationCredential = {
    method: "PATCH";
    path: "/api/v1/organizations/{organization_id}/automation/credentials/{credential_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; credential_id: string };

      body: Schemas.UpdateCredentialRequest;
    };
    responses: {
      200: {
        data: {
          account?: (string | null) | undefined;
          created_at: string;
          data: Record<string, unknown>;
          description: string;
          expires_at?: (string | null) | undefined;
          id: string;
          kind: string;
          name: string;
          organization_id: Schemas.OrganizationId;
          origin: string;
          secrets_set: Array<string>;
          status: Schemas.CredentialStatusResponse;
          updated_at: string;
          used_by_workflows: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type post_DisconnectAutomationCredentialOauth = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/automation/credentials/{credential_id}/oauth/disconnect";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; credential_id: string };
    };
    responses: {
      200: {
        data: {
          account?: (string | null) | undefined;
          created_at: string;
          data: Record<string, unknown>;
          description: string;
          expires_at?: (string | null) | undefined;
          id: string;
          kind: string;
          name: string;
          organization_id: Schemas.OrganizationId;
          origin: string;
          secrets_set: Array<string>;
          status: Schemas.CredentialStatusResponse;
          updated_at: string;
          used_by_workflows: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type post_StartAutomationCredentialOauth = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/automation/credentials/{credential_id}/oauth/start";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; credential_id: string };

      body: Schemas.StartOauthRequest;
    };
    responses: {
      200: { data: { authorization_url: string }; pagination?: (null | Schemas.PaginationMetadata) | undefined };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type post_RotateAutomationCredential = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/automation/credentials/{credential_id}/rotate";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; credential_id: string };
    };
    responses: {
      200: {
        data: Schemas.CredentialResponse & { secret: unknown };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_ListAutomationEvents = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/automation/events";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{ label: string; name: string; payload_example: unknown; subject_kind: string; version: number }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_EvaluateExpression = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/automation/expressions/evaluate";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.EvaluateExpressionRequest;
    };
    responses: {
      200: { data: { value: unknown }; pagination?: (null | Schemas.PaginationMetadata) | undefined };
      401: unknown;
      403: unknown;
      422: unknown;
    };
  };
  export type get_ListWorkflowFolders = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/automation/folders";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{ created_at: string; id: string; name: string; parent_id?: (string | null) | undefined }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateWorkflowFolder = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/automation/folders";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateFolderRequest;
    };
    responses: {
      201: {
        data: { created_at: string; id: string; name: string; parent_id?: (string | null) | undefined };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      409: unknown;
    };
  };
  export type delete_DeleteWorkflowFolder = {
    method: "DELETE";
    path: "/api/v1/organizations/{organization_id}/automation/folders/{folder_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; folder_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown; 409: unknown };
  };
  export type patch_UpdateWorkflowFolder = {
    method: "PATCH";
    path: "/api/v1/organizations/{organization_id}/automation/folders/{folder_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; folder_id: string };

      body: Schemas.UpdateFolderRequest;
    };
    responses: {
      200: {
        data: { created_at: string; id: string; name: string; parent_id?: (string | null) | undefined };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_ListRuns = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/automation/runs";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          created_at: string;
          error?: (string | null) | undefined;
          failed_connector_id?: (string | null) | undefined;
          finished_at?: (string | null) | undefined;
          id: string;
          next_attempt_at?: (string | null) | undefined;
          number: number;
          organization_id: Schemas.OrganizationId;
          started_at?: (string | null) | undefined;
          status: string;
          steps_done: number;
          steps_total: number;
          trigger_event_id?: (string | null) | undefined;
          trigger_event_name?: (string | null) | undefined;
          trigger_id?: (string | null) | undefined;
          trigger_payload?: unknown | undefined;
          workflow_id: string;
          workflow_version_created_at: string;
          workflow_version_id: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type get_GetRun = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/automation/runs/{run_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; run_id: string };
    };
    responses: {
      200: {
        data: Schemas.RunResponse & {
          graph?: (null | Schemas.GraphDto) | undefined;
          steps: Array<Schemas.RunStepResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type post_ReplayRun = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/automation/runs/{run_id}/replay";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; run_id: string };

      body: Schemas.ReplayRunRequest;
    };
    responses: {
      200: {
        data: {
          created_at: string;
          error?: (string | null) | undefined;
          failed_connector_id?: (string | null) | undefined;
          finished_at?: (string | null) | undefined;
          id: string;
          next_attempt_at?: (string | null) | undefined;
          number: number;
          organization_id: Schemas.OrganizationId;
          started_at?: (string | null) | undefined;
          status: string;
          steps_done: number;
          steps_total: number;
          trigger_event_id?: (string | null) | undefined;
          trigger_event_name?: (string | null) | undefined;
          trigger_id?: (string | null) | undefined;
          trigger_payload?: unknown | undefined;
          workflow_id: string;
          workflow_version_created_at: string;
          workflow_version_id: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_GetAutomationSettings = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/automation/settings";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: {
          disable_target_after?: (number | null) | undefined;
          event_retention_seconds: number;
          retry_schedule_seconds: Array<number>;
          succeeded_run_retention_seconds: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type put_UpdateAutomationSettings = {
    method: "PUT";
    path: "/api/v1/organizations/{organization_id}/automation/settings";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.AutomationSettingsBody;
    };
    responses: {
      200: {
        data: {
          disable_target_after?: (number | null) | undefined;
          event_retention_seconds: number;
          retry_schedule_seconds: Array<number>;
          succeeded_run_retention_seconds: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      409: unknown;
    };
  };
  export type get_ListWorkflows = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/automation/workflows";
    requestFormat: "json";
    parameters: {
      query: Partial<{ archived: boolean }>;
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          archived_at?: (string | null) | undefined;
          created_at: string;
          current_version_id?: (string | null) | undefined;
          description?: (string | null) | undefined;
          enabled: boolean;
          folder_id?: (string | null) | undefined;
          id: string;
          name: string;
          organization_id: Schemas.OrganizationId;
          tags: Array<string>;
          updated_at: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateWorkflow = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/automation/workflows";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateWorkflowRequest;
    };
    responses: {
      201: {
        data: {
          archived_at?: (string | null) | undefined;
          created_at: string;
          current_version_id?: (string | null) | undefined;
          description?: (string | null) | undefined;
          enabled: boolean;
          folder_id?: (string | null) | undefined;
          id: string;
          name: string;
          organization_id: Schemas.OrganizationId;
          tags: Array<string>;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      409: unknown;
    };
  };
  export type post_DryRunWorkflow = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/automation/workflows/dry-run";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.DryRunRequest;
    };
    responses: {
      200: {
        data: { steps: Array<Schemas.DryRunStepResponse> };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      413: unknown;
      422: { code: string; details: Schemas.GraphInvalidDetails; message: string; status: number };
    };
  };
  export type get_GetWorkflow = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; workflow_id: string };
    };
    responses: {
      200: {
        data: {
          archived_at?: (string | null) | undefined;
          created_at: string;
          current_version?: (null | Schemas.WorkflowVersionResponse) | undefined;
          description?: (string | null) | undefined;
          enabled: boolean;
          folder_id?: (string | null) | undefined;
          id: string;
          layout?: (null | Schemas.WorkflowLayoutDto) | undefined;
          name: string;
          organization_id: Schemas.OrganizationId;
          tags: Array<string>;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type delete_DeleteWorkflow = {
    method: "DELETE";
    path: "/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; workflow_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type patch_UpdateWorkflow = {
    method: "PATCH";
    path: "/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; workflow_id: string };

      body: Schemas.UpdateWorkflowRequest;
    };
    responses: {
      200: {
        data: {
          archived_at?: (string | null) | undefined;
          created_at: string;
          current_version_id?: (string | null) | undefined;
          description?: (string | null) | undefined;
          enabled: boolean;
          folder_id?: (string | null) | undefined;
          id: string;
          name: string;
          organization_id: Schemas.OrganizationId;
          tags: Array<string>;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type post_StartRun = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}/runs";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; workflow_id: string };

      body: Schemas.StartRunRequest;
    };
    responses: {
      201: { data: { run_id: string }; pagination?: (null | Schemas.PaginationMetadata) | undefined };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type put_SaveWorkflowVersion = {
    method: "PUT";
    path: "/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}/versions";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; workflow_id: string };

      body: Schemas.SaveWorkflowVersionRequest;
    };
    responses: {
      201: {
        data: {
          created_at: string;
          created_by?: (string | null) | undefined;
          graph: Schemas.GraphDto;
          id: string;
          version: number;
          workflow_id: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: { code: string; details: Schemas.GraphInvalidDetails; message: string; status: number };
      422: { code: string; details: Schemas.GraphInvalidDetails; message: string; status: number };
    };
  };
  export type get_GetOrganizationBillingSettings = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/billing-settings";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: {
          bic?: (string | null) | undefined;
          default_vat_rate_bp: number;
          iban?: (string | null) | undefined;
          invoice_footer: string;
          late_penalty: string;
          number_padding: number;
          payment_terms_days: number;
          quote_validity_days: number;
          recovery_fee_cents: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type put_ReplaceOrganizationBillingSettings = {
    method: "PUT";
    path: "/api/v1/organizations/{organization_id}/billing-settings";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.ReplaceBillingSettingsRequest;
    };
    responses: {
      200: {
        data: {
          bic?: (string | null) | undefined;
          default_vat_rate_bp: number;
          iban?: (string | null) | undefined;
          invoice_footer: string;
          late_penalty: string;
          number_padding: number;
          payment_terms_days: number;
          quote_validity_days: number;
          recovery_fee_cents: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      422: unknown;
    };
  };
  export type get_ListCustomUnits = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/custom-units";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          code: string;
          created_at: string;
          id: Schemas.CustomUnitId;
          organization_id: Schemas.OrganizationId;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateCustomUnit = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/custom-units";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateCustomUnitRequest;
    };
    responses: {
      201: {
        data: { code: string; created_at: string; id: Schemas.CustomUnitId; organization_id: Schemas.OrganizationId };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      409: unknown;
    };
  };
  export type get_ListCustomers = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/customers";
    requestFormat: "json";
    parameters: {
      query: Partial<{ page: number; per_page: number }>;
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<
          Schemas.CustomerResponse & {
            city?: (string | null) | undefined;
            last_activity_at?: (string | null) | undefined;
            overdue_cents: number;
            primary_contact?: (null | Schemas.PrimaryContactResponse) | undefined;
            revenue_12m_cents: number;
          }
        >;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateCustomer = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/customers";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateCustomerRequest;
    };
    responses: {
      201: {
        data: {
          archived_at?: (string | null) | undefined;
          billing_address_id?: (null | Schemas.CustomerContextId) | undefined;
          billing_contact_id?: (null | Schemas.CustomerContactId) | undefined;
          converted_at?: (string | null) | undefined;
          created_at: string;
          email?: (string | null) | undefined;
          id: Schemas.CustomerId;
          kind: Schemas.CustomerKind;
          lead_source?: (null | Schemas.LeadSource) | undefined;
          name: string;
          notes?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          owner_member_id?: (null | Schemas.MemberId) | undefined;
          payment_terms_days?: (number | null) | undefined;
          phone?: (string | null) | undefined;
          registration_number?: (string | null) | undefined;
          status: Schemas.CustomerStatus;
          updated_at: string;
          vat_number?: (string | null) | undefined;
          website?: (string | null) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      409: unknown;
      422: unknown;
    };
  };
  export type get_ListDayOverrides = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/day-overrides";
    requestFormat: "json";
    parameters: {
      query: { from: string; to: string; member_id?: string | undefined };
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          date: string;
          end?: (string | null) | undefined;
          id: Schemas.DayOverrideId;
          kind: Schemas.OverrideKind;
          lunch?: (null | Schemas.LunchInput) | undefined;
          member_id: Schemas.MemberId;
          note: string;
          organization_id: Schemas.OrganizationId;
          start?: (string | null) | undefined;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      422: unknown;
    };
  };
  export type get_ListDeals = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/deals";
    requestFormat: "json";
    parameters: {
      query: Partial<{
        page: number;
        per_page: number;
        customer_id: string;
        stage: "NEW" | "VISIT" | "QUOTE_SENT" | "NEGOTIATION" | "WON" | "LOST";
        q: string;
      }>;
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          amount_cents: number;
          closed_at?: (string | null) | undefined;
          created_at: string;
          customer_id: Schemas.CustomerId;
          expected_close_date?: (string | null) | undefined;
          id: Schemas.DealId;
          lost_reason?: (string | null) | undefined;
          notes?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          owner_member_id?: (null | Schemas.MemberId) | undefined;
          quote_count: number;
          source?: (null | Schemas.LeadSource) | undefined;
          stage: Schemas.DealStageName;
          stage_changed_at: string;
          title: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateDeal = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/deals";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateDealRequest;
    };
    responses: {
      201: {
        data: {
          amount_cents: number;
          closed_at?: (string | null) | undefined;
          created_at: string;
          customer_id: Schemas.CustomerId;
          expected_close_date?: (string | null) | undefined;
          id: Schemas.DealId;
          lost_reason?: (string | null) | undefined;
          notes?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          owner_member_id?: (null | Schemas.MemberId) | undefined;
          quote_count: number;
          source?: (null | Schemas.LeadSource) | undefined;
          stage: Schemas.DealStageName;
          stage_changed_at: string;
          title: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      422: unknown;
    };
  };
  export type get_GetDealSummary = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/deals/summary";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: {
          conversions: number;
          lost_count: number;
          open_amount_cents: number;
          open_count: number;
          stages: Array<Schemas.StageTotalResponse>;
          win_rate?: (number | null) | undefined;
          won_amount_cents: number;
          won_count: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type get_ListDoeModelStarters = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/doe-model-starters";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          description?: (string | null) | undefined;
          name: string;
          sections: Array<Schemas.DoeModelSection>;
          starter: Schemas.StarterModel;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type get_ListDoeModels = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/doe-models";
    requestFormat: "json";
    parameters: {
      query: Partial<{ page: number; per_page: number; include_archived: boolean }>;
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          archived_at?: (string | null) | undefined;
          created_at: string;
          description?: (string | null) | undefined;
          id: Schemas.DoeModelId;
          name: string;
          organization_id: Schemas.OrganizationId;
          sections: Array<Schemas.DoeModelSection>;
          style?: (null | Schemas.DoeStyle) | undefined;
          updated_at: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateDoeModel = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/doe-models";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateDoeModelRequest;
    };
    responses: {
      201: {
        data: {
          archived_at?: (string | null) | undefined;
          created_at: string;
          description?: (string | null) | undefined;
          id: Schemas.DoeModelId;
          name: string;
          organization_id: Schemas.OrganizationId;
          sections: Array<Schemas.DoeModelSection>;
          style?: (null | Schemas.DoeStyle) | undefined;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      409: unknown;
    };
  };
  export type post_CreateDoeModelFromStarter = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/doe-models/from-starter";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateDoeModelFromStarterRequest;
    };
    responses: {
      201: {
        data: {
          archived_at?: (string | null) | undefined;
          created_at: string;
          description?: (string | null) | undefined;
          id: Schemas.DoeModelId;
          name: string;
          organization_id: Schemas.OrganizationId;
          sections: Array<Schemas.DoeModelSection>;
          style?: (null | Schemas.DoeStyle) | undefined;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type get_GetDoeModel = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/doe-models/{doe_model_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; doe_model_id: string };
    };
    responses: {
      200: {
        data: {
          archived_at?: (string | null) | undefined;
          created_at: string;
          description?: (string | null) | undefined;
          id: Schemas.DoeModelId;
          name: string;
          organization_id: Schemas.OrganizationId;
          sections: Array<Schemas.DoeModelSection>;
          style?: (null | Schemas.DoeStyle) | undefined;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type put_UpdateDoeModel = {
    method: "PUT";
    path: "/api/v1/organizations/{organization_id}/doe-models/{doe_model_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; doe_model_id: string };

      body: Schemas.UpdateDoeModelRequest;
    };
    responses: {
      200: {
        data: {
          archived_at?: (string | null) | undefined;
          created_at: string;
          description?: (string | null) | undefined;
          id: Schemas.DoeModelId;
          name: string;
          organization_id: Schemas.OrganizationId;
          sections: Array<Schemas.DoeModelSection>;
          style?: (null | Schemas.DoeStyle) | undefined;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type delete_ArchiveDoeModel = {
    method: "DELETE";
    path: "/api/v1/organizations/{organization_id}/doe-models/{doe_model_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; doe_model_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type post_RestoreDoeModel = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/doe-models/{doe_model_id}/restore";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; doe_model_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type get_GetDoeSettings = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/doe-settings";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: { data: { style: Schemas.DoeStyle }; pagination?: (null | Schemas.PaginationMetadata) | undefined };
      401: unknown;
      403: unknown;
    };
  };
  export type put_UpdateDoeSettings = {
    method: "PUT";
    path: "/api/v1/organizations/{organization_id}/doe-settings";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.UpdateDoeSettingsRequest;
    };
    responses: {
      200: { data: { style: Schemas.DoeStyle }; pagination?: (null | Schemas.PaginationMetadata) | undefined };
      401: unknown;
      403: unknown;
    };
  };
  export type post_PreviewDoeStyle = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/doe-style/preview";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.PreviewDoeStyleRequest;
    };
    responses: { 200: unknown; 401: unknown; 403: unknown; 409: unknown };
  };
  export type post_PreviewDoeStylePages = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/doe-style/preview/pages";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.PreviewDoeStyleRequest;
    };
    responses: {
      200: { data: { pages: Array<string> }; pagination?: (null | Schemas.PaginationMetadata) | undefined };
      401: unknown;
      403: unknown;
      409: unknown;
    };
  };
  export type get_ListDoes = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/does";
    requestFormat: "json";
    parameters: {
      query: Partial<{ page: number; per_page: number }>;
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          id: Schemas.DoeId;
          issued_on: string;
          project_id: Schemas.ProjectId;
          project_name: string;
          revision: Schemas.Revision;
          title: string;
          updated_at: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type get_GetDoe = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/does/{doe_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; doe_id: string };
    };
    responses: {
      200: {
        data: {
          author?: (string | null) | undefined;
          cover_photo_key?: (string | null) | undefined;
          created_at: string;
          id: Schemas.DoeId;
          issued_on: string;
          lot?: (string | null) | undefined;
          model_id?: (null | Schemas.DoeModelId) | undefined;
          parties: Array<Schemas.DoeParty>;
          place?: (string | null) | undefined;
          project_id: Schemas.ProjectId;
          revision: Schemas.Revision;
          sections: Array<Schemas.DoeSection>;
          title: string;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type put_UpdateDoe = {
    method: "PUT";
    path: "/api/v1/organizations/{organization_id}/does/{doe_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; doe_id: string };

      body: Schemas.UpdateDoeRequest;
    };
    responses: {
      200: {
        data: {
          author?: (string | null) | undefined;
          cover_photo_key?: (string | null) | undefined;
          created_at: string;
          id: Schemas.DoeId;
          issued_on: string;
          lot?: (string | null) | undefined;
          model_id?: (null | Schemas.DoeModelId) | undefined;
          parties: Array<Schemas.DoeParty>;
          place?: (string | null) | undefined;
          project_id: Schemas.ProjectId;
          revision: Schemas.Revision;
          sections: Array<Schemas.DoeSection>;
          title: string;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type delete_DeleteDoe = {
    method: "DELETE";
    path: "/api/v1/organizations/{organization_id}/does/{doe_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; doe_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type get_ExportDoePdf = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/does/{doe_id}/pdf";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; doe_id: string };
    };
    responses: { 200: unknown; 401: unknown; 403: unknown; 404: unknown; 409: unknown };
  };
  export type post_DraftDoeSupply = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/does/{doe_id}/supplies/draft";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; doe_id: string };

      body: Schemas.DraftSupplyRequest;
    };
    responses: {
      200: {
        data: {
          datasheet_keys: Array<string>;
          designation: string;
          destination?: (string | null) | undefined;
          dpgf_ref?: (string | null) | undefined;
          id: string;
          maker?: (string | null) | undefined;
          photos: Array<Schemas.DoeFile>;
          source_product_id?: (null | Schemas.ProductId) | undefined;
          specs?: (string | null) | undefined;
          supplier?: (null | Schemas.SupplierSnapshot) | undefined;
          title?: (string | null) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type get_ListEmployeeProfiles = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/employee-profiles";
    requestFormat: "json";
    parameters: {
      query: Partial<{ page: number; per_page: number }>;
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          created_at: string;
          effective_hourly_rate_cents?: (number | null) | undefined;
          hourly_rate_cents?: (number | null) | undefined;
          id: Schemas.EmployeeId;
          is_salaried: boolean;
          member_id: Schemas.MemberId;
          monthly_cost_cents?: (number | null) | undefined;
          organization_id: Schemas.OrganizationId;
          updated_at: string;
          weekly_contract_minutes: number;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type get_ListEquipment = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/equipment";
    requestFormat: "json";
    parameters: {
      query: Partial<{ page: number; per_page: number }>;
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          created_at: string;
          hourly_rate_cents: number;
          id: Schemas.EquipmentId;
          name: string;
          organization_id: Schemas.OrganizationId;
          updated_at: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateEquipment = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/equipment";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateEquipmentRequest;
    };
    responses: {
      201: {
        data: {
          created_at: string;
          hourly_rate_cents: number;
          id: Schemas.EquipmentId;
          name: string;
          organization_id: Schemas.OrganizationId;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      409: unknown;
    };
  };
  export type get_ListExpenseCategories = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/expense-categories";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          created_at: string;
          hue: number;
          id: Schemas.ExpenseCategoryId;
          name: string;
          organization_id: Schemas.OrganizationId;
          updated_at: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateExpenseCategory = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/expense-categories";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.ExpenseCategoryRequest;
    };
    responses: {
      201: {
        data: {
          created_at: string;
          hue: number;
          id: Schemas.ExpenseCategoryId;
          name: string;
          organization_id: Schemas.OrganizationId;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      409: unknown;
      422: unknown;
    };
  };
  export type get_ListExpenses = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/expenses";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          advanced_by_member_id?: (null | Schemas.MemberId) | undefined;
          amount_ht_cents: number;
          category_id: Schemas.ExpenseCategoryId;
          created_at: string;
          customer_id?: (null | Schemas.CustomerId) | undefined;
          customer_name?: (string | null) | undefined;
          date: string;
          days_late: number;
          display_state: Schemas.ExpenseDisplayState;
          due_date?: (string | null) | undefined;
          id: Schemas.ExpenseId;
          label: string;
          method?: (null | Schemas.PaymentMethod) | undefined;
          note?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          paid_on?: (string | null) | undefined;
          proof_file_key?: (string | null) | undefined;
          reference?: (string | null) | undefined;
          reimbursed_on?: (string | null) | undefined;
          status: Schemas.ExpenseStatus;
          supplier_id?: (null | Schemas.SupplierId) | undefined;
          supplier_name?: (string | null) | undefined;
          total_cents: number;
          updated_at: string;
          vat_cents: number;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateExpense = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/expenses";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateExpenseRequest;
    };
    responses: {
      201: {
        data: {
          advanced_by_member_id?: (null | Schemas.MemberId) | undefined;
          amount_ht_cents: number;
          category_id: Schemas.ExpenseCategoryId;
          created_at: string;
          customer_id?: (null | Schemas.CustomerId) | undefined;
          customer_name?: (string | null) | undefined;
          date: string;
          days_late: number;
          display_state: Schemas.ExpenseDisplayState;
          due_date?: (string | null) | undefined;
          id: Schemas.ExpenseId;
          label: string;
          method?: (null | Schemas.PaymentMethod) | undefined;
          note?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          paid_on?: (string | null) | undefined;
          proof_file_key?: (string | null) | undefined;
          reference?: (string | null) | undefined;
          reimbursed_on?: (string | null) | undefined;
          status: Schemas.ExpenseStatus;
          supplier_id?: (null | Schemas.SupplierId) | undefined;
          supplier_name?: (string | null) | undefined;
          total_cents: number;
          updated_at: string;
          vat_cents: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      422: unknown;
    };
  };
  export type get_ListExtraPeriods = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/extra-periods";
    requestFormat: "json";
    parameters: {
      query: Partial<{
        member_id: string;
        status: "PENDING" | "APPROVED" | "REJECTED" | "CANCELLED";
        from: string;
        to: string;
        page: number;
        per_page: number;
      }>;
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          created_at: string;
          created_by_member_id?: (null | Schemas.MemberId) | undefined;
          date: string;
          decided_at?: (string | null) | undefined;
          decided_by_member_id?: (null | Schemas.MemberId) | undefined;
          decision_comment?: (string | null) | undefined;
          end: string;
          id: Schemas.ExtraPeriodId;
          lunch?: (null | Schemas.LunchInput) | undefined;
          member_id: Schemas.MemberId;
          organization_id: Schemas.OrganizationId;
          rate_bp: number;
          reason: string;
          start: string;
          status: Schemas.ExtraStatus;
          surcharge_id?: (null | Schemas.SurchargeId) | undefined;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateExtraPeriod = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/extra-periods";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateExtraPeriodRequest;
    };
    responses: {
      201: {
        data: {
          created_at: string;
          created_by_member_id?: (null | Schemas.MemberId) | undefined;
          date: string;
          decided_at?: (string | null) | undefined;
          decided_by_member_id?: (null | Schemas.MemberId) | undefined;
          decision_comment?: (string | null) | undefined;
          end: string;
          id: Schemas.ExtraPeriodId;
          lunch?: (null | Schemas.LunchInput) | undefined;
          member_id: Schemas.MemberId;
          organization_id: Schemas.OrganizationId;
          rate_bp: number;
          reason: string;
          start: string;
          status: Schemas.ExtraStatus;
          surcharge_id?: (null | Schemas.SurchargeId) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      422: unknown;
    };
  };
  export type post_ApproveExtraPeriod = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/extra-periods/{period_id}/approve";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; period_id: string };
    };
    responses: {
      200: {
        data: {
          created_at: string;
          created_by_member_id?: (null | Schemas.MemberId) | undefined;
          date: string;
          decided_at?: (string | null) | undefined;
          decided_by_member_id?: (null | Schemas.MemberId) | undefined;
          decision_comment?: (string | null) | undefined;
          end: string;
          id: Schemas.ExtraPeriodId;
          lunch?: (null | Schemas.LunchInput) | undefined;
          member_id: Schemas.MemberId;
          organization_id: Schemas.OrganizationId;
          rate_bp: number;
          reason: string;
          start: string;
          status: Schemas.ExtraStatus;
          surcharge_id?: (null | Schemas.SurchargeId) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      422: unknown;
    };
  };
  export type post_CancelExtraPeriod = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/extra-periods/{period_id}/cancel";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; period_id: string };
    };
    responses: {
      200: {
        data: {
          created_at: string;
          created_by_member_id?: (null | Schemas.MemberId) | undefined;
          date: string;
          decided_at?: (string | null) | undefined;
          decided_by_member_id?: (null | Schemas.MemberId) | undefined;
          decision_comment?: (string | null) | undefined;
          end: string;
          id: Schemas.ExtraPeriodId;
          lunch?: (null | Schemas.LunchInput) | undefined;
          member_id: Schemas.MemberId;
          organization_id: Schemas.OrganizationId;
          rate_bp: number;
          reason: string;
          start: string;
          status: Schemas.ExtraStatus;
          surcharge_id?: (null | Schemas.SurchargeId) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      422: unknown;
    };
  };
  export type post_RejectExtraPeriod = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/extra-periods/{period_id}/reject";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; period_id: string };

      body: Schemas.RejectExtraPeriodRequest;
    };
    responses: {
      200: {
        data: {
          created_at: string;
          created_by_member_id?: (null | Schemas.MemberId) | undefined;
          date: string;
          decided_at?: (string | null) | undefined;
          decided_by_member_id?: (null | Schemas.MemberId) | undefined;
          decision_comment?: (string | null) | undefined;
          end: string;
          id: Schemas.ExtraPeriodId;
          lunch?: (null | Schemas.LunchInput) | undefined;
          member_id: Schemas.MemberId;
          organization_id: Schemas.OrganizationId;
          rate_bp: number;
          reason: string;
          start: string;
          status: Schemas.ExtraStatus;
          surcharge_id?: (null | Schemas.SurchargeId) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      422: unknown;
    };
  };
  export type get_ListMyAssignmentReports = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/field/assignment-reports";
    requestFormat: "json";
    parameters: {
      query: Partial<{ resolution: "PENDING" | "APPLIED" | "DISMISSED"; page: number; per_page: number }>;
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          comment?: (string | null) | undefined;
          created_at: string;
          id: Schemas.AssignmentReportId;
          organization_id: Schemas.OrganizationId;
          reported_by: Schemas.MemberId;
          reported_minutes: number;
          resolution: Schemas.AssignmentReportResolution;
          resolution_note?: (string | null) | undefined;
          resolved_at?: (string | null) | undefined;
          resolved_by?: (null | Schemas.MemberId) | undefined;
          task_assignment_id: Schemas.TaskAssignmentId;
          updated_at: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_ReportAssignment = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/field/assignments/{task_assignment_id}/report";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; task_assignment_id: string };

      body: Schemas.ReportAssignmentRequest;
    };
    responses: {
      201: {
        data: {
          comment?: (string | null) | undefined;
          created_at: string;
          id: Schemas.AssignmentReportId;
          organization_id: Schemas.OrganizationId;
          reported_by: Schemas.MemberId;
          reported_minutes: number;
          resolution: Schemas.AssignmentReportResolution;
          resolution_note?: (string | null) | undefined;
          resolved_at?: (string | null) | undefined;
          resolved_by?: (null | Schemas.MemberId) | undefined;
          task_assignment_id: Schemas.TaskAssignmentId;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_GetCurrentTimeEntry = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/field/current";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Partial<{ day_ended_at: string | null; running: null | Schemas.TimeEntryResponse }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_EndWorkingDay = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/field/day-end";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.EndDayRequest;
    };
    responses: {
      201: {
        data: {
          employee_id: Schemas.EmployeeId;
          ended_at: string;
          id: Schemas.DayLogId;
          organization_id: Schemas.OrganizationId;
          work_date: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      409: unknown;
    };
  };
  export type get_ListMyFieldTasks = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/field/tasks";
    requestFormat: "json";
    parameters: {
      query: Partial<{ work_date: string }>;
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          all_day: boolean;
          customer_context_id?: (null | Schemas.CustomerContextId) | undefined;
          customer_id?: (null | Schemas.CustomerId) | undefined;
          description?: (string | null) | undefined;
          ends_at?: (string | null) | undefined;
          id: Schemas.TaskId;
          maintenance_line_id?: (null | Schemas.MaintenanceLineId) | undefined;
          starts_at?: (string | null) | undefined;
          status: Schemas.TaskStatus;
          task_assignment_id: Schemas.TaskAssignmentId;
          title: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_ProposeMaintenanceIntervention = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/field/tasks/{task_id}/intervention";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; task_id: string };

      body: Schemas.ProposeInterventionRequest;
    };
    responses: {
      201: {
        data: {
          done_on: string;
          id: Schemas.InterventionId;
          line_id: Schemas.MaintenanceLineId;
          status: Schemas.InterventionStatus;
          task_id?: (null | Schemas.TaskId) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type post_StartTimeEntry = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/field/time-entries";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.StartTimeEntryRequest;
    };
    responses: {
      201: {
        data: {
          employee_id: Schemas.EmployeeId;
          ended_at?: (string | null) | undefined;
          id: Schemas.TimeEntryId;
          organization_id: Schemas.OrganizationId;
          photos: Array<Schemas.TimeEntryPhotoResponse>;
          started_at: string;
          task_id: Schemas.TaskId;
          worked_minutes?: (number | null) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      409: unknown;
    };
  };
  export type post_DeclareTimeEntry = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/field/time-entries/declare";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.DeclareTimeEntryRequest;
    };
    responses: {
      201: {
        data: {
          employee_id: Schemas.EmployeeId;
          ended_at?: (string | null) | undefined;
          id: Schemas.TimeEntryId;
          organization_id: Schemas.OrganizationId;
          photos: Array<Schemas.TimeEntryPhotoResponse>;
          started_at: string;
          task_id: Schemas.TaskId;
          worked_minutes?: (number | null) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      409: unknown;
    };
  };
  export type get_GetHrPlanningFeed = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/hr/planning-feed";
    requestFormat: "json";
    parameters: {
      query: { from: string; to: string };
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: {
          absence_types: Array<Schemas.AbsenceTypeResponse>;
          absences: Array<Schemas.AbsenceResponse>;
          contracts: Array<Schemas.MemberContractResponse>;
          members: Array<Schemas.FeedMemberResponse>;
          overrides: Array<Schemas.DayOverrideResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      422: unknown;
    };
  };
  export type get_ListInvitations = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/invitations";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          created_at: string;
          expires_at: string;
          id: Schemas.InvitationId;
          member_id?: (null | Schemas.MemberId) | undefined;
          organization_id: Schemas.OrganizationId;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateInvitation = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/invitations";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateInvitationRequest;
    };
    responses: {
      201: {
        data: Schemas.InvitationResponse & { token: string };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_ListInvoices = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/invoices";
    requestFormat: "json";
    parameters: {
      query: Partial<{ page: number; per_page: number }>;
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          balance_cents: number;
          created_at: string;
          credited_cents: number;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          delivery_address?: (null | Schemas.DeliveryAddressResponse) | undefined;
          display_state: Schemas.InvoiceDisplayState;
          due_at?: (string | null) | undefined;
          due_on?: (string | null) | undefined;
          events: Array<Schemas.InvoiceEventResponse>;
          full_cancellation: boolean;
          gross_cents: number;
          id: Schemas.InvoiceId;
          internal_note?: (string | null) | undefined;
          issued_at?: (string | null) | undefined;
          issued_on?: (string | null) | undefined;
          kind: Schemas.InvoiceKind;
          legal_notice: string;
          lines: Array<Schemas.InvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number?: (string | null) | undefined;
          operation_nature?: (null | Schemas.OperationNature) | undefined;
          organization_id: Schemas.OrganizationId;
          paid_cents: number;
          payment_terms_days: number;
          project_id?: (null | Schemas.ProjectId) | undefined;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          reason?: (string | null) | undefined;
          sent_at?: (string | null) | undefined;
          source_invoice_id?: (null | Schemas.InvoiceId) | undefined;
          status: Schemas.InvoiceStatus;
          title: string;
          updated_at: string;
          vat_breakdown: Array<Schemas.InvoiceVatBreakdownLineResponse>;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateInvoice = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/invoices";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateInvoiceRequest;
    };
    responses: {
      201: {
        data: {
          balance_cents: number;
          created_at: string;
          credited_cents: number;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          delivery_address?: (null | Schemas.DeliveryAddressResponse) | undefined;
          display_state: Schemas.InvoiceDisplayState;
          due_at?: (string | null) | undefined;
          due_on?: (string | null) | undefined;
          events: Array<Schemas.InvoiceEventResponse>;
          full_cancellation: boolean;
          gross_cents: number;
          id: Schemas.InvoiceId;
          internal_note?: (string | null) | undefined;
          issued_at?: (string | null) | undefined;
          issued_on?: (string | null) | undefined;
          kind: Schemas.InvoiceKind;
          legal_notice: string;
          lines: Array<Schemas.InvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number?: (string | null) | undefined;
          operation_nature?: (null | Schemas.OperationNature) | undefined;
          organization_id: Schemas.OrganizationId;
          paid_cents: number;
          payment_terms_days: number;
          project_id?: (null | Schemas.ProjectId) | undefined;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          reason?: (string | null) | undefined;
          sent_at?: (string | null) | undefined;
          source_invoice_id?: (null | Schemas.InvoiceId) | undefined;
          status: Schemas.InvoiceStatus;
          title: string;
          updated_at: string;
          vat_breakdown: Array<Schemas.InvoiceVatBreakdownLineResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      409: unknown;
      422: unknown;
    };
  };
  export type get_ListOutstandingBalanceByCustomer = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/invoices/outstanding";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          customer_id: Schemas.CustomerId;
          oldest_due_at?: (string | null) | undefined;
          outstanding_cents: number;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type get_ListLeaveAccounts = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/leave-accounts";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          acquisition: Schemas.AcquisitionBody;
          archived_at?: (string | null) | undefined;
          carry_over_max: number;
          created_at: string;
          id: Schemas.LeaveAccountId;
          name: string;
          organization_id: Schemas.OrganizationId;
          period_start_month: number;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateLeaveAccount = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/leave-accounts";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.LeaveAccountRequest;
    };
    responses: {
      201: {
        data: {
          acquisition: Schemas.AcquisitionBody;
          archived_at?: (string | null) | undefined;
          carry_over_max: number;
          created_at: string;
          id: Schemas.LeaveAccountId;
          name: string;
          organization_id: Schemas.OrganizationId;
          period_start_month: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      422: unknown;
    };
  };
  export type put_UpdateLeaveAccount = {
    method: "PUT";
    path: "/api/v1/organizations/{organization_id}/leave-accounts/{account_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; account_id: string };

      body: Schemas.LeaveAccountRequest;
    };
    responses: {
      200: {
        data: {
          acquisition: Schemas.AcquisitionBody;
          archived_at?: (string | null) | undefined;
          carry_over_max: number;
          created_at: string;
          id: Schemas.LeaveAccountId;
          name: string;
          organization_id: Schemas.OrganizationId;
          period_start_month: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      422: unknown;
    };
  };
  export type post_ArchiveLeaveAccount = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/leave-accounts/{account_id}/archive";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; account_id: string };
    };
    responses: {
      200: {
        data: {
          acquisition: Schemas.AcquisitionBody;
          archived_at?: (string | null) | undefined;
          carry_over_max: number;
          created_at: string;
          id: Schemas.LeaveAccountId;
          name: string;
          organization_id: Schemas.OrganizationId;
          period_start_month: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type post_RestoreLeaveAccount = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/leave-accounts/{account_id}/restore";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; account_id: string };
    };
    responses: {
      200: {
        data: {
          acquisition: Schemas.AcquisitionBody;
          archived_at?: (string | null) | undefined;
          carry_over_max: number;
          created_at: string;
          id: Schemas.LeaveAccountId;
          name: string;
          organization_id: Schemas.OrganizationId;
          period_start_month: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type get_ListLeaveAdjustments = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/leave-adjustments";
    requestFormat: "json";
    parameters: {
      query: Partial<{ member_id: string; account_id: string }>;
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          account_id: Schemas.LeaveAccountId;
          author_member_id?: (null | Schemas.MemberId) | undefined;
          created_at: string;
          date: string;
          days: number;
          id: Schemas.LeaveAdjustmentId;
          kind: Schemas.AdjustmentKind;
          member_id: Schemas.MemberId;
          organization_id: Schemas.OrganizationId;
          reason: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateLeaveAdjustment = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/leave-adjustments";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateLeaveAdjustmentRequest;
    };
    responses: {
      201: {
        data: {
          account_id: Schemas.LeaveAccountId;
          author_member_id?: (null | Schemas.MemberId) | undefined;
          created_at: string;
          date: string;
          days: number;
          id: Schemas.LeaveAdjustmentId;
          kind: Schemas.AdjustmentKind;
          member_id: Schemas.MemberId;
          organization_id: Schemas.OrganizationId;
          reason: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      422: unknown;
    };
  };
  export type delete_DeleteLeaveAdjustment = {
    method: "DELETE";
    path: "/api/v1/organizations/{organization_id}/leave-adjustments/{adjustment_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; adjustment_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type get_ReadLedger = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/ledger";
    requestFormat: "json";
    parameters: {
      query: Partial<{ month: string }>;
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: {
          entries: Array<Schemas.LedgerEntryResponse>;
          months: Array<string>;
          payables: Schemas.PayablesResponse;
          summary: Schemas.LedgerSummaryResponse;
          trend: Array<Schemas.MonthTrendResponse>;
          vat: Schemas.VatSummaryResponse;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      422: unknown;
    };
  };
  export type patch_UpdateOrganizationLegalIdentity = {
    method: "PATCH";
    path: "/api/v1/organizations/{organization_id}/legal-identity";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.UpdateLegalIdentityRequest;
    };
    responses: {
      200: {
        data: {
          address_city?: (string | null) | undefined;
          address_country?: (string | null) | undefined;
          address_line1?: (string | null) | undefined;
          address_line2?: (string | null) | undefined;
          address_postal_code?: (string | null) | undefined;
          contact_email?: (string | null) | undefined;
          contact_phone?: (string | null) | undefined;
          created_at: string;
          field_clock_enabled: boolean;
          id: Schemas.OrganizationId;
          insurance_mention?: (string | null) | undefined;
          legal_form?: (string | null) | undefined;
          legal_name?: (string | null) | undefined;
          mcp_enabled: boolean;
          missing_legal_identity_fields: Array<Schemas.str>;
          name: string;
          owner_id: Schemas.UserId;
          registration_number?: (string | null) | undefined;
          share_capital_cents?: (number | null) | undefined;
          slug: string;
          updated_at: string;
          vat_on_debits: boolean;
          vat_status?: (null | Schemas.VatStatusResponse) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type get_ListMaintenanceContracts = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/maintenance/contracts";
    requestFormat: "json";
    parameters: {
      query: { season: number; include_archived?: boolean | undefined };
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          amount_cents?: (number | null) | undefined;
          archived_at?: (string | null) | undefined;
          created_at: string;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          id: Schemas.MaintenanceContractId;
          lines: Array<Schemas.MaintenanceLine>;
          manager_member_id?: (null | Schemas.MemberId) | undefined;
          notes?: (string | null) | undefined;
          project_id: Schemas.ProjectId;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          reference?: (string | null) | undefined;
          season: Schemas.Season;
          status: Schemas.ContractStatus;
          updated_at: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateMaintenanceContract = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/maintenance/contracts";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateMaintenanceContractRequest;
    };
    responses: {
      201: {
        data: {
          amount_cents?: (number | null) | undefined;
          archived_at?: (string | null) | undefined;
          created_at: string;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          id: Schemas.MaintenanceContractId;
          lines: Array<Schemas.MaintenanceLine>;
          manager_member_id?: (null | Schemas.MemberId) | undefined;
          notes?: (string | null) | undefined;
          project_id: Schemas.ProjectId;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          reference?: (string | null) | undefined;
          season: Schemas.Season;
          status: Schemas.ContractStatus;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_GetMaintenanceContract = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/maintenance/contracts/{contract_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; contract_id: string };
    };
    responses: {
      200: {
        data: {
          amount_cents?: (number | null) | undefined;
          archived_at?: (string | null) | undefined;
          created_at: string;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          id: Schemas.MaintenanceContractId;
          lines: Array<Schemas.MaintenanceLine>;
          manager_member_id?: (null | Schemas.MemberId) | undefined;
          notes?: (string | null) | undefined;
          project_id: Schemas.ProjectId;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          reference?: (string | null) | undefined;
          season: Schemas.Season;
          status: Schemas.ContractStatus;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type put_UpdateMaintenanceContract = {
    method: "PUT";
    path: "/api/v1/organizations/{organization_id}/maintenance/contracts/{contract_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; contract_id: string };

      body: Schemas.ContractBody;
    };
    responses: {
      200: {
        data: {
          amount_cents?: (number | null) | undefined;
          archived_at?: (string | null) | undefined;
          created_at: string;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          id: Schemas.MaintenanceContractId;
          lines: Array<Schemas.MaintenanceLine>;
          manager_member_id?: (null | Schemas.MemberId) | undefined;
          notes?: (string | null) | undefined;
          project_id: Schemas.ProjectId;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          reference?: (string | null) | undefined;
          season: Schemas.Season;
          status: Schemas.ContractStatus;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type post_ArchiveMaintenanceContract = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/maintenance/contracts/{contract_id}/archive";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; contract_id: string };
    };
    responses: {
      200: {
        data: {
          amount_cents?: (number | null) | undefined;
          archived_at?: (string | null) | undefined;
          created_at: string;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          id: Schemas.MaintenanceContractId;
          lines: Array<Schemas.MaintenanceLine>;
          manager_member_id?: (null | Schemas.MemberId) | undefined;
          notes?: (string | null) | undefined;
          project_id: Schemas.ProjectId;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          reference?: (string | null) | undefined;
          season: Schemas.Season;
          status: Schemas.ContractStatus;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type post_DuplicateMaintenanceContract = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/maintenance/contracts/{contract_id}/duplicate";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; contract_id: string };
    };
    responses: {
      201: {
        data: {
          amount_cents?: (number | null) | undefined;
          archived_at?: (string | null) | undefined;
          created_at: string;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          id: Schemas.MaintenanceContractId;
          lines: Array<Schemas.MaintenanceLine>;
          manager_member_id?: (null | Schemas.MemberId) | undefined;
          notes?: (string | null) | undefined;
          project_id: Schemas.ProjectId;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          reference?: (string | null) | undefined;
          season: Schemas.Season;
          status: Schemas.ContractStatus;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_GetMaintenanceGrid = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/maintenance/grid";
    requestFormat: "json";
    parameters: {
      query: { season: number };
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: { contracts: Array<Schemas.GridContract>; year: number };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type get_ListMaintenanceInterventions = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/maintenance/interventions";
    requestFormat: "json";
    parameters: {
      query: Partial<{ status: "proposed" | "validated" | "rejected"; page: number; per_page: number }>;
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          created_at: string;
          done_on: string;
          id: Schemas.InterventionId;
          line_id: Schemas.MaintenanceLineId;
          note?: (string | null) | undefined;
          proposed_by: Schemas.MemberId;
          resolved_at?: (string | null) | undefined;
          resolved_by?: (null | Schemas.MemberId) | undefined;
          status: Schemas.InterventionStatus;
          task_id?: (null | Schemas.TaskId) | undefined;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_RejectMaintenanceIntervention = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/maintenance/interventions/{intervention_id}/reject";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; intervention_id: string };

      body: Schemas.RejectInterventionRequest;
    };
    responses: {
      200: {
        data: {
          created_at: string;
          done_on: string;
          id: Schemas.InterventionId;
          line_id: Schemas.MaintenanceLineId;
          note?: (string | null) | undefined;
          proposed_by: Schemas.MemberId;
          resolved_at?: (string | null) | undefined;
          resolved_by?: (null | Schemas.MemberId) | undefined;
          status: Schemas.InterventionStatus;
          task_id?: (null | Schemas.TaskId) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type post_ValidateMaintenanceIntervention = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/maintenance/interventions/{intervention_id}/validate";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; intervention_id: string };
    };
    responses: {
      200: {
        data: {
          created_at: string;
          done_on: string;
          id: Schemas.InterventionId;
          line_id: Schemas.MaintenanceLineId;
          note?: (string | null) | undefined;
          proposed_by: Schemas.MemberId;
          resolved_at?: (string | null) | undefined;
          resolved_by?: (null | Schemas.MemberId) | undefined;
          status: Schemas.InterventionStatus;
          task_id?: (null | Schemas.TaskId) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type post_RecordMaintenanceIntervention = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/maintenance/lines/{line_id}/interventions";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; line_id: string };

      body: Schemas.RecordInterventionRequest;
    };
    responses: {
      201: {
        data: {
          created_at: string;
          done_on: string;
          id: Schemas.InterventionId;
          line_id: Schemas.MaintenanceLineId;
          note?: (string | null) | undefined;
          proposed_by: Schemas.MemberId;
          resolved_at?: (string | null) | undefined;
          resolved_by?: (null | Schemas.MemberId) | undefined;
          status: Schemas.InterventionStatus;
          task_id?: (null | Schemas.TaskId) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_GetMaintenanceRecap = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/maintenance/recap";
    requestFormat: "json";
    parameters: {
      query: { week: string };
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: { groups: Array<Schemas.RecapGroup>; week: Schemas.IsoWeekNumber; year: number };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      409: unknown;
    };
  };
  export type get_ExportMaintenanceRecapPdf = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/maintenance/recap.pdf";
    requestFormat: "json";
    parameters: {
      query: { week: string };
      path: { organization_id: string };
    };
    responses: { 200: unknown; 400: unknown; 401: unknown; 403: unknown };
  };
  export type post_PlanMaintenanceLines = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/maintenance/recap/plan";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.PlanMaintenanceLinesRequest;
    };
    responses: {
      201: {
        data: Array<{
          all_day: boolean;
          assignments: Array<Schemas.TaskAssignmentSummary>;
          blocks_availability: boolean;
          child_count?: (number | null) | undefined;
          created_at: string;
          customer_context_id?: (null | Schemas.CustomerContextId) | undefined;
          customer_id?: (null | Schemas.CustomerId) | undefined;
          description?: (string | null) | undefined;
          ends_at?: (string | null) | undefined;
          equipment: Array<Schemas.TaskEquipmentResponse>;
          expenses_cents: number;
          expenses_label?: (string | null) | undefined;
          id: Schemas.TaskId;
          labels: Array<Schemas.TaskLabelResponse>;
          maintenance_line_id?: (null | Schemas.MaintenanceLineId) | undefined;
          member_ids: Array<Schemas.MemberId>;
          organization_id: Schemas.OrganizationId;
          parent_task_id?: (null | Schemas.TaskId) | undefined;
          project_id?: (null | Schemas.ProjectId) | undefined;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          recurrence_id?: (null | Schemas.TaskRecurrenceId) | undefined;
          starts_at?: (string | null) | undefined;
          status: Schemas.TaskStatus;
          title: string;
          updated_at: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_ListOrganizationMemberContracts = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/member-contracts";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          end_date?: (string | null) | undefined;
          id: Schemas.MemberContractId;
          member_id: Schemas.MemberId;
          organization_id: Schemas.OrganizationId;
          schedule: Schemas.ScheduleInput;
          start_date: string;
          trial_end_date?: (string | null) | undefined;
          type: Schemas.ContractType;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type get_ListMemberDocuments = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/member-documents";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          added_at: string;
          file_key: string;
          id: Schemas.MemberDocumentId;
          kind: Schemas.DocumentKind;
          member_id: Schemas.MemberId;
          name: string;
          organization_id: Schemas.OrganizationId;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type get_ListMemberProfiles = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/member-profiles";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          address: Schemas.MemberProfileAddressResponse;
          archived_at?: (string | null) | undefined;
          job_title?: (string | null) | undefined;
          landline?: (string | null) | undefined;
          manager_member_id?: (null | Schemas.MemberId) | undefined;
          member_id: Schemas.MemberId;
          mobile?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          personal_email?: (string | null) | undefined;
          work_email?: (string | null) | undefined;
          work_permit_end_date?: (string | null) | undefined;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type get_ListMembers = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/members";
    requestFormat: "json";
    parameters: {
      query: Partial<{ page: number; per_page: number }>;
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          account?: (null | Schemas.MemberAccountResponse) | undefined;
          created_at: string;
          display_name: string;
          first_name?: (string | null) | undefined;
          id: Schemas.MemberId;
          joined_at?: (string | null) | undefined;
          last_name: string;
          organization_id: Schemas.OrganizationId;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateMember = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/members";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateMemberRequest;
    };
    responses: {
      201: {
        data: {
          account?: (null | Schemas.MemberAccountResponse) | undefined;
          created_at: string;
          display_name: string;
          first_name?: (string | null) | undefined;
          id: Schemas.MemberId;
          joined_at?: (string | null) | undefined;
          last_name: string;
          organization_id: Schemas.OrganizationId;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      409: unknown;
    };
  };
  export type get_GetMyMember = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/members/me";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: { display_name: string; member_id: Schemas.MemberId; organization_id: Schemas.OrganizationId };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      404: unknown;
    };
  };
  export type get_GetMyPermissions = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/members/me/permissions";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: { data: { permissions: Array<string> }; pagination?: (null | Schemas.PaginationMetadata) | undefined };
      401: unknown;
      403: unknown;
    };
  };
  export type post_ArchiveMember = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/members/{member_id}/archive";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; member_id: string };

      body: Schemas.ArchiveMemberRequest;
    };
    responses: {
      200: {
        data: {
          address: Schemas.MemberProfileAddressResponse;
          archived_at?: (string | null) | undefined;
          job_title?: (string | null) | undefined;
          landline?: (string | null) | undefined;
          manager_member_id?: (null | Schemas.MemberId) | undefined;
          member_id: Schemas.MemberId;
          mobile?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          personal_email?: (string | null) | undefined;
          work_email?: (string | null) | undefined;
          work_permit_end_date?: (string | null) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      422: unknown;
    };
  };
  export type get_ListMemberContracts = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/members/{member_id}/contracts";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; member_id: string };
    };
    responses: {
      200: {
        data: Array<{
          end_date?: (string | null) | undefined;
          id: Schemas.MemberContractId;
          member_id: Schemas.MemberId;
          organization_id: Schemas.OrganizationId;
          schedule: Schemas.ScheduleInput;
          start_date: string;
          trial_end_date?: (string | null) | undefined;
          type: Schemas.ContractType;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type post_CreateMemberContract = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/members/{member_id}/contracts";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; member_id: string };

      body: Schemas.PutMemberContractRequest;
    };
    responses: {
      201: {
        data: {
          end_date?: (string | null) | undefined;
          id: Schemas.MemberContractId;
          member_id: Schemas.MemberId;
          organization_id: Schemas.OrganizationId;
          schedule: Schemas.ScheduleInput;
          start_date: string;
          trial_end_date?: (string | null) | undefined;
          type: Schemas.ContractType;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
      422: unknown;
    };
  };
  export type put_UpdateMemberContract = {
    method: "PUT";
    path: "/api/v1/organizations/{organization_id}/members/{member_id}/contracts/{contract_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; member_id: string; contract_id: string };

      body: Schemas.PutMemberContractRequest;
    };
    responses: {
      200: {
        data: {
          end_date?: (string | null) | undefined;
          id: Schemas.MemberContractId;
          member_id: Schemas.MemberId;
          organization_id: Schemas.OrganizationId;
          schedule: Schemas.ScheduleInput;
          start_date: string;
          trial_end_date?: (string | null) | undefined;
          type: Schemas.ContractType;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
      422: unknown;
    };
  };
  export type delete_DeleteMemberContract = {
    method: "DELETE";
    path: "/api/v1/organizations/{organization_id}/members/{member_id}/contracts/{contract_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; member_id: string; contract_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type put_PutDayOverride = {
    method: "PUT";
    path: "/api/v1/organizations/{organization_id}/members/{member_id}/day-overrides/{date}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; member_id: string; date: string };

      body: Schemas.PutDayOverrideRequest;
    };
    responses: {
      200: {
        data: {
          date: string;
          end?: (string | null) | undefined;
          id: Schemas.DayOverrideId;
          kind: Schemas.OverrideKind;
          lunch?: (null | Schemas.LunchInput) | undefined;
          member_id: Schemas.MemberId;
          note: string;
          organization_id: Schemas.OrganizationId;
          start?: (string | null) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      422: unknown;
    };
  };
  export type delete_DeleteDayOverride = {
    method: "DELETE";
    path: "/api/v1/organizations/{organization_id}/members/{member_id}/day-overrides/{date}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; member_id: string; date: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type post_AddMemberDocument = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/members/{member_id}/documents";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; member_id: string };

      body: Schemas.AddMemberDocumentRequest;
    };
    responses: {
      201: {
        data: {
          added_at: string;
          file_key: string;
          id: Schemas.MemberDocumentId;
          kind: Schemas.DocumentKind;
          member_id: Schemas.MemberId;
          name: string;
          organization_id: Schemas.OrganizationId;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      422: unknown;
    };
  };
  export type delete_DeleteMemberDocument = {
    method: "DELETE";
    path: "/api/v1/organizations/{organization_id}/members/{member_id}/documents/{document_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; member_id: string; document_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type get_GetMemberProfile = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/members/{member_id}/profile";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; member_id: string };
    };
    responses: {
      200: {
        data: {
          address: Schemas.MemberProfileAddressResponse;
          archived_at?: (string | null) | undefined;
          job_title?: (string | null) | undefined;
          landline?: (string | null) | undefined;
          manager_member_id?: (null | Schemas.MemberId) | undefined;
          member_id: Schemas.MemberId;
          mobile?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          personal_email?: (string | null) | undefined;
          work_email?: (string | null) | undefined;
          work_permit_end_date?: (string | null) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type put_PutMemberProfile = {
    method: "PUT";
    path: "/api/v1/organizations/{organization_id}/members/{member_id}/profile";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; member_id: string };

      body: Schemas.PutMemberProfileRequest;
    };
    responses: {
      200: {
        data: {
          address: Schemas.MemberProfileAddressResponse;
          archived_at?: (string | null) | undefined;
          job_title?: (string | null) | undefined;
          landline?: (string | null) | undefined;
          manager_member_id?: (null | Schemas.MemberId) | undefined;
          member_id: Schemas.MemberId;
          mobile?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          personal_email?: (string | null) | undefined;
          work_email?: (string | null) | undefined;
          work_permit_end_date?: (string | null) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      422: unknown;
    };
  };
  export type post_RestoreMember = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/members/{member_id}/restore";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; member_id: string };
    };
    responses: {
      200: {
        data: {
          address: Schemas.MemberProfileAddressResponse;
          archived_at?: (string | null) | undefined;
          job_title?: (string | null) | undefined;
          landline?: (string | null) | undefined;
          manager_member_id?: (null | Schemas.MemberId) | undefined;
          member_id: Schemas.MemberId;
          mobile?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          personal_email?: (string | null) | undefined;
          work_email?: (string | null) | undefined;
          work_permit_end_date?: (string | null) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type get_GetOrganizationNumbering = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/numbering";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          kind: Schemas.NumberSeriesKind;
          last_assigned_number: number;
          next_number: number;
          next_reference: string;
          padding: number;
          prefix: string;
          year: number;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type put_SetOrganizationNextNumber = {
    method: "PUT";
    path: "/api/v1/organizations/{organization_id}/numbering/{kind}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; kind: "QUOTE" | "INVOICE" | "CREDIT_NOTE" };

      body: Schemas.SetNextNumberRequest;
    };
    responses: {
      200: {
        data: {
          kind: Schemas.NumberSeriesKind;
          last_assigned_number: number;
          next_number: number;
          next_reference: string;
          padding: number;
          prefix: string;
          year: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      422: unknown;
    };
  };
  export type get_GetPlanning = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/planning";
    requestFormat: "json";
    parameters: {
      query: { from: string; to: string };
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: {
          entries: Array<Schemas.PlanningEntryResponse>;
          resources: Array<Schemas.PlanningResourceResponse>;
          timezone: string;
          work_time: Array<Schemas.PlanningWorkTimeResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
    };
  };
  export type get_GetPlanningAvailability = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/planning/availability";
    requestFormat: "json";
    parameters: {
      query: { starts_at: string; ends_at: string; all_day?: boolean | undefined };
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: { resources: Array<Schemas.AvailabilityResourceResponse> };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
    };
  };
  export type get_ListProducts = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/products";
    requestFormat: "json";
    parameters: {
      query: Partial<{ page: number; per_page: number }>;
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          archived_at?: (string | null) | undefined;
          billing_frequency: Schemas.BillingFrequency;
          category: string;
          cost_cents?: (number | null) | undefined;
          created_at: string;
          datasheet_keys: Array<string>;
          default_vat_rate_bp?: (number | null) | undefined;
          description?: (string | null) | undefined;
          id: Schemas.ProductId;
          invoice_usage_count: number;
          manufacturer?: (string | null) | undefined;
          name: string;
          organization_id: Schemas.OrganizationId;
          photo_keys: Array<string>;
          quote_usage_count: number;
          sku?: (string | null) | undefined;
          supplier_id?: (null | Schemas.SupplierId) | undefined;
          technical_specs?: (string | null) | undefined;
          unit: string;
          unit_price_cents: number;
          updated_at: string;
          usage_count: number;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateProduct = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/products";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateProductRequest;
    };
    responses: {
      201: {
        data: {
          archived_at?: (string | null) | undefined;
          billing_frequency: Schemas.BillingFrequency;
          category: string;
          cost_cents?: (number | null) | undefined;
          created_at: string;
          datasheet_keys: Array<string>;
          default_vat_rate_bp?: (number | null) | undefined;
          description?: (string | null) | undefined;
          id: Schemas.ProductId;
          invoice_usage_count: number;
          manufacturer?: (string | null) | undefined;
          name: string;
          organization_id: Schemas.OrganizationId;
          photo_keys: Array<string>;
          quote_usage_count: number;
          sku?: (string | null) | undefined;
          supplier_id?: (null | Schemas.SupplierId) | undefined;
          technical_specs?: (string | null) | undefined;
          unit: string;
          unit_price_cents: number;
          updated_at: string;
          usage_count: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      409: unknown;
    };
  };
  export type get_ListProjectTemplates = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/project-templates";
    requestFormat: "json";
    parameters: {
      query: Partial<{ page: number; per_page: number; include_archived: boolean }>;
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          archived_at?: (string | null) | undefined;
          created_at: string;
          description?: (string | null) | undefined;
          id: Schemas.ProjectTemplateId;
          name: string;
          organization_id: Schemas.OrganizationId;
          tasks?: (Array<Schemas.ProjectTemplateTaskResponse> | null) | undefined;
          updated_at: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateProjectTemplate = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/project-templates";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateProjectTemplateRequest;
    };
    responses: {
      201: {
        data: {
          archived_at?: (string | null) | undefined;
          created_at: string;
          description?: (string | null) | undefined;
          id: Schemas.ProjectTemplateId;
          name: string;
          organization_id: Schemas.OrganizationId;
          tasks?: (Array<Schemas.ProjectTemplateTaskResponse> | null) | undefined;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      409: unknown;
    };
  };
  export type get_GetProjectTemplate = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; project_template_id: string };
    };
    responses: {
      200: {
        data: {
          archived_at?: (string | null) | undefined;
          created_at: string;
          description?: (string | null) | undefined;
          id: Schemas.ProjectTemplateId;
          name: string;
          organization_id: Schemas.OrganizationId;
          tasks?: (Array<Schemas.ProjectTemplateTaskResponse> | null) | undefined;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type delete_ArchiveProjectTemplate = {
    method: "DELETE";
    path: "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; project_template_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type patch_PatchProjectTemplate = {
    method: "PATCH";
    path: "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; project_template_id: string };

      body: Schemas.UpdateProjectTemplateRequest;
    };
    responses: {
      200: {
        data: {
          archived_at?: (string | null) | undefined;
          created_at: string;
          description?: (string | null) | undefined;
          id: Schemas.ProjectTemplateId;
          name: string;
          organization_id: Schemas.OrganizationId;
          tasks?: (Array<Schemas.ProjectTemplateTaskResponse> | null) | undefined;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type post_InstantiateProjectTemplate = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}/instantiate";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; project_template_id: string };

      body: Schemas.InstantiateProjectTemplateRequest;
    };
    responses: {
      201: {
        data: { project: Schemas.ProjectResponse; tasks: Array<Schemas.TaskResponse> };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type post_RestoreProjectTemplate = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}/restore";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; project_template_id: string };
    };
    responses: {
      200: {
        data: {
          archived_at?: (string | null) | undefined;
          created_at: string;
          description?: (string | null) | undefined;
          id: Schemas.ProjectTemplateId;
          name: string;
          organization_id: Schemas.OrganizationId;
          tasks?: (Array<Schemas.ProjectTemplateTaskResponse> | null) | undefined;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type put_ReplaceProjectTemplateTasks = {
    method: "PUT";
    path: "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}/tasks";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; project_template_id: string };

      body: Schemas.ReplaceProjectTemplateTasksRequest;
    };
    responses: {
      200: {
        data: Array<{
          all_day: boolean;
          blocks_availability: boolean;
          day_offset: number;
          description?: (string | null) | undefined;
          ends_minute?: (number | null) | undefined;
          expenses_cents: number;
          expenses_label?: (string | null) | undefined;
          id: Schemas.ProjectTemplateTaskId;
          organization_id: Schemas.OrganizationId;
          parent_index?: (number | null) | undefined;
          position: number;
          starts_minute?: (number | null) | undefined;
          template_id: Schemas.ProjectTemplateId;
          title: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_ListProjects = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/projects";
    requestFormat: "json";
    parameters: {
      query: Partial<{
        page: number;
        per_page: number;
        customer_id: string;
        quote_id: string;
        include_archived: boolean;
      }>;
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          archived_at?: (string | null) | undefined;
          created_at: string;
          customer_context_id?: (null | Schemas.CustomerContextId) | undefined;
          customer_id?: (null | Schemas.CustomerId) | undefined;
          id: Schemas.ProjectId;
          is_internal: boolean;
          name: string;
          organization_id: Schemas.OrganizationId;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          updated_at: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateProject = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/projects";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateProjectRequest;
    };
    responses: {
      201: {
        data: {
          archived_at?: (string | null) | undefined;
          created_at: string;
          customer_context_id?: (null | Schemas.CustomerContextId) | undefined;
          customer_id?: (null | Schemas.CustomerId) | undefined;
          id: Schemas.ProjectId;
          is_internal: boolean;
          name: string;
          organization_id: Schemas.OrganizationId;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      409: unknown;
    };
  };
  export type get_GetProject = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/projects/{project_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; project_id: string };
    };
    responses: {
      200: {
        data: {
          archived_at?: (string | null) | undefined;
          created_at: string;
          customer_context_id?: (null | Schemas.CustomerContextId) | undefined;
          customer_id?: (null | Schemas.CustomerId) | undefined;
          id: Schemas.ProjectId;
          is_internal: boolean;
          name: string;
          organization_id: Schemas.OrganizationId;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type delete_ArchiveProject = {
    method: "DELETE";
    path: "/api/v1/organizations/{organization_id}/projects/{project_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; project_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type patch_PatchProject = {
    method: "PATCH";
    path: "/api/v1/organizations/{organization_id}/projects/{project_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; project_id: string };

      body: Schemas.UpdateProjectRequest;
    };
    responses: {
      200: {
        data: {
          archived_at?: (string | null) | undefined;
          created_at: string;
          customer_context_id?: (null | Schemas.CustomerContextId) | undefined;
          customer_id?: (null | Schemas.CustomerId) | undefined;
          id: Schemas.ProjectId;
          is_internal: boolean;
          name: string;
          organization_id: Schemas.OrganizationId;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_GetProjectChannel = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/projects/{project_id}/channel";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; project_id: string };
    };
    responses: {
      200: {
        data: {
          archived: boolean;
          created_at: string;
          id: Schemas.ChannelId;
          name: string;
          organization_id: Schemas.OrganizationId;
          project_id?: (null | Schemas.ProjectId) | undefined;
          topic?: (string | null) | undefined;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type post_CreateProjectChannel = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/projects/{project_id}/channel";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; project_id: string };

      body: Schemas.CreateProjectChannelRequest;
    };
    responses: {
      201: {
        data: {
          archived: boolean;
          created_at: string;
          id: Schemas.ChannelId;
          name: string;
          organization_id: Schemas.OrganizationId;
          project_id?: (null | Schemas.ProjectId) | undefined;
          topic?: (string | null) | undefined;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_GetProjectDoe = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/projects/{project_id}/doe";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; project_id: string };
    };
    responses: {
      200: {
        data: {
          author?: (string | null) | undefined;
          cover_photo_key?: (string | null) | undefined;
          created_at: string;
          id: Schemas.DoeId;
          issued_on: string;
          lot?: (string | null) | undefined;
          model_id?: (null | Schemas.DoeModelId) | undefined;
          parties: Array<Schemas.DoeParty>;
          place?: (string | null) | undefined;
          project_id: Schemas.ProjectId;
          revision: Schemas.Revision;
          sections: Array<Schemas.DoeSection>;
          title: string;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type post_CreateDoe = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/projects/{project_id}/doe";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; project_id: string };

      body: Schemas.CreateDoeRequest;
    };
    responses: {
      201: {
        data: {
          author?: (string | null) | undefined;
          cover_photo_key?: (string | null) | undefined;
          created_at: string;
          id: Schemas.DoeId;
          issued_on: string;
          lot?: (string | null) | undefined;
          model_id?: (null | Schemas.DoeModelId) | undefined;
          parties: Array<Schemas.DoeParty>;
          place?: (string | null) | undefined;
          project_id: Schemas.ProjectId;
          revision: Schemas.Revision;
          sections: Array<Schemas.DoeSection>;
          title: string;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type post_RestoreProject = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/projects/{project_id}/restore";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; project_id: string };
    };
    responses: {
      200: {
        data: {
          archived_at?: (string | null) | undefined;
          created_at: string;
          customer_context_id?: (null | Schemas.CustomerContextId) | undefined;
          customer_id?: (null | Schemas.CustomerId) | undefined;
          id: Schemas.ProjectId;
          is_internal: boolean;
          name: string;
          organization_id: Schemas.OrganizationId;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type get_ListQuotes = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/quotes";
    requestFormat: "json";
    parameters: {
      query: Partial<{ page: number; per_page: number; deal_id: string }>;
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          billing_address_id?: (null | Schemas.CustomerContextId) | undefined;
          billing_contact_id?: (null | Schemas.CustomerContactId) | undefined;
          created_at: string;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          deal_id?: (null | Schemas.DealId) | undefined;
          decline_reason?: (string | null) | undefined;
          deposit_percent: string;
          gross_cents: number;
          id: Schemas.QuoteId;
          is_expired: boolean;
          issuer_snapshot?: (null | Schemas.IssuerSnapshot) | undefined;
          lines: Array<Schemas.QuoteLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          owner_member_id?: (null | Schemas.MemberId) | undefined;
          recipient_snapshot?: (null | Schemas.RecipientSnapshot) | undefined;
          recurring: Array<Schemas.RecurringTotalResponse>;
          reference?: (string | null) | undefined;
          responded_at?: (string | null) | undefined;
          responded_by?: (string | null) | undefined;
          revision_of_id?: (null | Schemas.QuoteId) | undefined;
          status: Schemas.QuoteStatus;
          superseded_by_id?: (null | Schemas.QuoteId) | undefined;
          title: string;
          updated_at: string;
          valid_until?: (string | null) | undefined;
          vat_breakdown: Array<Schemas.QuoteVatBreakdownLineResponse>;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateQuote = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/quotes";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateQuoteRequest;
    };
    responses: {
      201: {
        data: {
          billing_address_id?: (null | Schemas.CustomerContextId) | undefined;
          billing_contact_id?: (null | Schemas.CustomerContactId) | undefined;
          created_at: string;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          deal_id?: (null | Schemas.DealId) | undefined;
          decline_reason?: (string | null) | undefined;
          deposit_percent: string;
          gross_cents: number;
          id: Schemas.QuoteId;
          is_expired: boolean;
          issuer_snapshot?: (null | Schemas.IssuerSnapshot) | undefined;
          lines: Array<Schemas.QuoteLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          owner_member_id?: (null | Schemas.MemberId) | undefined;
          recipient_snapshot?: (null | Schemas.RecipientSnapshot) | undefined;
          recurring: Array<Schemas.RecurringTotalResponse>;
          reference?: (string | null) | undefined;
          responded_at?: (string | null) | undefined;
          responded_by?: (string | null) | undefined;
          revision_of_id?: (null | Schemas.QuoteId) | undefined;
          status: Schemas.QuoteStatus;
          superseded_by_id?: (null | Schemas.QuoteId) | undefined;
          title: string;
          updated_at: string;
          valid_until?: (string | null) | undefined;
          vat_breakdown: Array<Schemas.QuoteVatBreakdownLineResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      409: unknown;
      422: unknown;
    };
  };
  export type post_RecordReceipt = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/receipts";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.RecordReceiptRequest;
    };
    responses: {
      201: {
        data: {
          amount_cents: number;
          created_at: string;
          customer_id?: (null | Schemas.CustomerId) | undefined;
          id: Schemas.ReceiptId;
          method: Schemas.PaymentMethod;
          note?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          paid_on: string;
          reference?: (string | null) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      422: unknown;
    };
  };
  export type get_GetProfitability = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/reporting/profitability";
    requestFormat: "json";
    parameters: {
      query: { from: string; to: string };
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: {
          costs_redacted: boolean;
          double_booked: Array<Schemas.ProjectProfitabilityResponse>;
          incomplete: Array<Schemas.ProjectProfitabilityResponse>;
          least_profitable: Array<Schemas.ProjectProfitabilityResponse>;
          members: Array<Schemas.MemberProfitabilityResponse>;
          most_profitable: Array<Schemas.ProjectProfitabilityResponse>;
          projects: Array<Schemas.ProjectProfitabilityResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      409: unknown;
    };
  };
  export type get_GetWorkedHours = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/reporting/worked-hours";
    requestFormat: "json";
    parameters: {
      query: { from: string; to: string };
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: { costs_redacted: boolean; members: Array<Schemas.WorkedHoursRow>; total_planned_minutes: number };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      409: unknown;
    };
  };
  export type get_ListRoles = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/roles";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          created_at: string;
          id: Schemas.RoleId;
          is_seeded: boolean;
          name: string;
          organization_id: Schemas.OrganizationId;
          permissions: Array<string>;
          updated_at: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateRole = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/roles";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateRoleRequest;
    };
    responses: {
      201: {
        data: {
          created_at: string;
          id: Schemas.RoleId;
          is_seeded: boolean;
          name: string;
          organization_id: Schemas.OrganizationId;
          permissions: Array<string>;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      409: unknown;
    };
  };
  export type get_ListServiceRates = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/service-rates";
    requestFormat: "json";
    parameters: {
      query: Partial<{ page: number; per_page: number }>;
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          archived_at?: (string | null) | undefined;
          billing_frequency: Schemas.BillingFrequency;
          category: string;
          cost_cents?: (number | null) | undefined;
          created_at: string;
          default_vat_rate_bp?: (number | null) | undefined;
          description?: (string | null) | undefined;
          id: Schemas.ServiceRateId;
          invoice_usage_count: number;
          label: string;
          organization_id: Schemas.OrganizationId;
          quote_usage_count: number;
          rate_cents: number;
          unit: string;
          updated_at: string;
          usage_count: number;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateServiceRate = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/service-rates";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateServiceRateRequest;
    };
    responses: {
      201: {
        data: {
          archived_at?: (string | null) | undefined;
          billing_frequency: Schemas.BillingFrequency;
          category: string;
          cost_cents?: (number | null) | undefined;
          created_at: string;
          default_vat_rate_bp?: (number | null) | undefined;
          description?: (string | null) | undefined;
          id: Schemas.ServiceRateId;
          invoice_usage_count: number;
          label: string;
          organization_id: Schemas.OrganizationId;
          quote_usage_count: number;
          rate_cents: number;
          unit: string;
          updated_at: string;
          usage_count: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      409: unknown;
    };
  };
  export type get_ListSupplierInvoices = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/supplier-invoices";
    requestFormat: "json";
    parameters: {
      query: Partial<{ page: number; per_page: number }>;
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          created_at: string;
          currency: string;
          due_on?: (string | null) | undefined;
          gross_cents: number;
          id: Schemas.SupplierInvoiceId;
          issued_on: string;
          lines: Array<Schemas.SupplierInvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number: string;
          organization_id: Schemas.OrganizationId;
          received_at: string;
          source: Schemas.SupplierInvoiceSource;
          source_file_key?: (string | null) | undefined;
          source_file_mime_type?: (string | null) | undefined;
          status: Schemas.SupplierInvoiceStatus;
          supplier_id?: (null | Schemas.SupplierId) | undefined;
          supplier_name: string;
          supplier_registration_number?: (string | null) | undefined;
          supplier_vat_number?: (string | null) | undefined;
          updated_at: string;
          vat_breakdown: Array<Schemas.SupplierInvoiceVatBreakdownLineResponse>;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_ImportSupplierInvoice = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/supplier-invoices/import";
    requestFormat: "binary";
    parameters: {
      path: { organization_id: string };

      body: Array<number>;
    };
    responses: {
      200: {
        data:
          | {
              invoice: Schemas.SupplierInvoiceResponse;
              outcome: "created";
              totals_mismatch?: (null | Schemas.TotalsMismatchResponse) | undefined;
            }
          | { outcome: "parse_failed"; reason: string };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      201: {
        data:
          | {
              invoice: Schemas.SupplierInvoiceResponse;
              outcome: "created";
              totals_mismatch?: (null | Schemas.TotalsMismatchResponse) | undefined;
            }
          | { outcome: "parse_failed"; reason: string };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      409: unknown;
      413: unknown;
    };
  };
  export type get_ListSuppliers = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/suppliers";
    requestFormat: "json";
    parameters: {
      query: Partial<{ page: number; per_page: number }>;
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          address?: (string | null) | undefined;
          addresses: Array<Schemas.SupplierAddressResponse>;
          archived_at?: (string | null) | undefined;
          contacts: Array<Schemas.SupplierContactResponse>;
          created_at: string;
          default_category_id?: (null | Schemas.ExpenseCategoryId) | undefined;
          email?: (string | null) | undefined;
          iban?: (string | null) | undefined;
          id: Schemas.SupplierId;
          name: string;
          notes?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          payment_terms_days?: (number | null) | undefined;
          phone?: (string | null) | undefined;
          registration_number?: (string | null) | undefined;
          updated_at: string;
          vat_number?: (string | null) | undefined;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateSupplier = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/suppliers";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateSupplierRequest;
    };
    responses: {
      201: {
        data: {
          address?: (string | null) | undefined;
          addresses: Array<Schemas.SupplierAddressResponse>;
          archived_at?: (string | null) | undefined;
          contacts: Array<Schemas.SupplierContactResponse>;
          created_at: string;
          default_category_id?: (null | Schemas.ExpenseCategoryId) | undefined;
          email?: (string | null) | undefined;
          iban?: (string | null) | undefined;
          id: Schemas.SupplierId;
          name: string;
          notes?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          payment_terms_days?: (number | null) | undefined;
          phone?: (string | null) | undefined;
          registration_number?: (string | null) | undefined;
          updated_at: string;
          vat_number?: (string | null) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      409: unknown;
    };
  };
  export type get_GetSupplier = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/suppliers/{supplier_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; supplier_id: string };
    };
    responses: {
      200: {
        data: {
          address?: (string | null) | undefined;
          addresses: Array<Schemas.SupplierAddressResponse>;
          archived_at?: (string | null) | undefined;
          contacts: Array<Schemas.SupplierContactResponse>;
          created_at: string;
          default_category_id?: (null | Schemas.ExpenseCategoryId) | undefined;
          email?: (string | null) | undefined;
          iban?: (string | null) | undefined;
          id: Schemas.SupplierId;
          name: string;
          notes?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          payment_terms_days?: (number | null) | undefined;
          phone?: (string | null) | undefined;
          registration_number?: (string | null) | undefined;
          updated_at: string;
          vat_number?: (string | null) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type put_UpdateSupplier = {
    method: "PUT";
    path: "/api/v1/organizations/{organization_id}/suppliers/{supplier_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; supplier_id: string };

      body: Schemas.UpdateSupplierRequest;
    };
    responses: {
      200: {
        data: {
          address?: (string | null) | undefined;
          addresses: Array<Schemas.SupplierAddressResponse>;
          archived_at?: (string | null) | undefined;
          contacts: Array<Schemas.SupplierContactResponse>;
          created_at: string;
          default_category_id?: (null | Schemas.ExpenseCategoryId) | undefined;
          email?: (string | null) | undefined;
          iban?: (string | null) | undefined;
          id: Schemas.SupplierId;
          name: string;
          notes?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          payment_terms_days?: (number | null) | undefined;
          phone?: (string | null) | undefined;
          registration_number?: (string | null) | undefined;
          updated_at: string;
          vat_number?: (string | null) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type post_ArchiveSupplier = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/suppliers/{supplier_id}/archive";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; supplier_id: string };
    };
    responses: {
      200: {
        data: {
          address?: (string | null) | undefined;
          addresses: Array<Schemas.SupplierAddressResponse>;
          archived_at?: (string | null) | undefined;
          contacts: Array<Schemas.SupplierContactResponse>;
          created_at: string;
          default_category_id?: (null | Schemas.ExpenseCategoryId) | undefined;
          email?: (string | null) | undefined;
          iban?: (string | null) | undefined;
          id: Schemas.SupplierId;
          name: string;
          notes?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          payment_terms_days?: (number | null) | undefined;
          phone?: (string | null) | undefined;
          registration_number?: (string | null) | undefined;
          updated_at: string;
          vat_number?: (string | null) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type post_RestoreSupplier = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/suppliers/{supplier_id}/restore";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; supplier_id: string };
    };
    responses: {
      200: {
        data: {
          address?: (string | null) | undefined;
          addresses: Array<Schemas.SupplierAddressResponse>;
          archived_at?: (string | null) | undefined;
          contacts: Array<Schemas.SupplierContactResponse>;
          created_at: string;
          default_category_id?: (null | Schemas.ExpenseCategoryId) | undefined;
          email?: (string | null) | undefined;
          iban?: (string | null) | undefined;
          id: Schemas.SupplierId;
          name: string;
          notes?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          payment_terms_days?: (number | null) | undefined;
          phone?: (string | null) | undefined;
          registration_number?: (string | null) | undefined;
          updated_at: string;
          vat_number?: (string | null) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type get_ListSupportApiKeys = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/support/api-keys";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          created_at: string;
          id: Schemas.SupportApiKeyId;
          last_used_at?: (string | null) | undefined;
          name: string;
          prefix: string;
          revoked_at?: (string | null) | undefined;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateSupportApiKey = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/support/api-keys";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateSupportApiKeyRequest;
    };
    responses: {
      201: {
        data: { created_at: string; id: Schemas.SupportApiKeyId; key: string; name: string; prefix: string };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      409: unknown;
    };
  };
  export type delete_RevokeSupportApiKey = {
    method: "DELETE";
    path: "/api/v1/organizations/{organization_id}/support/api-keys/{key_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; key_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type get_GetSupportAttachmentUrl = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/support/attachments/{attachment_id}/url";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; attachment_id: string };
    };
    responses: {
      200: { data: { expires_at: string; url: string }; pagination?: (null | Schemas.PaginationMetadata) | undefined };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type delete_DeleteSupportDraft = {
    method: "DELETE";
    path: "/api/v1/organizations/{organization_id}/support/entries/{entry_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; entry_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown; 409: unknown };
  };
  export type patch_EditSupportDraft = {
    method: "PATCH";
    path: "/api/v1/organizations/{organization_id}/support/entries/{entry_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; entry_id: string };

      body: Schemas.EditSupportDraftRequest;
    };
    responses: {
      200: {
        data: {
          attachments: Array<Schemas.SupportAttachmentResponse>;
          author_member_id?: (null | Schemas.MemberId) | undefined;
          body: string;
          created_at: string;
          id: Schemas.TicketEntryId;
          kind: Schemas.EntryKind;
          ticket_id: Schemas.TicketId;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type post_AttachSupportEntryFile = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/support/entries/{entry_id}/attachments";
    requestFormat: "binary";
    parameters: {
      path: { organization_id: string; entry_id: string };
      header: { "X-File-Name": string };
      body: Array<number>;
    };
    responses: {
      201: {
        data: {
          created_at: string;
          file_name: string;
          id: Schemas.AttachmentId;
          mime_type: string;
          size_bytes: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
      413: unknown;
      422: unknown;
    };
  };
  export type post_SendSupportDraft = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/support/entries/{entry_id}/send";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; entry_id: string };
    };
    responses: {
      200: {
        data: {
          attachments: Array<Schemas.SupportAttachmentResponse>;
          author_member_id?: (null | Schemas.MemberId) | undefined;
          body: string;
          created_at: string;
          id: Schemas.TicketEntryId;
          kind: Schemas.EntryKind;
          ticket_id: Schemas.TicketId;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_GetSupportSettings = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/support/settings";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: { data: { display_fields: Array<string> }; pagination?: (null | Schemas.PaginationMetadata) | undefined };
      401: unknown;
      403: unknown;
    };
  };
  export type put_ReplaceSupportSettings = {
    method: "PUT";
    path: "/api/v1/organizations/{organization_id}/support/settings";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.ReplaceSupportSettingsRequest;
    };
    responses: {
      200: { data: { display_fields: Array<string> }; pagination?: (null | Schemas.PaginationMetadata) | undefined };
      401: unknown;
      403: unknown;
      422: unknown;
    };
  };
  export type get_ListSupportTickets = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/support/tickets";
    requestFormat: "json";
    parameters: {
      query: Partial<{
        status: string;
        assignee: string;
        badge: string;
        custom_field_key: string;
        custom_field_value: string;
        page: number;
        per_page: number;
      }>;
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          assigned_at?: (string | null) | undefined;
          assignee_member_id?: (null | Schemas.MemberId) | undefined;
          badges: Array<Schemas.BadgeResponse>;
          body: string;
          created_at: string;
          custom_fields: Record<string, unknown>;
          external_id?: (string | null) | undefined;
          id: Schemas.TicketId;
          number: Schemas.TicketNumber;
          status: Schemas.TicketStatus;
          subject: string;
          updated_at: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      422: unknown;
    };
  };
  export type get_GetSupportTicket = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; ticket_id: string };
    };
    responses: {
      200: {
        data: {
          assigned_at?: (string | null) | undefined;
          assignee_member_id?: (null | Schemas.MemberId) | undefined;
          badges: Array<Schemas.BadgeResponse>;
          body: string;
          created_at: string;
          custom_fields: Record<string, unknown>;
          external_id?: (string | null) | undefined;
          id: Schemas.TicketId;
          number: Schemas.TicketNumber;
          status: Schemas.TicketStatus;
          subject: string;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type put_AssignSupportTicket = {
    method: "PUT";
    path: "/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/assignee";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; ticket_id: string };

      body: Schemas.AssignTicketRequest;
    };
    responses: {
      200: {
        data: {
          assigned_at?: (string | null) | undefined;
          assignee_member_id?: (null | Schemas.MemberId) | undefined;
          badges: Array<Schemas.BadgeResponse>;
          body: string;
          created_at: string;
          custom_fields: Record<string, unknown>;
          external_id?: (string | null) | undefined;
          id: Schemas.TicketId;
          number: Schemas.TicketNumber;
          status: Schemas.TicketStatus;
          subject: string;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type post_AddSupportTicketBadge = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/badges";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; ticket_id: string };

      body: Schemas.AddTicketBadgeRequest;
    };
    responses: {
      200: {
        data: {
          assigned_at?: (string | null) | undefined;
          assignee_member_id?: (null | Schemas.MemberId) | undefined;
          badges: Array<Schemas.BadgeResponse>;
          body: string;
          created_at: string;
          custom_fields: Record<string, unknown>;
          external_id?: (string | null) | undefined;
          id: Schemas.TicketId;
          number: Schemas.TicketNumber;
          status: Schemas.TicketStatus;
          subject: string;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
      422: unknown;
    };
  };
  export type delete_RemoveSupportTicketBadge = {
    method: "DELETE";
    path: "/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/badges/{label}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; ticket_id: string; label: string };
    };
    responses: {
      200: {
        data: {
          assigned_at?: (string | null) | undefined;
          assignee_member_id?: (null | Schemas.MemberId) | undefined;
          badges: Array<Schemas.BadgeResponse>;
          body: string;
          created_at: string;
          custom_fields: Record<string, unknown>;
          external_id?: (string | null) | undefined;
          id: Schemas.TicketId;
          number: Schemas.TicketNumber;
          status: Schemas.TicketStatus;
          subject: string;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type get_ListSupportTicketEntries = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/entries";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; ticket_id: string };
    };
    responses: {
      200: {
        data: Array<{
          attachments: Array<Schemas.SupportAttachmentResponse>;
          author_member_id?: (null | Schemas.MemberId) | undefined;
          body: string;
          created_at: string;
          id: Schemas.TicketEntryId;
          kind: Schemas.EntryKind;
          ticket_id: Schemas.TicketId;
          updated_at: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type post_AddSupportTicketEntry = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/entries";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; ticket_id: string };

      body: Schemas.AddTicketEntryRequest;
    };
    responses: {
      201: {
        data: {
          attachments: Array<Schemas.SupportAttachmentResponse>;
          author_member_id?: (null | Schemas.MemberId) | undefined;
          body: string;
          created_at: string;
          id: Schemas.TicketEntryId;
          kind: Schemas.EntryKind;
          ticket_id: Schemas.TicketId;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
      422: unknown;
    };
  };
  export type patch_TransitionSupportTicket = {
    method: "PATCH";
    path: "/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/status";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; ticket_id: string };

      body: Schemas.TransitionTicketRequest;
    };
    responses: {
      200: {
        data: {
          assigned_at?: (string | null) | undefined;
          assignee_member_id?: (null | Schemas.MemberId) | undefined;
          badges: Array<Schemas.BadgeResponse>;
          body: string;
          created_at: string;
          custom_fields: Record<string, unknown>;
          external_id?: (string | null) | undefined;
          id: Schemas.TicketId;
          number: Schemas.TicketNumber;
          status: Schemas.TicketStatus;
          subject: string;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      422: unknown;
    };
  };
  export type get_ListSurcharges = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/surcharges";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          archived_at?: (string | null) | undefined;
          created_at: string;
          id: Schemas.SurchargeId;
          kind: Schemas.SurchargeKind;
          name: string;
          organization_id: Schemas.OrganizationId;
          rate_bp: number;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateSurcharge = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/surcharges";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.SurchargeRequest;
    };
    responses: {
      201: {
        data: {
          archived_at?: (string | null) | undefined;
          created_at: string;
          id: Schemas.SurchargeId;
          kind: Schemas.SurchargeKind;
          name: string;
          organization_id: Schemas.OrganizationId;
          rate_bp: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      422: unknown;
    };
  };
  export type put_UpdateSurcharge = {
    method: "PUT";
    path: "/api/v1/organizations/{organization_id}/surcharges/{surcharge_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; surcharge_id: string };

      body: Schemas.SurchargeRequest;
    };
    responses: {
      200: {
        data: {
          archived_at?: (string | null) | undefined;
          created_at: string;
          id: Schemas.SurchargeId;
          kind: Schemas.SurchargeKind;
          name: string;
          organization_id: Schemas.OrganizationId;
          rate_bp: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      422: unknown;
    };
  };
  export type post_ArchiveSurcharge = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/surcharges/{surcharge_id}/archive";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; surcharge_id: string };
    };
    responses: {
      200: {
        data: {
          archived_at?: (string | null) | undefined;
          created_at: string;
          id: Schemas.SurchargeId;
          kind: Schemas.SurchargeKind;
          name: string;
          organization_id: Schemas.OrganizationId;
          rate_bp: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type post_RestoreSurcharge = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/surcharges/{surcharge_id}/restore";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; surcharge_id: string };
    };
    responses: {
      200: {
        data: {
          archived_at?: (string | null) | undefined;
          created_at: string;
          id: Schemas.SurchargeId;
          kind: Schemas.SurchargeKind;
          name: string;
          organization_id: Schemas.OrganizationId;
          rate_bp: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type get_ListTaskLabels = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/task-labels";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          color: string;
          created_at: string;
          id: Schemas.TaskLabelId;
          name: string;
          organization_id: Schemas.OrganizationId;
          updated_at: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateTaskLabel = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/task-labels";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateTaskLabelRequest;
    };
    responses: {
      201: {
        data: {
          color: string;
          created_at: string;
          id: Schemas.TaskLabelId;
          name: string;
          organization_id: Schemas.OrganizationId;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      409: unknown;
    };
  };
  export type delete_DeleteTaskLabel = {
    method: "DELETE";
    path: "/api/v1/organizations/{organization_id}/task-labels/{label_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; label_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type patch_UpdateTaskLabel = {
    method: "PATCH";
    path: "/api/v1/organizations/{organization_id}/task-labels/{label_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; label_id: string };

      body: Schemas.UpdateTaskLabelRequest;
    };
    responses: {
      200: {
        data: {
          color: string;
          created_at: string;
          id: Schemas.TaskLabelId;
          name: string;
          organization_id: Schemas.OrganizationId;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_ListTaskRecurrences = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/task-recurrences";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<
          Schemas.RecurrenceRuleResponse & {
            all_day: boolean;
            assignee_member_ids: Array<Schemas.MemberId>;
            blocks_availability: boolean;
            created_at: string;
            customer_context_id?: (null | Schemas.CustomerContextId) | undefined;
            customer_id?: (null | Schemas.CustomerId) | undefined;
            description?: (string | null) | undefined;
            duration_minutes: number;
            ends_on?: (string | null) | undefined;
            horizon_filled_to: string;
            id: Schemas.TaskRecurrenceId;
            organization_id: Schemas.OrganizationId;
            project_id?: (null | Schemas.ProjectId) | undefined;
            start_time: string;
            starts_on: string;
            timezone: string;
            title: string;
            updated_at: string;
          }
        >;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateTaskRecurrence = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/task-recurrences";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateTaskRecurrenceRequest;
    };
    responses: {
      201: {
        data: Schemas.RecurrenceRuleResponse & {
          all_day: boolean;
          assignee_member_ids: Array<Schemas.MemberId>;
          blocks_availability: boolean;
          created_at: string;
          customer_context_id?: (null | Schemas.CustomerContextId) | undefined;
          customer_id?: (null | Schemas.CustomerId) | undefined;
          description?: (string | null) | undefined;
          duration_minutes: number;
          ends_on?: (string | null) | undefined;
          horizon_filled_to: string;
          id: Schemas.TaskRecurrenceId;
          organization_id: Schemas.OrganizationId;
          project_id?: (null | Schemas.ProjectId) | undefined;
          start_time: string;
          starts_on: string;
          timezone: string;
          title: string;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_ListTasks = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/tasks";
    requestFormat: "json";
    parameters: {
      query: Partial<{
        page: number;
        per_page: number;
        parent_task_id: string;
        project_id: string;
        status: Array<Schemas.TaskStatus>;
        assignee_id: string;
        label_id: string;
        customer_id: string;
        q: string;
        unscheduled: boolean;
      }>;
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          all_day: boolean;
          assignments: Array<Schemas.TaskAssignmentSummary>;
          blocks_availability: boolean;
          child_count?: (number | null) | undefined;
          created_at: string;
          customer_context_id?: (null | Schemas.CustomerContextId) | undefined;
          customer_id?: (null | Schemas.CustomerId) | undefined;
          description?: (string | null) | undefined;
          ends_at?: (string | null) | undefined;
          equipment: Array<Schemas.TaskEquipmentResponse>;
          expenses_cents: number;
          expenses_label?: (string | null) | undefined;
          id: Schemas.TaskId;
          labels: Array<Schemas.TaskLabelResponse>;
          maintenance_line_id?: (null | Schemas.MaintenanceLineId) | undefined;
          member_ids: Array<Schemas.MemberId>;
          organization_id: Schemas.OrganizationId;
          parent_task_id?: (null | Schemas.TaskId) | undefined;
          project_id?: (null | Schemas.ProjectId) | undefined;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          recurrence_id?: (null | Schemas.TaskRecurrenceId) | undefined;
          starts_at?: (string | null) | undefined;
          status: Schemas.TaskStatus;
          title: string;
          updated_at: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateTask = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/tasks";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateTaskRequest;
    };
    responses: {
      201: {
        data: {
          all_day: boolean;
          assignments: Array<Schemas.TaskAssignmentSummary>;
          blocks_availability: boolean;
          child_count?: (number | null) | undefined;
          created_at: string;
          customer_context_id?: (null | Schemas.CustomerContextId) | undefined;
          customer_id?: (null | Schemas.CustomerId) | undefined;
          description?: (string | null) | undefined;
          ends_at?: (string | null) | undefined;
          equipment: Array<Schemas.TaskEquipmentResponse>;
          expenses_cents: number;
          expenses_label?: (string | null) | undefined;
          id: Schemas.TaskId;
          labels: Array<Schemas.TaskLabelResponse>;
          maintenance_line_id?: (null | Schemas.MaintenanceLineId) | undefined;
          member_ids: Array<Schemas.MemberId>;
          organization_id: Schemas.OrganizationId;
          parent_task_id?: (null | Schemas.TaskId) | undefined;
          project_id?: (null | Schemas.ProjectId) | undefined;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          recurrence_id?: (null | Schemas.TaskRecurrenceId) | undefined;
          starts_at?: (string | null) | undefined;
          status: Schemas.TaskStatus;
          title: string;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      409: unknown;
    };
  };
  export type post_BulkAssignTasks = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/tasks/bulk-assign";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.BulkAssignTasksRequest;
    };
    responses: {
      200: {
        data: { tasks: Array<Schemas.TaskResponse> };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type get_GetTask = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/tasks/{task_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; task_id: string };
    };
    responses: {
      200: {
        data: {
          all_day: boolean;
          assignments: Array<Schemas.TaskAssignmentSummary>;
          blocks_availability: boolean;
          child_count?: (number | null) | undefined;
          created_at: string;
          customer_context_id?: (null | Schemas.CustomerContextId) | undefined;
          customer_id?: (null | Schemas.CustomerId) | undefined;
          description?: (string | null) | undefined;
          ends_at?: (string | null) | undefined;
          equipment: Array<Schemas.TaskEquipmentResponse>;
          expenses_cents: number;
          expenses_label?: (string | null) | undefined;
          id: Schemas.TaskId;
          labels: Array<Schemas.TaskLabelResponse>;
          maintenance_line_id?: (null | Schemas.MaintenanceLineId) | undefined;
          member_ids: Array<Schemas.MemberId>;
          organization_id: Schemas.OrganizationId;
          parent_task_id?: (null | Schemas.TaskId) | undefined;
          project_id?: (null | Schemas.ProjectId) | undefined;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          recurrence_id?: (null | Schemas.TaskRecurrenceId) | undefined;
          starts_at?: (string | null) | undefined;
          status: Schemas.TaskStatus;
          title: string;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type delete_DeleteTask = {
    method: "DELETE";
    path: "/api/v1/organizations/{organization_id}/tasks/{task_id}";
    requestFormat: "json";
    parameters: {
      query: Partial<{ scope: "THIS_OCCURRENCE" | "THIS_AND_FOLLOWING" }>;
      path: { organization_id: string; task_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type patch_PatchTask = {
    method: "PATCH";
    path: "/api/v1/organizations/{organization_id}/tasks/{task_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; task_id: string };

      body: Schemas.UpdateTaskRequest;
    };
    responses: {
      200: {
        data: { detached: boolean; task: Schemas.TaskResponse };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_ListTaskComments = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/tasks/{task_id}/comments";
    requestFormat: "json";
    parameters: {
      query: Partial<{ page: number; per_page: number }>;
      path: { organization_id: string; task_id: string };
    };
    responses: {
      200: {
        data: Array<{
          author: Schemas.TaskCommentAuthorResponse;
          author_is_self: boolean;
          body: string;
          created_at: string;
          id: Schemas.TaskCommentId;
          organization_id: Schemas.OrganizationId;
          task_id: Schemas.TaskId;
          updated_at: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type post_CreateTaskComment = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/tasks/{task_id}/comments";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; task_id: string };

      body: Schemas.CreateTaskCommentRequest;
    };
    responses: {
      201: {
        data: {
          author: Schemas.TaskCommentAuthorResponse;
          author_is_self: boolean;
          body: string;
          created_at: string;
          id: Schemas.TaskCommentId;
          organization_id: Schemas.OrganizationId;
          task_id: Schemas.TaskId;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type delete_DeleteTaskComment = {
    method: "DELETE";
    path: "/api/v1/organizations/{organization_id}/tasks/{task_id}/comments/{comment_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; task_id: string; comment_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type patch_UpdateTaskComment = {
    method: "PATCH";
    path: "/api/v1/organizations/{organization_id}/tasks/{task_id}/comments/{comment_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; task_id: string; comment_id: string };

      body: Schemas.UpdateTaskCommentRequest;
    };
    responses: {
      200: {
        data: {
          author: Schemas.TaskCommentAuthorResponse;
          author_is_self: boolean;
          body: string;
          created_at: string;
          id: Schemas.TaskCommentId;
          organization_id: Schemas.OrganizationId;
          task_id: Schemas.TaskId;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type get_GetProduct = {
    method: "GET";
    path: "/api/v1/products/{product_id}";
    requestFormat: "json";
    parameters: {
      path: { product_id: string };
    };
    responses: {
      200: {
        data: {
          archived_at?: (string | null) | undefined;
          billing_frequency: Schemas.BillingFrequency;
          category: string;
          cost_cents?: (number | null) | undefined;
          created_at: string;
          datasheet_keys: Array<string>;
          default_vat_rate_bp?: (number | null) | undefined;
          description?: (string | null) | undefined;
          id: Schemas.ProductId;
          invoice_usage_count: number;
          manufacturer?: (string | null) | undefined;
          name: string;
          organization_id: Schemas.OrganizationId;
          photo_keys: Array<string>;
          quote_usage_count: number;
          sku?: (string | null) | undefined;
          supplier_id?: (null | Schemas.SupplierId) | undefined;
          technical_specs?: (string | null) | undefined;
          unit: string;
          unit_price_cents: number;
          updated_at: string;
          usage_count: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type delete_DeleteProduct = {
    method: "DELETE";
    path: "/api/v1/products/{product_id}";
    requestFormat: "json";
    parameters: {
      path: { product_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type patch_UpdateProduct = {
    method: "PATCH";
    path: "/api/v1/products/{product_id}";
    requestFormat: "json";
    parameters: {
      path: { product_id: string };

      body: Schemas.UpdateProductRequest;
    };
    responses: {
      200: {
        data: {
          archived_at?: (string | null) | undefined;
          billing_frequency: Schemas.BillingFrequency;
          category: string;
          cost_cents?: (number | null) | undefined;
          created_at: string;
          datasheet_keys: Array<string>;
          default_vat_rate_bp?: (number | null) | undefined;
          description?: (string | null) | undefined;
          id: Schemas.ProductId;
          invoice_usage_count: number;
          manufacturer?: (string | null) | undefined;
          name: string;
          organization_id: Schemas.OrganizationId;
          photo_keys: Array<string>;
          quote_usage_count: number;
          sku?: (string | null) | undefined;
          supplier_id?: (null | Schemas.SupplierId) | undefined;
          technical_specs?: (string | null) | undefined;
          unit: string;
          unit_price_cents: number;
          updated_at: string;
          usage_count: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_GetProjectBillingSummary = {
    method: "GET";
    path: "/api/v1/projects/{project_id}/billing-summary";
    requestFormat: "json";
    parameters: {
      path: { project_id: string };
    };
    responses: {
      200: {
        data: {
          billed_cents: number;
          project_id: Schemas.ProjectId;
          quoted_cents?: (number | null) | undefined;
          remaining_cents?: (number | null) | undefined;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type get_ListProjectInvoices = {
    method: "GET";
    path: "/api/v1/projects/{project_id}/invoices";
    requestFormat: "json";
    parameters: {
      path: { project_id: string };
    };
    responses: {
      200: {
        data: Array<{
          balance_cents: number;
          created_at: string;
          credited_cents: number;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          delivery_address?: (null | Schemas.DeliveryAddressResponse) | undefined;
          display_state: Schemas.InvoiceDisplayState;
          due_at?: (string | null) | undefined;
          due_on?: (string | null) | undefined;
          events: Array<Schemas.InvoiceEventResponse>;
          full_cancellation: boolean;
          gross_cents: number;
          id: Schemas.InvoiceId;
          internal_note?: (string | null) | undefined;
          issued_at?: (string | null) | undefined;
          issued_on?: (string | null) | undefined;
          kind: Schemas.InvoiceKind;
          legal_notice: string;
          lines: Array<Schemas.InvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number?: (string | null) | undefined;
          operation_nature?: (null | Schemas.OperationNature) | undefined;
          organization_id: Schemas.OrganizationId;
          paid_cents: number;
          payment_terms_days: number;
          project_id?: (null | Schemas.ProjectId) | undefined;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          reason?: (string | null) | undefined;
          sent_at?: (string | null) | undefined;
          source_invoice_id?: (null | Schemas.InvoiceId) | undefined;
          status: Schemas.InvoiceStatus;
          title: string;
          updated_at: string;
          vat_breakdown: Array<Schemas.InvoiceVatBreakdownLineResponse>;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type post_IssueProjectDeposit = {
    method: "POST";
    path: "/api/v1/projects/{project_id}/invoices/deposit";
    requestFormat: "json";
    parameters: {
      path: { project_id: string };

      body: Schemas.IssueDepositRequest;
    };
    responses: {
      201: {
        data: {
          balance_cents: number;
          created_at: string;
          credited_cents: number;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          delivery_address?: (null | Schemas.DeliveryAddressResponse) | undefined;
          display_state: Schemas.InvoiceDisplayState;
          due_at?: (string | null) | undefined;
          due_on?: (string | null) | undefined;
          events: Array<Schemas.InvoiceEventResponse>;
          full_cancellation: boolean;
          gross_cents: number;
          id: Schemas.InvoiceId;
          internal_note?: (string | null) | undefined;
          issued_at?: (string | null) | undefined;
          issued_on?: (string | null) | undefined;
          kind: Schemas.InvoiceKind;
          legal_notice: string;
          lines: Array<Schemas.InvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number?: (string | null) | undefined;
          operation_nature?: (null | Schemas.OperationNature) | undefined;
          organization_id: Schemas.OrganizationId;
          paid_cents: number;
          payment_terms_days: number;
          project_id?: (null | Schemas.ProjectId) | undefined;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          reason?: (string | null) | undefined;
          sent_at?: (string | null) | undefined;
          source_invoice_id?: (null | Schemas.InvoiceId) | undefined;
          status: Schemas.InvoiceStatus;
          title: string;
          updated_at: string;
          vat_breakdown: Array<Schemas.InvoiceVatBreakdownLineResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type post_IssueProjectFinalInvoice = {
    method: "POST";
    path: "/api/v1/projects/{project_id}/invoices/final";
    requestFormat: "json";
    parameters: {
      path: { project_id: string };

      body: Schemas.IssueFinalInvoiceRequest;
    };
    responses: {
      201: {
        data: {
          balance_cents: number;
          created_at: string;
          credited_cents: number;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          delivery_address?: (null | Schemas.DeliveryAddressResponse) | undefined;
          display_state: Schemas.InvoiceDisplayState;
          due_at?: (string | null) | undefined;
          due_on?: (string | null) | undefined;
          events: Array<Schemas.InvoiceEventResponse>;
          full_cancellation: boolean;
          gross_cents: number;
          id: Schemas.InvoiceId;
          internal_note?: (string | null) | undefined;
          issued_at?: (string | null) | undefined;
          issued_on?: (string | null) | undefined;
          kind: Schemas.InvoiceKind;
          legal_notice: string;
          lines: Array<Schemas.InvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number?: (string | null) | undefined;
          operation_nature?: (null | Schemas.OperationNature) | undefined;
          organization_id: Schemas.OrganizationId;
          paid_cents: number;
          payment_terms_days: number;
          project_id?: (null | Schemas.ProjectId) | undefined;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          reason?: (string | null) | undefined;
          sent_at?: (string | null) | undefined;
          source_invoice_id?: (null | Schemas.InvoiceId) | undefined;
          status: Schemas.InvoiceStatus;
          title: string;
          updated_at: string;
          vat_breakdown: Array<Schemas.InvoiceVatBreakdownLineResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_GetProjectSupplierCosts = {
    method: "GET";
    path: "/api/v1/projects/{project_id}/supplier-costs";
    requestFormat: "json";
    parameters: {
      path: { project_id: string };
    };
    responses: {
      200: {
        data: {
          allocated_cents: number;
          lines: Array<Schemas.ProjectSupplierCostLineResponse>;
          project_id: Schemas.ProjectId;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type get_GetQuote = {
    method: "GET";
    path: "/api/v1/quotes/{quote_id}";
    requestFormat: "json";
    parameters: {
      path: { quote_id: string };
    };
    responses: {
      200: {
        data: {
          billing_address_id?: (null | Schemas.CustomerContextId) | undefined;
          billing_contact_id?: (null | Schemas.CustomerContactId) | undefined;
          created_at: string;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          deal_id?: (null | Schemas.DealId) | undefined;
          decline_reason?: (string | null) | undefined;
          deposit_percent: string;
          gross_cents: number;
          id: Schemas.QuoteId;
          is_expired: boolean;
          issuer_snapshot?: (null | Schemas.IssuerSnapshot) | undefined;
          lines: Array<Schemas.QuoteLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          owner_member_id?: (null | Schemas.MemberId) | undefined;
          recipient_snapshot?: (null | Schemas.RecipientSnapshot) | undefined;
          recurring: Array<Schemas.RecurringTotalResponse>;
          reference?: (string | null) | undefined;
          responded_at?: (string | null) | undefined;
          responded_by?: (string | null) | undefined;
          revision_of_id?: (null | Schemas.QuoteId) | undefined;
          status: Schemas.QuoteStatus;
          superseded_by_id?: (null | Schemas.QuoteId) | undefined;
          title: string;
          updated_at: string;
          valid_until?: (string | null) | undefined;
          vat_breakdown: Array<Schemas.QuoteVatBreakdownLineResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type delete_DeleteQuote = {
    method: "DELETE";
    path: "/api/v1/quotes/{quote_id}";
    requestFormat: "json";
    parameters: {
      path: { quote_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown; 409: unknown };
  };
  export type patch_UpdateQuote = {
    method: "PATCH";
    path: "/api/v1/quotes/{quote_id}";
    requestFormat: "json";
    parameters: {
      path: { quote_id: string };

      body: Schemas.UpdateQuoteRequest;
    };
    responses: {
      200: {
        data: {
          billing_address_id?: (null | Schemas.CustomerContextId) | undefined;
          billing_contact_id?: (null | Schemas.CustomerContactId) | undefined;
          created_at: string;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          deal_id?: (null | Schemas.DealId) | undefined;
          decline_reason?: (string | null) | undefined;
          deposit_percent: string;
          gross_cents: number;
          id: Schemas.QuoteId;
          is_expired: boolean;
          issuer_snapshot?: (null | Schemas.IssuerSnapshot) | undefined;
          lines: Array<Schemas.QuoteLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          owner_member_id?: (null | Schemas.MemberId) | undefined;
          recipient_snapshot?: (null | Schemas.RecipientSnapshot) | undefined;
          recurring: Array<Schemas.RecurringTotalResponse>;
          reference?: (string | null) | undefined;
          responded_at?: (string | null) | undefined;
          responded_by?: (string | null) | undefined;
          revision_of_id?: (null | Schemas.QuoteId) | undefined;
          status: Schemas.QuoteStatus;
          superseded_by_id?: (null | Schemas.QuoteId) | undefined;
          title: string;
          updated_at: string;
          valid_until?: (string | null) | undefined;
          vat_breakdown: Array<Schemas.QuoteVatBreakdownLineResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
      422: unknown;
    };
  };
  export type post_CreateInvoiceFromQuote = {
    method: "POST";
    path: "/api/v1/quotes/{quote_id}/invoices";
    requestFormat: "json";
    parameters: {
      path: { quote_id: string };

      body: Schemas.CreateInvoiceFromQuoteRequest;
    };
    responses: {
      201: {
        data: {
          balance_cents: number;
          created_at: string;
          credited_cents: number;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          delivery_address?: (null | Schemas.DeliveryAddressResponse) | undefined;
          display_state: Schemas.InvoiceDisplayState;
          due_at?: (string | null) | undefined;
          due_on?: (string | null) | undefined;
          events: Array<Schemas.InvoiceEventResponse>;
          full_cancellation: boolean;
          gross_cents: number;
          id: Schemas.InvoiceId;
          internal_note?: (string | null) | undefined;
          issued_at?: (string | null) | undefined;
          issued_on?: (string | null) | undefined;
          kind: Schemas.InvoiceKind;
          legal_notice: string;
          lines: Array<Schemas.InvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number?: (string | null) | undefined;
          operation_nature?: (null | Schemas.OperationNature) | undefined;
          organization_id: Schemas.OrganizationId;
          paid_cents: number;
          payment_terms_days: number;
          project_id?: (null | Schemas.ProjectId) | undefined;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          reason?: (string | null) | undefined;
          sent_at?: (string | null) | undefined;
          source_invoice_id?: (null | Schemas.InvoiceId) | undefined;
          status: Schemas.InvoiceStatus;
          title: string;
          updated_at: string;
          vat_breakdown: Array<Schemas.InvoiceVatBreakdownLineResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
      422: unknown;
    };
  };
  export type get_GetQuoteInvoicing = {
    method: "GET";
    path: "/api/v1/quotes/{quote_id}/invoicing";
    requestFormat: "json";
    parameters: {
      path: { quote_id: string };
    };
    responses: {
      200: {
        data: {
          blocker?: (null | Schemas.QuoteInvoicingBlockerResponse) | undefined;
          can_deposit: boolean;
          can_full: boolean;
          invoiced_percent: string;
          invoices: Array<Schemas.InvoiceResponse>;
          remaining_percent: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type get_ExportQuotePdf = {
    method: "GET";
    path: "/api/v1/quotes/{quote_id}/pdf";
    requestFormat: "json";
    parameters: {
      path: { quote_id: string };
    };
    responses: { 200: unknown; 401: unknown; 403: unknown; 404: unknown; 409: unknown };
  };
  export type post_CreateQuotePlan = {
    method: "POST";
    path: "/api/v1/quotes/{quote_id}/plan";
    requestFormat: "json";
    parameters: {
      path: { quote_id: string };

      body: Schemas.CreateQuotePlanRequest;
    };
    responses: {
      201: {
        data: { project: Schemas.PlannedProjectResponse; tasks: Array<Schemas.PlannedTaskResponse> };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_GetQuotePlanProposal = {
    method: "GET";
    path: "/api/v1/quotes/{quote_id}/plan-proposal";
    requestFormat: "json";
    parameters: {
      path: { quote_id: string };
    };
    responses: {
      200: {
        data: { quote: Schemas.QuoteResponse; tasks: Array<Schemas.TaskProposalResponse> };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type post_ReviseQuote = {
    method: "POST";
    path: "/api/v1/quotes/{quote_id}/revision";
    requestFormat: "json";
    parameters: {
      path: { quote_id: string };
    };
    responses: {
      201: {
        data: {
          billing_address_id?: (null | Schemas.CustomerContextId) | undefined;
          billing_contact_id?: (null | Schemas.CustomerContactId) | undefined;
          created_at: string;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          deal_id?: (null | Schemas.DealId) | undefined;
          decline_reason?: (string | null) | undefined;
          deposit_percent: string;
          gross_cents: number;
          id: Schemas.QuoteId;
          is_expired: boolean;
          issuer_snapshot?: (null | Schemas.IssuerSnapshot) | undefined;
          lines: Array<Schemas.QuoteLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          owner_member_id?: (null | Schemas.MemberId) | undefined;
          recipient_snapshot?: (null | Schemas.RecipientSnapshot) | undefined;
          recurring: Array<Schemas.RecurringTotalResponse>;
          reference?: (string | null) | undefined;
          responded_at?: (string | null) | undefined;
          responded_by?: (string | null) | undefined;
          revision_of_id?: (null | Schemas.QuoteId) | undefined;
          status: Schemas.QuoteStatus;
          superseded_by_id?: (null | Schemas.QuoteId) | undefined;
          title: string;
          updated_at: string;
          valid_until?: (string | null) | undefined;
          vat_breakdown: Array<Schemas.QuoteVatBreakdownLineResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type patch_UpdateQuoteStatus = {
    method: "PATCH";
    path: "/api/v1/quotes/{quote_id}/status";
    requestFormat: "json";
    parameters: {
      path: { quote_id: string };

      body: Schemas.UpdateQuoteStatusRequest;
    };
    responses: {
      200: {
        data: {
          billing_address_id?: (null | Schemas.CustomerContextId) | undefined;
          billing_contact_id?: (null | Schemas.CustomerContactId) | undefined;
          created_at: string;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          deal_id?: (null | Schemas.DealId) | undefined;
          decline_reason?: (string | null) | undefined;
          deposit_percent: string;
          gross_cents: number;
          id: Schemas.QuoteId;
          is_expired: boolean;
          issuer_snapshot?: (null | Schemas.IssuerSnapshot) | undefined;
          lines: Array<Schemas.QuoteLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          owner_member_id?: (null | Schemas.MemberId) | undefined;
          recipient_snapshot?: (null | Schemas.RecipientSnapshot) | undefined;
          recurring: Array<Schemas.RecurringTotalResponse>;
          reference?: (string | null) | undefined;
          responded_at?: (string | null) | undefined;
          responded_by?: (string | null) | undefined;
          revision_of_id?: (null | Schemas.QuoteId) | undefined;
          status: Schemas.QuoteStatus;
          superseded_by_id?: (null | Schemas.QuoteId) | undefined;
          title: string;
          updated_at: string;
          valid_until?: (string | null) | undefined;
          vat_breakdown: Array<Schemas.QuoteVatBreakdownLineResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
      422: unknown;
    };
  };
  export type post_AllocateReceipt = {
    method: "POST";
    path: "/api/v1/receipts/{receipt_id}/allocate";
    requestFormat: "json";
    parameters: {
      path: { receipt_id: string };

      body: Schemas.AllocateReceiptRequest;
    };
    responses: {
      200: {
        data: {
          amount_cents: number;
          created_at: string;
          deleted_at?: (string | null) | undefined;
          deleted_by?: (null | Schemas.UserId) | undefined;
          id: Schemas.InvoicePaymentId;
          invoice_id: Schemas.InvoiceId;
          method: Schemas.PaymentMethod;
          note?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          paid_on: string;
          recorded_by: Schemas.UserId;
          reference?: (string | null) | undefined;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
      422: unknown;
    };
  };
  export type delete_DeleteRole = {
    method: "DELETE";
    path: "/api/v1/roles/{role_id}";
    requestFormat: "json";
    parameters: {
      path: { role_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown; 409: unknown };
  };
  export type patch_UpdateRole = {
    method: "PATCH";
    path: "/api/v1/roles/{role_id}";
    requestFormat: "json";
    parameters: {
      path: { role_id: string };

      body: Schemas.UpdateRoleRequest;
    };
    responses: {
      200: {
        data: {
          created_at: string;
          id: Schemas.RoleId;
          is_seeded: boolean;
          name: string;
          organization_id: Schemas.OrganizationId;
          permissions: Array<string>;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_GetServiceRate = {
    method: "GET";
    path: "/api/v1/service-rates/{service_rate_id}";
    requestFormat: "json";
    parameters: {
      path: { service_rate_id: string };
    };
    responses: {
      200: {
        data: {
          archived_at?: (string | null) | undefined;
          billing_frequency: Schemas.BillingFrequency;
          category: string;
          cost_cents?: (number | null) | undefined;
          created_at: string;
          default_vat_rate_bp?: (number | null) | undefined;
          description?: (string | null) | undefined;
          id: Schemas.ServiceRateId;
          invoice_usage_count: number;
          label: string;
          organization_id: Schemas.OrganizationId;
          quote_usage_count: number;
          rate_cents: number;
          unit: string;
          updated_at: string;
          usage_count: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type delete_DeleteServiceRate = {
    method: "DELETE";
    path: "/api/v1/service-rates/{service_rate_id}";
    requestFormat: "json";
    parameters: {
      path: { service_rate_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type patch_UpdateServiceRate = {
    method: "PATCH";
    path: "/api/v1/service-rates/{service_rate_id}";
    requestFormat: "json";
    parameters: {
      path: { service_rate_id: string };

      body: Schemas.UpdateServiceRateRequest;
    };
    responses: {
      200: {
        data: {
          archived_at?: (string | null) | undefined;
          billing_frequency: Schemas.BillingFrequency;
          category: string;
          cost_cents?: (number | null) | undefined;
          created_at: string;
          default_vat_rate_bp?: (number | null) | undefined;
          description?: (string | null) | undefined;
          id: Schemas.ServiceRateId;
          invoice_usage_count: number;
          label: string;
          organization_id: Schemas.OrganizationId;
          quote_usage_count: number;
          rate_cents: number;
          unit: string;
          updated_at: string;
          usage_count: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type put_ReplaceSupplierInvoiceLineAllocations = {
    method: "PUT";
    path: "/api/v1/supplier-invoice-lines/{supplier_invoice_line_id}/allocations";
    requestFormat: "json";
    parameters: {
      path: { supplier_invoice_line_id: string };

      body: Schemas.ReplaceLineAllocationsRequest;
    };
    responses: {
      200: {
        data: Array<{
          amount_cents: number;
          created_at: string;
          id: Schemas.SupplierInvoiceLineAllocationId;
          organization_id: Schemas.OrganizationId;
          project_id: Schemas.ProjectId;
          supplier_invoice_line_id: Schemas.SupplierInvoiceLineId;
          updated_at: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_GetSupplierInvoice = {
    method: "GET";
    path: "/api/v1/supplier-invoices/{supplier_invoice_id}";
    requestFormat: "json";
    parameters: {
      path: { supplier_invoice_id: string };
    };
    responses: {
      200: {
        data: {
          created_at: string;
          currency: string;
          due_on?: (string | null) | undefined;
          gross_cents: number;
          id: Schemas.SupplierInvoiceId;
          issued_on: string;
          lines: Array<Schemas.SupplierInvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number: string;
          organization_id: Schemas.OrganizationId;
          received_at: string;
          source: Schemas.SupplierInvoiceSource;
          source_file_key?: (string | null) | undefined;
          source_file_mime_type?: (string | null) | undefined;
          status: Schemas.SupplierInvoiceStatus;
          supplier_id?: (null | Schemas.SupplierId) | undefined;
          supplier_name: string;
          supplier_registration_number?: (string | null) | undefined;
          supplier_vat_number?: (string | null) | undefined;
          updated_at: string;
          vat_breakdown: Array<Schemas.SupplierInvoiceVatBreakdownLineResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type patch_UpdateSupplierInvoiceNotes = {
    method: "PATCH";
    path: "/api/v1/supplier-invoices/{supplier_invoice_id}";
    requestFormat: "json";
    parameters: {
      path: { supplier_invoice_id: string };

      body: Schemas.UpdateSupplierInvoiceRequest;
    };
    responses: {
      200: {
        data: {
          created_at: string;
          currency: string;
          due_on?: (string | null) | undefined;
          gross_cents: number;
          id: Schemas.SupplierInvoiceId;
          issued_on: string;
          lines: Array<Schemas.SupplierInvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number: string;
          organization_id: Schemas.OrganizationId;
          received_at: string;
          source: Schemas.SupplierInvoiceSource;
          source_file_key?: (string | null) | undefined;
          source_file_mime_type?: (string | null) | undefined;
          status: Schemas.SupplierInvoiceStatus;
          supplier_id?: (null | Schemas.SupplierId) | undefined;
          supplier_name: string;
          supplier_registration_number?: (string | null) | undefined;
          supplier_vat_number?: (string | null) | undefined;
          updated_at: string;
          vat_breakdown: Array<Schemas.SupplierInvoiceVatBreakdownLineResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type post_ConfirmSupplierInvoice = {
    method: "POST";
    path: "/api/v1/supplier-invoices/{supplier_invoice_id}/confirm";
    requestFormat: "json";
    parameters: {
      path: { supplier_invoice_id: string };

      body: Schemas.ConfirmSupplierInvoiceRequest;
    };
    responses: {
      200: {
        data: {
          created_at: string;
          currency: string;
          due_on?: (string | null) | undefined;
          gross_cents: number;
          id: Schemas.SupplierInvoiceId;
          issued_on: string;
          lines: Array<Schemas.SupplierInvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number: string;
          organization_id: Schemas.OrganizationId;
          received_at: string;
          source: Schemas.SupplierInvoiceSource;
          source_file_key?: (string | null) | undefined;
          source_file_mime_type?: (string | null) | undefined;
          status: Schemas.SupplierInvoiceStatus;
          supplier_id?: (null | Schemas.SupplierId) | undefined;
          supplier_name: string;
          supplier_registration_number?: (string | null) | undefined;
          supplier_vat_number?: (string | null) | undefined;
          updated_at: string;
          vat_breakdown: Array<Schemas.SupplierInvoiceVatBreakdownLineResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type post_RejectSupplierInvoice = {
    method: "POST";
    path: "/api/v1/supplier-invoices/{supplier_invoice_id}/reject";
    requestFormat: "json";
    parameters: {
      path: { supplier_invoice_id: string };

      body: Schemas.RejectSupplierInvoiceRequest;
    };
    responses: {
      200: {
        data: {
          created_at: string;
          currency: string;
          due_on?: (string | null) | undefined;
          gross_cents: number;
          id: Schemas.SupplierInvoiceId;
          issued_on: string;
          lines: Array<Schemas.SupplierInvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number: string;
          organization_id: Schemas.OrganizationId;
          received_at: string;
          source: Schemas.SupplierInvoiceSource;
          source_file_key?: (string | null) | undefined;
          source_file_mime_type?: (string | null) | undefined;
          status: Schemas.SupplierInvoiceStatus;
          supplier_id?: (null | Schemas.SupplierId) | undefined;
          supplier_name: string;
          supplier_registration_number?: (string | null) | undefined;
          supplier_vat_number?: (string | null) | undefined;
          updated_at: string;
          vat_breakdown: Array<Schemas.SupplierInvoiceVatBreakdownLineResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_ListIntakeTickets = {
    method: "GET";
    path: "/api/v1/support/intake/tickets";
    requestFormat: "json";
    parameters: {
      query: Partial<{ custom_field_key: string; custom_field_value: string; page: number; per_page: number }>;
    };
    responses: {
      200: {
        data: Array<{
          badges: Array<Schemas.BadgeResponse>;
          body: string;
          created_at: string;
          custom_fields: Record<string, unknown>;
          entries?: (Array<Schemas.IntakeEntryResponse> | null) | undefined;
          external_id?: (string | null) | undefined;
          id: Schemas.TicketId;
          number: Schemas.TicketNumber;
          status: Schemas.TicketStatus;
          subject: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      422: unknown;
      429: unknown;
    };
  };
  export type post_CreateIntakeTicket = {
    method: "POST";
    path: "/api/v1/support/intake/tickets";
    requestFormat: "json";
    parameters: {
      body: Schemas.CreateIntakeTicketRequest;
    };
    responses: {
      200: {
        data: {
          badges: Array<Schemas.BadgeResponse>;
          body: string;
          created_at: string;
          custom_fields: Record<string, unknown>;
          entries?: (Array<Schemas.IntakeEntryResponse> | null) | undefined;
          external_id?: (string | null) | undefined;
          id: Schemas.TicketId;
          number: Schemas.TicketNumber;
          status: Schemas.TicketStatus;
          subject: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      201: {
        data: {
          badges: Array<Schemas.BadgeResponse>;
          body: string;
          created_at: string;
          custom_fields: Record<string, unknown>;
          entries?: (Array<Schemas.IntakeEntryResponse> | null) | undefined;
          external_id?: (string | null) | undefined;
          id: Schemas.TicketId;
          number: Schemas.TicketNumber;
          status: Schemas.TicketStatus;
          subject: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      422: unknown;
      429: unknown;
    };
  };
  export type get_GetIntakeTicket = {
    method: "GET";
    path: "/api/v1/support/intake/tickets/{ticket_id}";
    requestFormat: "json";
    parameters: {
      path: { ticket_id: string };
    };
    responses: {
      200: {
        data: {
          badges: Array<Schemas.BadgeResponse>;
          body: string;
          created_at: string;
          custom_fields: Record<string, unknown>;
          entries?: (Array<Schemas.IntakeEntryResponse> | null) | undefined;
          external_id?: (string | null) | undefined;
          id: Schemas.TicketId;
          number: Schemas.TicketNumber;
          status: Schemas.TicketStatus;
          subject: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      404: unknown;
      429: unknown;
    };
  };
  export type post_AttachIntakeFile = {
    method: "POST";
    path: "/api/v1/support/intake/tickets/{ticket_id}/attachments";
    requestFormat: "binary";
    parameters: {
      query: { entry_id: string };
      path: { ticket_id: string };
      header: { "X-File-Name": string };
      body: Array<number>;
    };
    responses: {
      201: {
        data: {
          created_at: string;
          file_name: string;
          id: Schemas.AttachmentId;
          mime_type: string;
          size_bytes: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      404: unknown;
      413: unknown;
      422: unknown;
      429: unknown;
    };
  };
  export type get_GetIntakeAttachmentUrl = {
    method: "GET";
    path: "/api/v1/support/intake/tickets/{ticket_id}/attachments/{attachment_id}/url";
    requestFormat: "json";
    parameters: {
      path: { ticket_id: string; attachment_id: string };
    };
    responses: {
      200: { data: { expires_at: string; url: string }; pagination?: (null | Schemas.PaginationMetadata) | undefined };
      401: unknown;
      404: unknown;
      429: unknown;
    };
  };
  export type post_PostIntakeMessage = {
    method: "POST";
    path: "/api/v1/support/intake/tickets/{ticket_id}/messages";
    requestFormat: "json";
    parameters: {
      path: { ticket_id: string };

      body: Schemas.PostIntakeMessageRequest;
    };
    responses: {
      201: {
        data: {
          attachments: Array<Schemas.IntakeAttachmentResponse>;
          body: string;
          created_at: string;
          id: Schemas.TicketEntryId;
          kind: Schemas.IntakeEntryKind;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      404: unknown;
      422: unknown;
      429: unknown;
    };
  };
  export type delete_DeleteTaskRecurrence = {
    method: "DELETE";
    path: "/api/v1/task-recurrences/{task_recurrence_id}";
    requestFormat: "json";
    parameters: {
      path: { task_recurrence_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type patch_PatchTaskRecurrence = {
    method: "PATCH";
    path: "/api/v1/task-recurrences/{task_recurrence_id}";
    requestFormat: "json";
    parameters: {
      path: { task_recurrence_id: string };

      body: Schemas.UpdateTaskRecurrenceRequest;
    };
    responses: {
      200: {
        data: Schemas.RecurrenceRuleResponse & {
          all_day: boolean;
          assignee_member_ids: Array<Schemas.MemberId>;
          blocks_availability: boolean;
          created_at: string;
          customer_context_id?: (null | Schemas.CustomerContextId) | undefined;
          customer_id?: (null | Schemas.CustomerId) | undefined;
          description?: (string | null) | undefined;
          duration_minutes: number;
          ends_on?: (string | null) | undefined;
          horizon_filled_to: string;
          id: Schemas.TaskRecurrenceId;
          organization_id: Schemas.OrganizationId;
          project_id?: (null | Schemas.ProjectId) | undefined;
          start_time: string;
          starts_on: string;
          timezone: string;
          title: string;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
    };
  };
  export type get_ListMyOrganizations = {
    method: "GET";
    path: "/api/v1/users/@me/organizations";
    requestFormat: "json";
    parameters: never;
    responses: {
      200: {
        data: Array<{
          address_city?: (string | null) | undefined;
          address_country?: (string | null) | undefined;
          address_line1?: (string | null) | undefined;
          address_line2?: (string | null) | undefined;
          address_postal_code?: (string | null) | undefined;
          contact_email?: (string | null) | undefined;
          contact_phone?: (string | null) | undefined;
          created_at: string;
          field_clock_enabled: boolean;
          id: Schemas.OrganizationId;
          insurance_mention?: (string | null) | undefined;
          legal_form?: (string | null) | undefined;
          legal_name?: (string | null) | undefined;
          mcp_enabled: boolean;
          missing_legal_identity_fields: Array<Schemas.str>;
          name: string;
          owner_id: Schemas.UserId;
          registration_number?: (string | null) | undefined;
          share_capital_cents?: (number | null) | undefined;
          slug: string;
          updated_at: string;
          vat_on_debits: boolean;
          vat_status?: (null | Schemas.VatStatusResponse) | undefined;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
    };
  };
  export type get_CompleteAutomationCredentialOauth = {
    method: "GET";
    path: "/oauth/callback";
    requestFormat: "json";
    parameters: {
      query: Partial<{ state: string; code: string; error: string; oauth_token: string; oauth_verifier: string }>;
    };
    responses: { 303: unknown; 400: unknown; 429: unknown };
  };
  export type post_ReceiveWebhook = {
    method: "POST";
    path: "/webhooks/{path}";
    requestFormat: "json";
    parameters: {
      path: { path: string };
    };
    responses: {
      202: Schemas.WebhookAccepted;
      400: Schemas.WebhookBodyInvalid;
      401: unknown;
      404: unknown;
      408: unknown;
      413: unknown;
      415: unknown;
      429: unknown;
    };
  };

  // </Endpoints>
}

// <EndpointByMethod>
export type EndpointByMethod = {
  patch: {
    "/api/v1/assignment-reports/{assignment_report_id}/resolution": Endpoints.patch_ResolveAssignmentReport;
    "/api/v1/chat/categories/{category_id}": Endpoints.patch_UpdateCategory;
    "/api/v1/chat/channels/{channel_id}": Endpoints.patch_UpdateChannel;
    "/api/v1/chat/messages/{message_id}": Endpoints.patch_UpdateMessage;
    "/api/v1/chat/threads/{channel_id}": Endpoints.patch_UpdateThread;
    "/api/v1/chat/webhooks/{webhook_id}": Endpoints.patch_UpdateWebhook;
    "/api/v1/cost-bases/{cost_basis_id}": Endpoints.patch_CorrectEmployeeCostBasis;
    "/api/v1/customer-contacts/{customer_contact_id}": Endpoints.patch_UpdateCustomerContact;
    "/api/v1/customer-contexts/{customer_context_id}": Endpoints.patch_UpdateCustomerContext;
    "/api/v1/customers/{customer_id}": Endpoints.patch_UpdateCustomer;
    "/api/v1/deals/{deal_id}": Endpoints.patch_UpdateDeal;
    "/api/v1/equipment/{equipment_id}": Endpoints.patch_UpdateEquipment;
    "/api/v1/expense-categories/{category_id}": Endpoints.patch_UpdateExpenseCategory;
    "/api/v1/expenses/{expense_id}": Endpoints.patch_UpdateExpense;
    "/api/v1/field/assignment-reports/{assignment_report_id}": Endpoints.patch_AmendAssignmentReport;
    "/api/v1/invoices/{invoice_id}": Endpoints.patch_UpdateInvoice;
    "/api/v1/members/{member_id}": Endpoints.patch_UpdateMember;
    "/api/v1/organizations/{organization_id}": Endpoints.patch_UpdateOrganization;
    "/api/v1/organizations/{organization_id}/absences/{absence_id}": Endpoints.patch_PatchAbsence;
    "/api/v1/organizations/{organization_id}/automation/credentials/{credential_id}": Endpoints.patch_UpdateAutomationCredential;
    "/api/v1/organizations/{organization_id}/automation/folders/{folder_id}": Endpoints.patch_UpdateWorkflowFolder;
    "/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}": Endpoints.patch_UpdateWorkflow;
    "/api/v1/organizations/{organization_id}/legal-identity": Endpoints.patch_UpdateOrganizationLegalIdentity;
    "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}": Endpoints.patch_PatchProjectTemplate;
    "/api/v1/organizations/{organization_id}/projects/{project_id}": Endpoints.patch_PatchProject;
    "/api/v1/organizations/{organization_id}/support/entries/{entry_id}": Endpoints.patch_EditSupportDraft;
    "/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/status": Endpoints.patch_TransitionSupportTicket;
    "/api/v1/organizations/{organization_id}/task-labels/{label_id}": Endpoints.patch_UpdateTaskLabel;
    "/api/v1/organizations/{organization_id}/tasks/{task_id}": Endpoints.patch_PatchTask;
    "/api/v1/organizations/{organization_id}/tasks/{task_id}/comments/{comment_id}": Endpoints.patch_UpdateTaskComment;
    "/api/v1/products/{product_id}": Endpoints.patch_UpdateProduct;
    "/api/v1/quotes/{quote_id}": Endpoints.patch_UpdateQuote;
    "/api/v1/quotes/{quote_id}/status": Endpoints.patch_UpdateQuoteStatus;
    "/api/v1/roles/{role_id}": Endpoints.patch_UpdateRole;
    "/api/v1/service-rates/{service_rate_id}": Endpoints.patch_UpdateServiceRate;
    "/api/v1/supplier-invoices/{supplier_invoice_id}": Endpoints.patch_UpdateSupplierInvoiceNotes;
    "/api/v1/task-recurrences/{task_recurrence_id}": Endpoints.patch_PatchTaskRecurrence;
  };
  delete: {
    "/api/v1/chat/categories/{category_id}": Endpoints.delete_DeleteCategory;
    "/api/v1/chat/channels/{channel_id}": Endpoints.delete_DeleteChannel;
    "/api/v1/chat/channels/{channel_id}/permissions/everyone": Endpoints.delete_DeleteEveryoneOverwrite;
    "/api/v1/chat/channels/{channel_id}/permissions/{target_type}/{target_id}": Endpoints.delete_DeleteTargetOverwrite;
    "/api/v1/chat/messages/{message_id}": Endpoints.delete_DeleteMessage;
    "/api/v1/chat/messages/{message_id}/reactions/{emoji}": Endpoints.delete_RemoveReaction;
    "/api/v1/chat/threads/{channel_id}": Endpoints.delete_DeleteThread;
    "/api/v1/chat/webhooks/{webhook_id}": Endpoints.delete_DeleteWebhook;
    "/api/v1/customer-contacts/{customer_contact_id}": Endpoints.delete_DeleteCustomerContact;
    "/api/v1/customer-contexts/{customer_context_id}": Endpoints.delete_DeleteCustomerContext;
    "/api/v1/customers/{customer_id}": Endpoints.delete_DeleteCustomer;
    "/api/v1/deals/{deal_id}": Endpoints.delete_DeleteDeal;
    "/api/v1/equipment/{equipment_id}": Endpoints.delete_DeleteEquipment;
    "/api/v1/expense-categories/{category_id}": Endpoints.delete_DeleteExpenseCategory;
    "/api/v1/expenses/{expense_id}": Endpoints.delete_DeleteExpense;
    "/api/v1/field/assignment-reports/{assignment_report_id}": Endpoints.delete_WithdrawAssignmentReport;
    "/api/v1/invitations/{invitation_id}": Endpoints.delete_RevokeInvitation;
    "/api/v1/invoice-payments/{invoice_payment_id}": Endpoints.delete_DeleteInvoicePayment;
    "/api/v1/invoices/{invoice_id}": Endpoints.delete_DeleteInvoice;
    "/api/v1/members/{member_id}": Endpoints.delete_DeleteMember;
    "/api/v1/members/{member_id}/employee-profile": Endpoints.delete_RemoveEmployeeProfile;
    "/api/v1/members/{member_id}/roles/{role_id}": Endpoints.delete_UnassignRole;
    "/api/v1/organizations/{organization_id}": Endpoints.delete_DeleteOrganization;
    "/api/v1/organizations/{organization_id}/absences/{absence_id}": Endpoints.delete_DeleteAbsence;
    "/api/v1/organizations/{organization_id}/automation/collections/{collection_id}": Endpoints.delete_DeleteCollection;
    "/api/v1/organizations/{organization_id}/automation/credentials/{credential_id}": Endpoints.delete_DeleteAutomationCredential;
    "/api/v1/organizations/{organization_id}/automation/folders/{folder_id}": Endpoints.delete_DeleteWorkflowFolder;
    "/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}": Endpoints.delete_DeleteWorkflow;
    "/api/v1/organizations/{organization_id}/doe-models/{doe_model_id}": Endpoints.delete_ArchiveDoeModel;
    "/api/v1/organizations/{organization_id}/does/{doe_id}": Endpoints.delete_DeleteDoe;
    "/api/v1/organizations/{organization_id}/leave-adjustments/{adjustment_id}": Endpoints.delete_DeleteLeaveAdjustment;
    "/api/v1/organizations/{organization_id}/members/{member_id}/contracts/{contract_id}": Endpoints.delete_DeleteMemberContract;
    "/api/v1/organizations/{organization_id}/members/{member_id}/day-overrides/{date}": Endpoints.delete_DeleteDayOverride;
    "/api/v1/organizations/{organization_id}/members/{member_id}/documents/{document_id}": Endpoints.delete_DeleteMemberDocument;
    "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}": Endpoints.delete_ArchiveProjectTemplate;
    "/api/v1/organizations/{organization_id}/projects/{project_id}": Endpoints.delete_ArchiveProject;
    "/api/v1/organizations/{organization_id}/support/api-keys/{key_id}": Endpoints.delete_RevokeSupportApiKey;
    "/api/v1/organizations/{organization_id}/support/entries/{entry_id}": Endpoints.delete_DeleteSupportDraft;
    "/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/badges/{label}": Endpoints.delete_RemoveSupportTicketBadge;
    "/api/v1/organizations/{organization_id}/task-labels/{label_id}": Endpoints.delete_DeleteTaskLabel;
    "/api/v1/organizations/{organization_id}/tasks/{task_id}": Endpoints.delete_DeleteTask;
    "/api/v1/organizations/{organization_id}/tasks/{task_id}/comments/{comment_id}": Endpoints.delete_DeleteTaskComment;
    "/api/v1/products/{product_id}": Endpoints.delete_DeleteProduct;
    "/api/v1/quotes/{quote_id}": Endpoints.delete_DeleteQuote;
    "/api/v1/roles/{role_id}": Endpoints.delete_DeleteRole;
    "/api/v1/service-rates/{service_rate_id}": Endpoints.delete_DeleteServiceRate;
    "/api/v1/task-recurrences/{task_recurrence_id}": Endpoints.delete_DeleteTaskRecurrence;
  };
  get: {
    "/api/v1/chat/channels/{channel_id}": Endpoints.get_GetChannel;
    "/api/v1/chat/channels/{channel_id}/messages": Endpoints.get_ListMessages;
    "/api/v1/chat/channels/{channel_id}/permissions": Endpoints.get_ListChannelPermissions;
    "/api/v1/chat/channels/{channel_id}/threads": Endpoints.get_ListThreads;
    "/api/v1/chat/channels/{channel_id}/webhooks": Endpoints.get_ListWebhooks;
    "/api/v1/chat/messages/{message_id}/reactions/{emoji}": Endpoints.get_ListReactors;
    "/api/v1/chat/organizations/{organization_id}/categories": Endpoints.get_ListCategories;
    "/api/v1/chat/organizations/{organization_id}/channels": Endpoints.get_ListChannels;
    "/api/v1/chat/organizations/{organization_id}/direct-channels": Endpoints.get_ListDirectChannels;
    "/api/v1/chat/organizations/{organization_id}/directory": Endpoints.get_ListChatDirectory;
    "/api/v1/chat/organizations/{organization_id}/notifications": Endpoints.get_ListNotifications;
    "/api/v1/chat/organizations/{organization_id}/roles": Endpoints.get_ListChatRoles;
    "/api/v1/chat/organizations/{organization_id}/unread": Endpoints.get_ListUnreadChannels;
    "/api/v1/customer-contacts/{customer_contact_id}": Endpoints.get_GetCustomerContact;
    "/api/v1/customer-contexts/{customer_context_id}": Endpoints.get_GetCustomerContext;
    "/api/v1/customers/{customer_id}": Endpoints.get_GetCustomer;
    "/api/v1/customers/{customer_id}/contacts": Endpoints.get_ListCustomerContacts;
    "/api/v1/customers/{customer_id}/customer-contexts": Endpoints.get_ListCustomerContexts;
    "/api/v1/deals/{deal_id}": Endpoints.get_GetDeal;
    "/api/v1/employees/{employee_id}/cost-bases": Endpoints.get_ListEmployeeCostBases;
    "/api/v1/equipment/{equipment_id}": Endpoints.get_GetEquipment;
    "/api/v1/expenses/{expense_id}": Endpoints.get_GetExpense;
    "/api/v1/files/url": Endpoints.get_GetFileUrl;
    "/api/v1/invoices/{invoice_id}": Endpoints.get_GetInvoice;
    "/api/v1/invoices/{invoice_id}/balance": Endpoints.get_GetInvoiceBalance;
    "/api/v1/invoices/{invoice_id}/credit-notes": Endpoints.get_ListInvoiceCreditNotes;
    "/api/v1/invoices/{invoice_id}/payments": Endpoints.get_ListInvoicePayments;
    "/api/v1/invoices/{invoice_id}/pdf": Endpoints.get_ExportInvoicePdf;
    "/api/v1/members/{member_id}": Endpoints.get_GetMember;
    "/api/v1/members/{member_id}/roles": Endpoints.get_ListMemberRoles;
    "/api/v1/members/{member_id}/work-time": Endpoints.get_GetWorkTime;
    "/api/v1/organizations": Endpoints.get_ListOrganizations;
    "/api/v1/organizations/{organization_id}": Endpoints.get_GetOrganization;
    "/api/v1/organizations/{organization_id}/absence-types": Endpoints.get_ListAbsenceTypes;
    "/api/v1/organizations/{organization_id}/absences": Endpoints.get_ListAbsences;
    "/api/v1/organizations/{organization_id}/assignment-reports": Endpoints.get_ListAssignmentReports;
    "/api/v1/organizations/{organization_id}/automation/collections": Endpoints.get_ListCollections;
    "/api/v1/organizations/{organization_id}/automation/collections/{collection_id}": Endpoints.get_GetCollection;
    "/api/v1/organizations/{organization_id}/automation/connectors": Endpoints.get_ListConnectors;
    "/api/v1/organizations/{organization_id}/automation/credential-kinds": Endpoints.get_ListCredentialKinds;
    "/api/v1/organizations/{organization_id}/automation/credentials": Endpoints.get_ListAutomationCredentials;
    "/api/v1/organizations/{organization_id}/automation/events": Endpoints.get_ListAutomationEvents;
    "/api/v1/organizations/{organization_id}/automation/folders": Endpoints.get_ListWorkflowFolders;
    "/api/v1/organizations/{organization_id}/automation/runs": Endpoints.get_ListRuns;
    "/api/v1/organizations/{organization_id}/automation/runs/{run_id}": Endpoints.get_GetRun;
    "/api/v1/organizations/{organization_id}/automation/settings": Endpoints.get_GetAutomationSettings;
    "/api/v1/organizations/{organization_id}/automation/workflows": Endpoints.get_ListWorkflows;
    "/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}": Endpoints.get_GetWorkflow;
    "/api/v1/organizations/{organization_id}/billing-settings": Endpoints.get_GetOrganizationBillingSettings;
    "/api/v1/organizations/{organization_id}/custom-units": Endpoints.get_ListCustomUnits;
    "/api/v1/organizations/{organization_id}/customers": Endpoints.get_ListCustomers;
    "/api/v1/organizations/{organization_id}/day-overrides": Endpoints.get_ListDayOverrides;
    "/api/v1/organizations/{organization_id}/deals": Endpoints.get_ListDeals;
    "/api/v1/organizations/{organization_id}/deals/summary": Endpoints.get_GetDealSummary;
    "/api/v1/organizations/{organization_id}/doe-model-starters": Endpoints.get_ListDoeModelStarters;
    "/api/v1/organizations/{organization_id}/doe-models": Endpoints.get_ListDoeModels;
    "/api/v1/organizations/{organization_id}/doe-models/{doe_model_id}": Endpoints.get_GetDoeModel;
    "/api/v1/organizations/{organization_id}/doe-settings": Endpoints.get_GetDoeSettings;
    "/api/v1/organizations/{organization_id}/does": Endpoints.get_ListDoes;
    "/api/v1/organizations/{organization_id}/does/{doe_id}": Endpoints.get_GetDoe;
    "/api/v1/organizations/{organization_id}/does/{doe_id}/pdf": Endpoints.get_ExportDoePdf;
    "/api/v1/organizations/{organization_id}/employee-profiles": Endpoints.get_ListEmployeeProfiles;
    "/api/v1/organizations/{organization_id}/equipment": Endpoints.get_ListEquipment;
    "/api/v1/organizations/{organization_id}/expense-categories": Endpoints.get_ListExpenseCategories;
    "/api/v1/organizations/{organization_id}/expenses": Endpoints.get_ListExpenses;
    "/api/v1/organizations/{organization_id}/extra-periods": Endpoints.get_ListExtraPeriods;
    "/api/v1/organizations/{organization_id}/field/assignment-reports": Endpoints.get_ListMyAssignmentReports;
    "/api/v1/organizations/{organization_id}/field/current": Endpoints.get_GetCurrentTimeEntry;
    "/api/v1/organizations/{organization_id}/field/tasks": Endpoints.get_ListMyFieldTasks;
    "/api/v1/organizations/{organization_id}/hr/planning-feed": Endpoints.get_GetHrPlanningFeed;
    "/api/v1/organizations/{organization_id}/invitations": Endpoints.get_ListInvitations;
    "/api/v1/organizations/{organization_id}/invoices": Endpoints.get_ListInvoices;
    "/api/v1/organizations/{organization_id}/invoices/outstanding": Endpoints.get_ListOutstandingBalanceByCustomer;
    "/api/v1/organizations/{organization_id}/leave-accounts": Endpoints.get_ListLeaveAccounts;
    "/api/v1/organizations/{organization_id}/leave-adjustments": Endpoints.get_ListLeaveAdjustments;
    "/api/v1/organizations/{organization_id}/ledger": Endpoints.get_ReadLedger;
    "/api/v1/organizations/{organization_id}/maintenance/contracts": Endpoints.get_ListMaintenanceContracts;
    "/api/v1/organizations/{organization_id}/maintenance/contracts/{contract_id}": Endpoints.get_GetMaintenanceContract;
    "/api/v1/organizations/{organization_id}/maintenance/grid": Endpoints.get_GetMaintenanceGrid;
    "/api/v1/organizations/{organization_id}/maintenance/interventions": Endpoints.get_ListMaintenanceInterventions;
    "/api/v1/organizations/{organization_id}/maintenance/recap": Endpoints.get_GetMaintenanceRecap;
    "/api/v1/organizations/{organization_id}/maintenance/recap.pdf": Endpoints.get_ExportMaintenanceRecapPdf;
    "/api/v1/organizations/{organization_id}/member-contracts": Endpoints.get_ListOrganizationMemberContracts;
    "/api/v1/organizations/{organization_id}/member-documents": Endpoints.get_ListMemberDocuments;
    "/api/v1/organizations/{organization_id}/member-profiles": Endpoints.get_ListMemberProfiles;
    "/api/v1/organizations/{organization_id}/members": Endpoints.get_ListMembers;
    "/api/v1/organizations/{organization_id}/members/me": Endpoints.get_GetMyMember;
    "/api/v1/organizations/{organization_id}/members/me/permissions": Endpoints.get_GetMyPermissions;
    "/api/v1/organizations/{organization_id}/members/{member_id}/contracts": Endpoints.get_ListMemberContracts;
    "/api/v1/organizations/{organization_id}/members/{member_id}/profile": Endpoints.get_GetMemberProfile;
    "/api/v1/organizations/{organization_id}/numbering": Endpoints.get_GetOrganizationNumbering;
    "/api/v1/organizations/{organization_id}/planning": Endpoints.get_GetPlanning;
    "/api/v1/organizations/{organization_id}/planning/availability": Endpoints.get_GetPlanningAvailability;
    "/api/v1/organizations/{organization_id}/products": Endpoints.get_ListProducts;
    "/api/v1/organizations/{organization_id}/project-templates": Endpoints.get_ListProjectTemplates;
    "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}": Endpoints.get_GetProjectTemplate;
    "/api/v1/organizations/{organization_id}/projects": Endpoints.get_ListProjects;
    "/api/v1/organizations/{organization_id}/projects/{project_id}": Endpoints.get_GetProject;
    "/api/v1/organizations/{organization_id}/projects/{project_id}/channel": Endpoints.get_GetProjectChannel;
    "/api/v1/organizations/{organization_id}/projects/{project_id}/doe": Endpoints.get_GetProjectDoe;
    "/api/v1/organizations/{organization_id}/quotes": Endpoints.get_ListQuotes;
    "/api/v1/organizations/{organization_id}/reporting/profitability": Endpoints.get_GetProfitability;
    "/api/v1/organizations/{organization_id}/reporting/worked-hours": Endpoints.get_GetWorkedHours;
    "/api/v1/organizations/{organization_id}/roles": Endpoints.get_ListRoles;
    "/api/v1/organizations/{organization_id}/service-rates": Endpoints.get_ListServiceRates;
    "/api/v1/organizations/{organization_id}/supplier-invoices": Endpoints.get_ListSupplierInvoices;
    "/api/v1/organizations/{organization_id}/suppliers": Endpoints.get_ListSuppliers;
    "/api/v1/organizations/{organization_id}/suppliers/{supplier_id}": Endpoints.get_GetSupplier;
    "/api/v1/organizations/{organization_id}/support/api-keys": Endpoints.get_ListSupportApiKeys;
    "/api/v1/organizations/{organization_id}/support/attachments/{attachment_id}/url": Endpoints.get_GetSupportAttachmentUrl;
    "/api/v1/organizations/{organization_id}/support/settings": Endpoints.get_GetSupportSettings;
    "/api/v1/organizations/{organization_id}/support/tickets": Endpoints.get_ListSupportTickets;
    "/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}": Endpoints.get_GetSupportTicket;
    "/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/entries": Endpoints.get_ListSupportTicketEntries;
    "/api/v1/organizations/{organization_id}/surcharges": Endpoints.get_ListSurcharges;
    "/api/v1/organizations/{organization_id}/task-labels": Endpoints.get_ListTaskLabels;
    "/api/v1/organizations/{organization_id}/task-recurrences": Endpoints.get_ListTaskRecurrences;
    "/api/v1/organizations/{organization_id}/tasks": Endpoints.get_ListTasks;
    "/api/v1/organizations/{organization_id}/tasks/{task_id}": Endpoints.get_GetTask;
    "/api/v1/organizations/{organization_id}/tasks/{task_id}/comments": Endpoints.get_ListTaskComments;
    "/api/v1/products/{product_id}": Endpoints.get_GetProduct;
    "/api/v1/projects/{project_id}/billing-summary": Endpoints.get_GetProjectBillingSummary;
    "/api/v1/projects/{project_id}/invoices": Endpoints.get_ListProjectInvoices;
    "/api/v1/projects/{project_id}/supplier-costs": Endpoints.get_GetProjectSupplierCosts;
    "/api/v1/quotes/{quote_id}": Endpoints.get_GetQuote;
    "/api/v1/quotes/{quote_id}/invoicing": Endpoints.get_GetQuoteInvoicing;
    "/api/v1/quotes/{quote_id}/pdf": Endpoints.get_ExportQuotePdf;
    "/api/v1/quotes/{quote_id}/plan-proposal": Endpoints.get_GetQuotePlanProposal;
    "/api/v1/service-rates/{service_rate_id}": Endpoints.get_GetServiceRate;
    "/api/v1/supplier-invoices/{supplier_invoice_id}": Endpoints.get_GetSupplierInvoice;
    "/api/v1/support/intake/tickets": Endpoints.get_ListIntakeTickets;
    "/api/v1/support/intake/tickets/{ticket_id}": Endpoints.get_GetIntakeTicket;
    "/api/v1/support/intake/tickets/{ticket_id}/attachments/{attachment_id}/url": Endpoints.get_GetIntakeAttachmentUrl;
    "/api/v1/users/@me/organizations": Endpoints.get_ListMyOrganizations;
    "/oauth/callback": Endpoints.get_CompleteAutomationCredentialOauth;
  };
  post: {
    "/api/v1/chat/channels/{channel_id}/messages": Endpoints.post_CreateMessage;
    "/api/v1/chat/channels/{channel_id}/threads": Endpoints.post_CreateThread;
    "/api/v1/chat/channels/{channel_id}/typing": Endpoints.post_StartTyping;
    "/api/v1/chat/channels/{channel_id}/webhooks": Endpoints.post_CreateWebhook;
    "/api/v1/chat/organizations/{organization_id}/categories": Endpoints.post_CreateCategory;
    "/api/v1/chat/organizations/{organization_id}/channels": Endpoints.post_CreateChannel;
    "/api/v1/chat/organizations/{organization_id}/direct-channels": Endpoints.post_OpenDirectChannel;
    "/api/v1/chat/webhooks/{webhook_id}/{token}": Endpoints.post_ExecuteWebhook;
    "/api/v1/customers/{customer_id}/contacts": Endpoints.post_CreateCustomerContact;
    "/api/v1/customers/{customer_id}/customer-contexts": Endpoints.post_CreateCustomerContext;
    "/api/v1/deals/{deal_id}/stage": Endpoints.post_MoveDealStage;
    "/api/v1/employees/{employee_id}/cost-bases": Endpoints.post_SetEmployeeCostBasis;
    "/api/v1/expenses/{expense_id}/pay": Endpoints.post_PayExpense;
    "/api/v1/expenses/{expense_id}/reimburse": Endpoints.post_ReimburseExpense;
    "/api/v1/field/time-entries/{time_entry_id}/photos": Endpoints.post_AttachTimeEntryPhoto;
    "/api/v1/field/time-entries/{time_entry_id}/recover": Endpoints.post_RecoverTimeEntry;
    "/api/v1/field/time-entries/{time_entry_id}/stop": Endpoints.post_StopTimeEntry;
    "/api/v1/files": Endpoints.post_UploadFile;
    "/api/v1/invitations/{token}/accept": Endpoints.post_AcceptInvitation;
    "/api/v1/invoices/{invoice_id}/cancel": Endpoints.post_CancelInvoice;
    "/api/v1/invoices/{invoice_id}/credit-notes": Endpoints.post_IssueCreditNote;
    "/api/v1/invoices/{invoice_id}/issue": Endpoints.post_IssueInvoice;
    "/api/v1/invoices/{invoice_id}/note": Endpoints.post_NoteInvoice;
    "/api/v1/invoices/{invoice_id}/payments": Endpoints.post_RecordInvoicePayment;
    "/api/v1/invoices/{invoice_id}/remind": Endpoints.post_RemindInvoice;
    "/api/v1/invoices/{invoice_id}/send": Endpoints.post_SendInvoice;
    "/api/v1/members/{member_id}/roles": Endpoints.post_AssignRole;
    "/api/v1/organizations": Endpoints.post_CreateOrganization;
    "/api/v1/organizations/{organization_id}/absence-types": Endpoints.post_CreateAbsenceType;
    "/api/v1/organizations/{organization_id}/absence-types/{absence_type_id}/archive": Endpoints.post_ArchiveAbsenceType;
    "/api/v1/organizations/{organization_id}/absence-types/{absence_type_id}/restore": Endpoints.post_RestoreAbsenceType;
    "/api/v1/organizations/{organization_id}/absences": Endpoints.post_CreateAbsence;
    "/api/v1/organizations/{organization_id}/absences/{absence_id}/approve": Endpoints.post_ApproveAbsence;
    "/api/v1/organizations/{organization_id}/absences/{absence_id}/cancel": Endpoints.post_CancelAbsence;
    "/api/v1/organizations/{organization_id}/absences/{absence_id}/reject": Endpoints.post_RejectAbsence;
    "/api/v1/organizations/{organization_id}/automation/collections": Endpoints.post_ImportCollection;
    "/api/v1/organizations/{organization_id}/automation/credentials": Endpoints.post_CreateAutomationCredential;
    "/api/v1/organizations/{organization_id}/automation/credentials/{credential_id}/oauth/disconnect": Endpoints.post_DisconnectAutomationCredentialOauth;
    "/api/v1/organizations/{organization_id}/automation/credentials/{credential_id}/oauth/start": Endpoints.post_StartAutomationCredentialOauth;
    "/api/v1/organizations/{organization_id}/automation/credentials/{credential_id}/rotate": Endpoints.post_RotateAutomationCredential;
    "/api/v1/organizations/{organization_id}/automation/expressions/evaluate": Endpoints.post_EvaluateExpression;
    "/api/v1/organizations/{organization_id}/automation/folders": Endpoints.post_CreateWorkflowFolder;
    "/api/v1/organizations/{organization_id}/automation/runs/{run_id}/replay": Endpoints.post_ReplayRun;
    "/api/v1/organizations/{organization_id}/automation/workflows": Endpoints.post_CreateWorkflow;
    "/api/v1/organizations/{organization_id}/automation/workflows/dry-run": Endpoints.post_DryRunWorkflow;
    "/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}/runs": Endpoints.post_StartRun;
    "/api/v1/organizations/{organization_id}/custom-units": Endpoints.post_CreateCustomUnit;
    "/api/v1/organizations/{organization_id}/customers": Endpoints.post_CreateCustomer;
    "/api/v1/organizations/{organization_id}/deals": Endpoints.post_CreateDeal;
    "/api/v1/organizations/{organization_id}/doe-models": Endpoints.post_CreateDoeModel;
    "/api/v1/organizations/{organization_id}/doe-models/from-starter": Endpoints.post_CreateDoeModelFromStarter;
    "/api/v1/organizations/{organization_id}/doe-models/{doe_model_id}/restore": Endpoints.post_RestoreDoeModel;
    "/api/v1/organizations/{organization_id}/doe-style/preview": Endpoints.post_PreviewDoeStyle;
    "/api/v1/organizations/{organization_id}/doe-style/preview/pages": Endpoints.post_PreviewDoeStylePages;
    "/api/v1/organizations/{organization_id}/does/{doe_id}/supplies/draft": Endpoints.post_DraftDoeSupply;
    "/api/v1/organizations/{organization_id}/equipment": Endpoints.post_CreateEquipment;
    "/api/v1/organizations/{organization_id}/expense-categories": Endpoints.post_CreateExpenseCategory;
    "/api/v1/organizations/{organization_id}/expenses": Endpoints.post_CreateExpense;
    "/api/v1/organizations/{organization_id}/extra-periods": Endpoints.post_CreateExtraPeriod;
    "/api/v1/organizations/{organization_id}/extra-periods/{period_id}/approve": Endpoints.post_ApproveExtraPeriod;
    "/api/v1/organizations/{organization_id}/extra-periods/{period_id}/cancel": Endpoints.post_CancelExtraPeriod;
    "/api/v1/organizations/{organization_id}/extra-periods/{period_id}/reject": Endpoints.post_RejectExtraPeriod;
    "/api/v1/organizations/{organization_id}/field/assignments/{task_assignment_id}/report": Endpoints.post_ReportAssignment;
    "/api/v1/organizations/{organization_id}/field/day-end": Endpoints.post_EndWorkingDay;
    "/api/v1/organizations/{organization_id}/field/tasks/{task_id}/intervention": Endpoints.post_ProposeMaintenanceIntervention;
    "/api/v1/organizations/{organization_id}/field/time-entries": Endpoints.post_StartTimeEntry;
    "/api/v1/organizations/{organization_id}/field/time-entries/declare": Endpoints.post_DeclareTimeEntry;
    "/api/v1/organizations/{organization_id}/invitations": Endpoints.post_CreateInvitation;
    "/api/v1/organizations/{organization_id}/invoices": Endpoints.post_CreateInvoice;
    "/api/v1/organizations/{organization_id}/leave-accounts": Endpoints.post_CreateLeaveAccount;
    "/api/v1/organizations/{organization_id}/leave-accounts/{account_id}/archive": Endpoints.post_ArchiveLeaveAccount;
    "/api/v1/organizations/{organization_id}/leave-accounts/{account_id}/restore": Endpoints.post_RestoreLeaveAccount;
    "/api/v1/organizations/{organization_id}/leave-adjustments": Endpoints.post_CreateLeaveAdjustment;
    "/api/v1/organizations/{organization_id}/maintenance/contracts": Endpoints.post_CreateMaintenanceContract;
    "/api/v1/organizations/{organization_id}/maintenance/contracts/{contract_id}/archive": Endpoints.post_ArchiveMaintenanceContract;
    "/api/v1/organizations/{organization_id}/maintenance/contracts/{contract_id}/duplicate": Endpoints.post_DuplicateMaintenanceContract;
    "/api/v1/organizations/{organization_id}/maintenance/interventions/{intervention_id}/reject": Endpoints.post_RejectMaintenanceIntervention;
    "/api/v1/organizations/{organization_id}/maintenance/interventions/{intervention_id}/validate": Endpoints.post_ValidateMaintenanceIntervention;
    "/api/v1/organizations/{organization_id}/maintenance/lines/{line_id}/interventions": Endpoints.post_RecordMaintenanceIntervention;
    "/api/v1/organizations/{organization_id}/maintenance/recap/plan": Endpoints.post_PlanMaintenanceLines;
    "/api/v1/organizations/{organization_id}/members": Endpoints.post_CreateMember;
    "/api/v1/organizations/{organization_id}/members/{member_id}/archive": Endpoints.post_ArchiveMember;
    "/api/v1/organizations/{organization_id}/members/{member_id}/contracts": Endpoints.post_CreateMemberContract;
    "/api/v1/organizations/{organization_id}/members/{member_id}/documents": Endpoints.post_AddMemberDocument;
    "/api/v1/organizations/{organization_id}/members/{member_id}/restore": Endpoints.post_RestoreMember;
    "/api/v1/organizations/{organization_id}/products": Endpoints.post_CreateProduct;
    "/api/v1/organizations/{organization_id}/project-templates": Endpoints.post_CreateProjectTemplate;
    "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}/instantiate": Endpoints.post_InstantiateProjectTemplate;
    "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}/restore": Endpoints.post_RestoreProjectTemplate;
    "/api/v1/organizations/{organization_id}/projects": Endpoints.post_CreateProject;
    "/api/v1/organizations/{organization_id}/projects/{project_id}/channel": Endpoints.post_CreateProjectChannel;
    "/api/v1/organizations/{organization_id}/projects/{project_id}/doe": Endpoints.post_CreateDoe;
    "/api/v1/organizations/{organization_id}/projects/{project_id}/restore": Endpoints.post_RestoreProject;
    "/api/v1/organizations/{organization_id}/quotes": Endpoints.post_CreateQuote;
    "/api/v1/organizations/{organization_id}/receipts": Endpoints.post_RecordReceipt;
    "/api/v1/organizations/{organization_id}/roles": Endpoints.post_CreateRole;
    "/api/v1/organizations/{organization_id}/service-rates": Endpoints.post_CreateServiceRate;
    "/api/v1/organizations/{organization_id}/supplier-invoices/import": Endpoints.post_ImportSupplierInvoice;
    "/api/v1/organizations/{organization_id}/suppliers": Endpoints.post_CreateSupplier;
    "/api/v1/organizations/{organization_id}/suppliers/{supplier_id}/archive": Endpoints.post_ArchiveSupplier;
    "/api/v1/organizations/{organization_id}/suppliers/{supplier_id}/restore": Endpoints.post_RestoreSupplier;
    "/api/v1/organizations/{organization_id}/support/api-keys": Endpoints.post_CreateSupportApiKey;
    "/api/v1/organizations/{organization_id}/support/entries/{entry_id}/attachments": Endpoints.post_AttachSupportEntryFile;
    "/api/v1/organizations/{organization_id}/support/entries/{entry_id}/send": Endpoints.post_SendSupportDraft;
    "/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/badges": Endpoints.post_AddSupportTicketBadge;
    "/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/entries": Endpoints.post_AddSupportTicketEntry;
    "/api/v1/organizations/{organization_id}/surcharges": Endpoints.post_CreateSurcharge;
    "/api/v1/organizations/{organization_id}/surcharges/{surcharge_id}/archive": Endpoints.post_ArchiveSurcharge;
    "/api/v1/organizations/{organization_id}/surcharges/{surcharge_id}/restore": Endpoints.post_RestoreSurcharge;
    "/api/v1/organizations/{organization_id}/task-labels": Endpoints.post_CreateTaskLabel;
    "/api/v1/organizations/{organization_id}/task-recurrences": Endpoints.post_CreateTaskRecurrence;
    "/api/v1/organizations/{organization_id}/tasks": Endpoints.post_CreateTask;
    "/api/v1/organizations/{organization_id}/tasks/bulk-assign": Endpoints.post_BulkAssignTasks;
    "/api/v1/organizations/{organization_id}/tasks/{task_id}/comments": Endpoints.post_CreateTaskComment;
    "/api/v1/projects/{project_id}/invoices/deposit": Endpoints.post_IssueProjectDeposit;
    "/api/v1/projects/{project_id}/invoices/final": Endpoints.post_IssueProjectFinalInvoice;
    "/api/v1/quotes/{quote_id}/invoices": Endpoints.post_CreateInvoiceFromQuote;
    "/api/v1/quotes/{quote_id}/plan": Endpoints.post_CreateQuotePlan;
    "/api/v1/quotes/{quote_id}/revision": Endpoints.post_ReviseQuote;
    "/api/v1/receipts/{receipt_id}/allocate": Endpoints.post_AllocateReceipt;
    "/api/v1/supplier-invoices/{supplier_invoice_id}/confirm": Endpoints.post_ConfirmSupplierInvoice;
    "/api/v1/supplier-invoices/{supplier_invoice_id}/reject": Endpoints.post_RejectSupplierInvoice;
    "/api/v1/support/intake/tickets": Endpoints.post_CreateIntakeTicket;
    "/api/v1/support/intake/tickets/{ticket_id}/attachments": Endpoints.post_AttachIntakeFile;
    "/api/v1/support/intake/tickets/{ticket_id}/messages": Endpoints.post_PostIntakeMessage;
    "/webhooks/{path}": Endpoints.post_ReceiveWebhook;
  };
  put: {
    "/api/v1/chat/channels/{channel_id}/permissions/everyone": Endpoints.put_UpsertEveryoneOverwrite;
    "/api/v1/chat/channels/{channel_id}/permissions/{target_type}/{target_id}": Endpoints.put_UpsertTargetOverwrite;
    "/api/v1/chat/channels/{channel_id}/read": Endpoints.put_MarkChannelRead;
    "/api/v1/chat/messages/{message_id}/reactions/{emoji}": Endpoints.put_AddReaction;
    "/api/v1/chat/notifications/{notification_id}/read": Endpoints.put_MarkNotificationRead;
    "/api/v1/chat/organizations/{organization_id}/notifications/read-all": Endpoints.put_MarkAllNotificationsRead;
    "/api/v1/chat/organizations/{organization_id}/presence": Endpoints.put_SetPresence;
    "/api/v1/members/{member_id}/employee-profile": Endpoints.put_UpsertEmployeeProfile;
    "/api/v1/members/{member_id}/rhythm": Endpoints.put_PutRhythm;
    "/api/v1/members/{member_id}/work-slots": Endpoints.put_PutWorkSlots;
    "/api/v1/organizations/{organization_id}/absence-types/{absence_type_id}": Endpoints.put_UpdateAbsenceType;
    "/api/v1/organizations/{organization_id}/automation/collections/{collection_id}": Endpoints.put_ReplaceCollection;
    "/api/v1/organizations/{organization_id}/automation/settings": Endpoints.put_UpdateAutomationSettings;
    "/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}/versions": Endpoints.put_SaveWorkflowVersion;
    "/api/v1/organizations/{organization_id}/billing-settings": Endpoints.put_ReplaceOrganizationBillingSettings;
    "/api/v1/organizations/{organization_id}/doe-models/{doe_model_id}": Endpoints.put_UpdateDoeModel;
    "/api/v1/organizations/{organization_id}/doe-settings": Endpoints.put_UpdateDoeSettings;
    "/api/v1/organizations/{organization_id}/does/{doe_id}": Endpoints.put_UpdateDoe;
    "/api/v1/organizations/{organization_id}/leave-accounts/{account_id}": Endpoints.put_UpdateLeaveAccount;
    "/api/v1/organizations/{organization_id}/maintenance/contracts/{contract_id}": Endpoints.put_UpdateMaintenanceContract;
    "/api/v1/organizations/{organization_id}/members/{member_id}/contracts/{contract_id}": Endpoints.put_UpdateMemberContract;
    "/api/v1/organizations/{organization_id}/members/{member_id}/day-overrides/{date}": Endpoints.put_PutDayOverride;
    "/api/v1/organizations/{organization_id}/members/{member_id}/profile": Endpoints.put_PutMemberProfile;
    "/api/v1/organizations/{organization_id}/numbering/{kind}": Endpoints.put_SetOrganizationNextNumber;
    "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}/tasks": Endpoints.put_ReplaceProjectTemplateTasks;
    "/api/v1/organizations/{organization_id}/suppliers/{supplier_id}": Endpoints.put_UpdateSupplier;
    "/api/v1/organizations/{organization_id}/support/settings": Endpoints.put_ReplaceSupportSettings;
    "/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/assignee": Endpoints.put_AssignSupportTicket;
    "/api/v1/organizations/{organization_id}/surcharges/{surcharge_id}": Endpoints.put_UpdateSurcharge;
    "/api/v1/supplier-invoice-lines/{supplier_invoice_line_id}/allocations": Endpoints.put_ReplaceSupplierInvoiceLineAllocations;
  };
};

// </EndpointByMethod>

// <EndpointByMethod.Shorthands>
export type PatchEndpoints = EndpointByMethod["patch"];
export type DeleteEndpoints = EndpointByMethod["delete"];
export type GetEndpoints = EndpointByMethod["get"];
export type PostEndpoints = EndpointByMethod["post"];
export type PutEndpoints = EndpointByMethod["put"];
// </EndpointByMethod.Shorthands>

// <ApiClientTypes>
export type EndpointParameters = {
  body?: unknown;
  query?: Record<string, unknown>;
  header?: Record<string, unknown>;
  path?: Record<string, unknown>;
};

export type MutationMethod = "post" | "put" | "patch" | "delete";
export type Method = "get" | "head" | "options" | MutationMethod;

type RequestFormat = "json" | "form-data" | "form-url" | "binary" | "text";

export type DefaultEndpoint = {
  parameters?: EndpointParameters | undefined;
  responses?: Record<string, unknown>;
  responseHeaders?: Record<string, unknown>;
};

export type Endpoint<TConfig extends DefaultEndpoint = DefaultEndpoint> = {
  operationId: string;
  method: Method;
  path: string;
  requestFormat: RequestFormat;
  parameters?: TConfig["parameters"];
  meta: {
    alias: string;
    hasParameters: boolean;
    areParametersRequired: boolean;
  };
  responses?: TConfig["responses"];
  responseHeaders?: TConfig["responseHeaders"];
};

export interface Fetcher {
  decodePathParams?: (path: string, pathParams: Record<string, string>) => string;
  encodeSearchParams?: (searchParams: Record<string, unknown> | undefined) => URLSearchParams;
  //
  fetch: (input: {
    method: Method;
    url: URL;
    urlSearchParams?: URLSearchParams | undefined;
    parameters?: EndpointParameters | undefined;
    path: string;
    overrides?: RequestInit;
    throwOnStatusError?: boolean;
  }) => Promise<Response>;
  parseResponseData?: (response: Response) => Promise<unknown>;
}

export const successStatusCodes = [
  200, 201, 202, 203, 204, 205, 206, 207, 208, 226, 300, 301, 302, 303, 304, 305, 306, 307, 308,
] as const;
export type SuccessStatusCode = (typeof successStatusCodes)[number];

export const errorStatusCodes = [
  400, 401, 402, 403, 404, 405, 406, 407, 408, 409, 410, 411, 412, 413, 414, 415, 416, 417, 418, 421, 422, 423, 424,
  425, 426, 428, 429, 431, 451, 500, 501, 502, 503, 504, 505, 506, 507, 508, 510, 511,
] as const;
export type ErrorStatusCode = (typeof errorStatusCodes)[number];

// Taken from https://github.com/unjs/fetchdts/blob/ec4eaeab5d287116171fc1efd61f4a1ad34e4609/src/fetch.ts#L3
export interface TypedHeaders<TypedHeaderValues extends Record<string, string> | unknown>
  extends Omit<Headers, "append" | "delete" | "get" | "getSetCookie" | "has" | "set" | "forEach"> {
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/append) */
  append: <Name extends Extract<keyof TypedHeaderValues, string> | (string & {})>(
    name: Name,
    value: Lowercase<Name> extends keyof TypedHeaderValues ? TypedHeaderValues[Lowercase<Name>] : string,
  ) => void;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/delete) */
  delete: <Name extends Extract<keyof TypedHeaderValues, string> | (string & {})>(name: Name) => void;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/get) */
  get: <Name extends Extract<keyof TypedHeaderValues, string> | (string & {})>(
    name: Name,
  ) => (Lowercase<Name> extends keyof TypedHeaderValues ? TypedHeaderValues[Lowercase<Name>] : string) | null;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/getSetCookie) */
  getSetCookie: () => string[];
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/has) */
  has: <Name extends Extract<keyof TypedHeaderValues, string> | (string & {})>(name: Name) => boolean;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/set) */
  set: <Name extends Extract<keyof TypedHeaderValues, string> | (string & {})>(
    name: Name,
    value: Lowercase<Name> extends keyof TypedHeaderValues ? TypedHeaderValues[Lowercase<Name>] : string,
  ) => void;
  forEach: (
    callbackfn: (
      value: TypedHeaderValues[keyof TypedHeaderValues] | (string & {}),
      key: Extract<keyof TypedHeaderValues, string> | (string & {}),
      parent: TypedHeaders<TypedHeaderValues>,
    ) => void,
    thisArg?: any,
  ) => void;
}

/** @see https://developer.mozilla.org/en-US/docs/Web/API/Response */
export interface TypedSuccessResponse<TSuccess, TStatusCode, THeaders>
  extends Omit<Response, "ok" | "status" | "json" | "headers"> {
  ok: true;
  status: TStatusCode;
  headers: never extends THeaders ? Headers : TypedHeaders<THeaders>;
  data: TSuccess;
  /** [MDN Reference](https://developer.mozilla.org/en-US/docs/Web/API/Response/json) */
  json: () => Promise<TSuccess>;
}

/** @see https://developer.mozilla.org/en-US/docs/Web/API/Response */
export interface TypedErrorResponse<TData, TStatusCode, THeaders>
  extends Omit<Response, "ok" | "status" | "json" | "headers"> {
  ok: false;
  status: TStatusCode;
  headers: never extends THeaders ? Headers : TypedHeaders<THeaders>;
  data: TData;
  /** [MDN Reference](https://developer.mozilla.org/en-US/docs/Web/API/Response/json) */
  json: () => Promise<TData>;
}

export type TypedApiResponse<TAllResponses extends Record<string | number, unknown> = {}, THeaders = {}> = {
  [K in keyof TAllResponses]: K extends string
    ? K extends `${infer TStatusCode extends number}`
      ? TStatusCode extends SuccessStatusCode
        ? TypedSuccessResponse<TAllResponses[K], TStatusCode, K extends keyof THeaders ? THeaders[K] : never>
        : TypedErrorResponse<TAllResponses[K], TStatusCode, K extends keyof THeaders ? THeaders[K] : never>
      : never
    : K extends number
      ? K extends SuccessStatusCode
        ? TypedSuccessResponse<TAllResponses[K], K, K extends keyof THeaders ? THeaders[K] : never>
        : TypedErrorResponse<TAllResponses[K], K, K extends keyof THeaders ? THeaders[K] : never>
      : never;
}[keyof TAllResponses];

export type SafeApiResponse<TEndpoint> = TEndpoint extends { responses: infer TResponses }
  ? TResponses extends Record<string, unknown>
    ? TypedApiResponse<TResponses, TEndpoint extends { responseHeaders: infer THeaders } ? THeaders : never>
    : never
  : never;

export type InferResponseByStatus<TEndpoint, TStatusCode> = Extract<
  SafeApiResponse<TEndpoint>,
  { status: TStatusCode }
>;

type RequiredKeys<T> = {
  [P in keyof T]-?: undefined extends T[P] ? never : P;
}[keyof T];

type MaybeOptionalArg<T> = RequiredKeys<T> extends never ? [config?: T] : [config: T];
type NotNever<T> = [T] extends [never] ? false : true;

// </ApiClientTypes>

// <TypedStatusError>
export class TypedStatusError<TData = unknown> extends Error {
  response: TypedErrorResponse<TData, ErrorStatusCode, unknown>;
  status: number;
  constructor(response: TypedErrorResponse<TData, ErrorStatusCode, unknown>) {
    super(`HTTP ${response.status}: ${response.statusText}`);
    this.name = "TypedStatusError";
    this.response = response;
    this.status = response.status;
  }
}
// </TypedStatusError>

// <ApiClient>
export class ApiClient {
  baseUrl: string = "";
  successStatusCodes = successStatusCodes;
  errorStatusCodes = errorStatusCodes;

  constructor(public fetcher: Fetcher) {}

  setBaseUrl(baseUrl: string) {
    this.baseUrl = baseUrl;
    return this;
  }

  /**
   * Replace path parameters in URL
   * Supports both OpenAPI format {param} and Express format :param
   */
  defaultDecodePathParams = (url: string, params: Record<string, string>): string => {
    return url
      .replace(/{(\w+)}/g, (_, key: string) => params[key] || `{${key}}`)
      .replace(/:([a-zA-Z0-9_]+)/g, (_, key: string) => params[key] || `:${key}`);
  };

  /** Uses URLSearchParams, skips null/undefined values */
  defaultEncodeSearchParams = (queryParams: Record<string, unknown> | undefined): URLSearchParams | undefined => {
    if (!queryParams) return;

    const searchParams = new URLSearchParams();
    Object.entries(queryParams).forEach(([key, value]) => {
      if (value != null) {
        // Skip null/undefined values
        if (Array.isArray(value)) {
          value.forEach((val) => val != null && searchParams.append(key, String(val)));
        } else {
          searchParams.append(key, String(value));
        }
      }
    });

    return searchParams;
  };

  defaultParseResponseData = async (response: Response): Promise<unknown> => {
    const contentType = response.headers.get("content-type") ?? "";
    if (contentType.startsWith("text/")) {
      return await response.text();
    }

    if (contentType === "application/octet-stream") {
      return await response.arrayBuffer();
    }

    if (
      contentType.includes("application/json") ||
      (contentType.includes("application/") && contentType.includes("json")) ||
      contentType === "*/*"
    ) {
      try {
        return await response.json();
      } catch {
        return undefined;
      }
    }

    return;
  };

  // <ApiClient.patch>
  patch<Path extends keyof PatchEndpoints, TEndpoint extends PatchEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
          : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
    >
  ): Promise<Extract<InferResponseByStatus<TEndpoint, SuccessStatusCode>, { data: {} }>["data"]>;

  patch<Path extends keyof PatchEndpoints, TEndpoint extends PatchEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
          : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
    >
  ): Promise<SafeApiResponse<TEndpoint>>;

  patch<Path extends keyof PatchEndpoints, _TEndpoint extends PatchEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<any>
  ): Promise<any> {
    return this.request("patch", path, ...params);
  }
  // </ApiClient.patch>

  // <ApiClient.delete>
  delete<Path extends keyof DeleteEndpoints, TEndpoint extends DeleteEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
          : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
    >
  ): Promise<Extract<InferResponseByStatus<TEndpoint, SuccessStatusCode>, { data: {} }>["data"]>;

  delete<Path extends keyof DeleteEndpoints, TEndpoint extends DeleteEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
          : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
    >
  ): Promise<SafeApiResponse<TEndpoint>>;

  delete<Path extends keyof DeleteEndpoints, _TEndpoint extends DeleteEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<any>
  ): Promise<any> {
    return this.request("delete", path, ...params);
  }
  // </ApiClient.delete>

  // <ApiClient.get>
  get<Path extends keyof GetEndpoints, TEndpoint extends GetEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
          : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
    >
  ): Promise<Extract<InferResponseByStatus<TEndpoint, SuccessStatusCode>, { data: {} }>["data"]>;

  get<Path extends keyof GetEndpoints, TEndpoint extends GetEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
          : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
    >
  ): Promise<SafeApiResponse<TEndpoint>>;

  get<Path extends keyof GetEndpoints, _TEndpoint extends GetEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<any>
  ): Promise<any> {
    return this.request("get", path, ...params);
  }
  // </ApiClient.get>

  // <ApiClient.post>
  post<Path extends keyof PostEndpoints, TEndpoint extends PostEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
          : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
    >
  ): Promise<Extract<InferResponseByStatus<TEndpoint, SuccessStatusCode>, { data: {} }>["data"]>;

  post<Path extends keyof PostEndpoints, TEndpoint extends PostEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
          : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
    >
  ): Promise<SafeApiResponse<TEndpoint>>;

  post<Path extends keyof PostEndpoints, _TEndpoint extends PostEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<any>
  ): Promise<any> {
    return this.request("post", path, ...params);
  }
  // </ApiClient.post>

  // <ApiClient.put>
  put<Path extends keyof PutEndpoints, TEndpoint extends PutEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
          : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
    >
  ): Promise<Extract<InferResponseByStatus<TEndpoint, SuccessStatusCode>, { data: {} }>["data"]>;

  put<Path extends keyof PutEndpoints, TEndpoint extends PutEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
          : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
    >
  ): Promise<SafeApiResponse<TEndpoint>>;

  put<Path extends keyof PutEndpoints, _TEndpoint extends PutEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<any>
  ): Promise<any> {
    return this.request("put", path, ...params);
  }
  // </ApiClient.put>

  // <ApiClient.request>
  /**
   * Generic request method with full type-safety for any endpoint
   */
  request<
    TMethod extends keyof EndpointByMethod,
    TPath extends keyof EndpointByMethod[TMethod],
    TEndpoint extends EndpointByMethod[TMethod][TPath],
  >(
    method: TMethod,
    path: TPath,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
          : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
    >
  ): Promise<Extract<InferResponseByStatus<TEndpoint, SuccessStatusCode>, { data: {} }>["data"]>;

  request<
    TMethod extends keyof EndpointByMethod,
    TPath extends keyof EndpointByMethod[TMethod],
    TEndpoint extends EndpointByMethod[TMethod][TPath],
  >(
    method: TMethod,
    path: TPath,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
          : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
    >
  ): Promise<SafeApiResponse<TEndpoint>>;

  request<
    TMethod extends keyof EndpointByMethod,
    TPath extends keyof EndpointByMethod[TMethod],
    TEndpoint extends EndpointByMethod[TMethod][TPath],
  >(method: TMethod, path: TPath, ...params: MaybeOptionalArg<any>): Promise<any> {
    const requestParams = params[0];
    const withResponse = requestParams?.withResponse;
    const {
      withResponse: _,
      throwOnStatusError = withResponse ? false : true,
      overrides,
      ...fetchParams
    } = requestParams || {};

    const parametersToSend: EndpointParameters = {};
    if (requestParams?.body !== undefined) (parametersToSend as any).body = requestParams.body;
    if (requestParams?.query !== undefined) (parametersToSend as any).query = requestParams.query;
    if (requestParams?.header !== undefined) (parametersToSend as any).header = requestParams.header;
    if (requestParams?.path !== undefined) (parametersToSend as any).path = requestParams.path;

    const resolvedPath = (this.fetcher.decodePathParams ?? this.defaultDecodePathParams)(
      this.baseUrl + (path as string),
      (parametersToSend.path ?? {}) as Record<string, string>,
    );
    const url = new URL(resolvedPath);
    const urlSearchParams = (this.fetcher.encodeSearchParams ?? this.defaultEncodeSearchParams)(parametersToSend.query);

    const promise = this.fetcher
      .fetch({
        method: method,
        path: path as string,
        url,
        urlSearchParams,
        parameters: Object.keys(fetchParams).length ? fetchParams : undefined,
        overrides,
        throwOnStatusError,
      })
      .then(async (response) => {
        const data = await (this.fetcher.parseResponseData ?? this.defaultParseResponseData)(response);
        const typedResponse = Object.assign(response, {
          data: data,
          json: () => Promise.resolve(data),
        }) as SafeApiResponse<TEndpoint>;

        if (throwOnStatusError && errorStatusCodes.includes(response.status as never)) {
          throw new TypedStatusError(typedResponse as never);
        }

        return withResponse ? typedResponse : data;
      });

    return promise as Extract<InferResponseByStatus<TEndpoint, SuccessStatusCode>, { data: {} }>["data"];
  }
  // </ApiClient.request>
}

export function createApiClient(fetcher: Fetcher, baseUrl?: string) {
  return new ApiClient(fetcher).setBaseUrl(baseUrl ?? "");
}

/**
 Example usage:
 const api = createApiClient((method, url, params) =>
   fetch(url, { method, body: JSON.stringify(params) }).then((res) => res.json()),
 );
 api.get("/users").then((users) => console.log(users));
 api.post("/users", { body: { name: "John" } }).then((user) => console.log(user));
 api.put("/users/:id", { path: { id: 1 }, body: { name: "John" } }).then((user) => console.log(user));

 // With error handling
 const result = await api.get("/users/{id}", { path: { id: "123" }, withResponse: true });
 if (result.ok) {
   // Access data directly
   const user = result.data;
   console.log(user);

   // Or use the json() method for compatibility
   const userFromJson = await result.json();
   console.log(userFromJson);
 } else {
   const error = result.data;
   console.error(`Error ${result.status}:`, error);
 }
*/

// </ApiClient>
