export namespace Schemas {
  // <Schemas>
  export type AbsenceId = string;
  export type AbsenceKind = "LEAVE" | "SICK" | "UNAVAILABLE";
  export type MemberId = string;
  export type OrganizationId = string;
  export type AbsenceResponse = {
    all_day: boolean;
    created_at: string;
    ends_at: string;
    id: AbsenceId;
    kind: AbsenceKind;
    member_id: MemberId;
    note?: (string | null) | undefined;
    organization_id: OrganizationId;
    starts_at: string;
    updated_at: string;
  };
  export type AmendAssignmentReportRequest = { comment?: (string | null) | undefined; reported_minutes: number };
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
  export type AttachmentResponse = { filename: string; mime_type: string; size_bytes: number; storage_key: string };
  export type AuthRequirementResponse = "None" | { Exactly: string } | { AnyOf: Array<string> };
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
  export type ProjectId = string;
  export type QuoteId = string;
  export type TaskRecurrenceId = string;
  export type TaskStatus = "PLANNED" | "IN_PROGRESS" | "DONE" | "CANCELLED";
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
  export type ChannelType = "TEXT" | "THREAD";
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
    topic?: (string | null) | undefined;
    updated_at: string;
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
  export type ConnectorDescriptorResponse = {
    auth: AuthRequirementResponse;
    family: string;
    fields: Array<FieldResponse>;
    kind: string;
    label: string;
    output_example: unknown;
    version: number;
  };
  export type ConnectorsResponse = {
    auth_schemes: Array<AuthSchemeResponse>;
    connectors: Array<ConnectorDescriptorResponse>;
  };
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
    ends_at: string;
    kind: AbsenceKind;
    member_id: MemberId;
    note?: (string | null) | undefined;
    starts_at: string;
  };
  export type CreateCategoryRequest = { name: string; position: number };
  export type CreateChannelRequest = {
    category_id?: (null | CategoryId) | undefined;
    name: string;
    position: number;
    topic?: (string | null) | undefined;
  };
  export type CredentialOriginRequest = "supplied" | "generated";
  export type CreateCredentialRequest = {
    data?: unknown | undefined;
    kind: string;
    name: string;
    origin: CredentialOriginRequest;
  };
  export type CreateCustomerContactRequest = {
    email?: (string | null) | undefined;
    first_name: string;
    is_primary: boolean;
    last_name: string;
    phone?: (string | null) | undefined;
    role?: (string | null) | undefined;
  };
  export type CreateCustomerContextRequest = {
    address_line?: (string | null) | undefined;
    city?: (string | null) | undefined;
    label: string;
    photo_key?: (string | null) | undefined;
    postal_code?: (string | null) | undefined;
  };
  export type CustomerPipelineStage = "NEW" | "CONTACTED" | "QUALIFIED" | "QUOTE_SENT" | "WON" | "LOST";
  export type CustomerStatus = "PROSPECT" | "CLIENT" | "ARCHIVED";
  export type CreateCustomerRequest = {
    email?: (string | null) | undefined;
    name: string;
    phone?: (string | null) | undefined;
    pipeline_stage: CustomerPipelineStage;
    registration_number?: (string | null) | undefined;
    status: CustomerStatus;
  };
  export type CreateEquipmentRequest = { hourly_rate_cents: number; name: string };
  export type CreateInvitationRequest = Partial<{ expires_at: string | null; member_id: null | MemberId }>;
  export type DeliveryAddressRequest = {
    city: string;
    country: string;
    line1: string;
    line2?: (string | null) | undefined;
    postal_code: string;
  };
  export type InvoiceKind = "STANDARD" | "DEPOSIT" | "FINAL" | "CREDIT_NOTE";
  export type InvoiceLineRequest = {
    label: string;
    quantity: string;
    unit_price_cents: number;
    vat_rate_basis_points?: (number | null) | undefined;
  };
  export type OperationNature = "GOODS" | "SERVICES" | "BOTH";
  export type CreateInvoiceRequest = {
    customer_context_id: CustomerContextId;
    customer_id: CustomerId;
    delivery_address?: (null | DeliveryAddressRequest) | undefined;
    due_at?: (string | null) | undefined;
    kind: InvoiceKind;
    lines: Array<InvoiceLineRequest>;
    notes?: (string | null) | undefined;
    operation_nature?: (null | OperationNature) | undefined;
    project_id?: (null | ProjectId) | undefined;
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
  export type ServiceRateUnit = "FLAT_RATE" | "HOUR" | "DAY" | "UNIT" | "ML" | "M2" | "M3" | "KG" | "TONNE" | "LITRE";
  export type CreateProductRequest = {
    default_vat_rate_bp?: (number | null) | undefined;
    description?: (string | null) | undefined;
    name: string;
    sku?: (string | null) | undefined;
    unit: ServiceRateUnit;
    unit_price_cents: number;
  };
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
  export type ServiceRateId = string;
  export type QuoteLineRequest = {
    label: string;
    notes?: (string | null) | undefined;
    photo_keys: Array<string>;
    quantity: string;
    service_rate_id?: (null | ServiceRateId) | undefined;
    unit: ServiceRateUnit;
    unit_price_cents: number;
    vat_rate_bp?: (number | null) | undefined;
  };
  export type CreateQuoteRequest = {
    customer_context_id: CustomerContextId;
    customer_id: CustomerId;
    lines: Array<QuoteLineRequest>;
    title: string;
  };
  export type CreateServiceRateRequest = {
    default_vat_rate_bp?: (number | null) | undefined;
    label: string;
    rate_cents: number;
    unit: ServiceRateUnit;
  };
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
    title: string;
  };
  export type CreateThreadRequest = { name: string; origin_message_id?: (null | MessageId) | undefined };
  export type CreateWebhookRequest = { avatar_url?: (string | null) | undefined; name: string };
  export type CreateWorkflowRequest = { description?: (string | null) | undefined; name: string };
  export type InvitationId = string;
  export type InvitationResponse = {
    created_at: string;
    expires_at: string;
    id: InvitationId;
    member_id?: (null | MemberId) | undefined;
    organization_id: OrganizationId;
  };
  export type CreatedInvitationResponse = InvitationResponse & { token: string };
  export type CredentialResponse = {
    created_at: string;
    id: string;
    kind: string;
    name: string;
    organization_id: OrganizationId;
    origin: string;
    updated_at: string;
  };
  export type CredentialWithSecretResponse = CredentialResponse & { secret: unknown };
  export type CustomerContactId = string;
  export type CustomerContactResponse = {
    created_at: string;
    customer_id: CustomerId;
    email?: (string | null) | undefined;
    first_name: string;
    id: CustomerContactId;
    is_primary: boolean;
    last_name: string;
    phone?: (string | null) | undefined;
    role?: (string | null) | undefined;
    updated_at: string;
  };
  export type CustomerContextResponse = {
    address_line?: (string | null) | undefined;
    city?: (string | null) | undefined;
    created_at: string;
    customer_id: CustomerId;
    id: CustomerContextId;
    label: string;
    photo_key?: (string | null) | undefined;
    postal_code?: (string | null) | undefined;
    updated_at: string;
  };
  export type CustomerOutstandingBalanceResponse = {
    customer_id: CustomerId;
    oldest_due_at?: (string | null) | undefined;
    outstanding_cents: number;
  };
  export type CustomerResponse = {
    created_at: string;
    email?: (string | null) | undefined;
    id: CustomerId;
    name: string;
    organization_id: OrganizationId;
    phone?: (string | null) | undefined;
    pipeline_stage: CustomerPipelineStage;
    registration_number?: (string | null) | undefined;
    status: CustomerStatus;
    updated_at: string;
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
  export type DeclareTimeEntryRequest = { ended_at: string; started_at: string; task_id: TaskId };
  export type DeleteScopeRequest = "THIS_OCCURRENCE" | "THIS_AND_FOLLOWING";
  export type DeliveryAddressResponse = {
    city: string;
    country: string;
    line1: string;
    line2?: (string | null) | undefined;
    postal_code: string;
  };
  export type EdgeDto = { branch?: (null | BranchDto) | undefined; from: string; to: string };
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
  export type EquipmentResponse = {
    created_at: string;
    hourly_rate_cents: number;
    id: EquipmentId;
    name: string;
    organization_id: OrganizationId;
    updated_at: string;
  };
  export type EventDescriptorResponse = {
    label: string;
    name: string;
    payload_example: unknown;
    subject_kind: string;
    version: number;
  };
  export type ExecuteWebhookRequest = { components?: (Array<Component> | null) | undefined; content: string };
  export type FieldTaskResponse = {
    all_day: boolean;
    customer_context_id?: (null | CustomerContextId) | undefined;
    customer_id?: (null | CustomerId) | undefined;
    description?: (string | null) | undefined;
    ends_at?: (string | null) | undefined;
    id: TaskId;
    starts_at?: (string | null) | undefined;
    status: TaskStatus;
    task_assignment_id: TaskAssignmentId;
    title: string;
  };
  export type FileUploadResponse = { key: string; mime_type: string; size_bytes: number };
  export type FileUrlResponse = { expires_at: string; url: string };
  export type PlacedConnectorDto = {
    config: Record<string, unknown>;
    credential_id?: (string | null) | undefined;
    id: string;
    kind: string;
    version: number;
  };
  export type GraphDto = { connectors: Array<PlacedConnectorDto>; edges: Array<EdgeDto> };
  export type GraphErrorResponse = {
    connector_id?: (string | null) | undefined;
    field?: (string | null) | undefined;
    message: string;
  };
  export type GraphInvalidDetails = { errors: Array<GraphErrorResponse> };
  export type GraphInvalidBody = { code: string; details: GraphInvalidDetails; message: string; status: number };
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
  export type InvoiceBalanceResponse = {
    credited_cents: number;
    gross_cents: number;
    paid_cents: number;
    remaining_cents: number;
  };
  export type InvoiceId = string;
  export type InvoiceLineId = string;
  export type InvoiceLineResponse = {
    created_at: string;
    id: InvoiceLineId;
    invoice_id: InvoiceId;
    label: string;
    organization_id: OrganizationId;
    position: number;
    quantity: string;
    unit_price_cents: number;
    updated_at: string;
    vat_rate_basis_points?: (number | null) | undefined;
  };
  export type InvoicePaymentId = string;
  export type UserId = string;
  export type InvoicePaymentResponse = {
    amount_cents: number;
    created_at: string;
    deleted_at?: (string | null) | undefined;
    deleted_by?: (null | UserId) | undefined;
    id: InvoicePaymentId;
    invoice_id: InvoiceId;
    method: string;
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
    created_at: string;
    customer_context_id: CustomerContextId;
    customer_id: CustomerId;
    delivery_address?: (null | DeliveryAddressResponse) | undefined;
    due_at?: (string | null) | undefined;
    gross_cents: number;
    id: InvoiceId;
    issued_at?: (string | null) | undefined;
    kind: InvoiceKind;
    lines: Array<InvoiceLineResponse>;
    net_cents: number;
    notes?: (string | null) | undefined;
    number?: (string | null) | undefined;
    operation_nature?: (null | OperationNature) | undefined;
    organization_id: OrganizationId;
    project_id?: (null | ProjectId) | undefined;
    source_invoice_id?: (null | InvoiceId) | undefined;
    status: InvoiceStatus;
    updated_at: string;
    vat_breakdown: Array<InvoiceVatBreakdownLineResponse>;
  };
  export type IssueCreditNoteRequest = {
    allow_exceeding_invoice_total?: boolean | undefined;
    lines: Array<InvoiceLineRequest>;
    notes?: (string | null) | undefined;
  };
  export type IssueDepositRequest = {
    allow_exceeding_total?: boolean | undefined;
    due_at?: (string | null) | undefined;
    notes?: (string | null) | undefined;
    percentage_bp: number;
  };
  export type IssueFinalInvoiceRequest = Partial<{
    allow_exceeding_total: boolean;
    due_at: string | null;
    notes: string | null;
  }>;
  export type IssueInvoiceRequest = Partial<{ allow_exceeding_total: boolean }>;
  export type MarkChannelReadRequest = { message_id: MessageId };
  export type MemberAccountResponse = { email: string; name: string };
  export type MissingCost = "HOURLY_RATE" | "MONTHLY_COST" | "CONTRACTED_HOURS" | "NO_COST_BASIS";
  export type MemberProfitability = {
    labour_cost_cents: number;
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
  export type WebhookId = string;
  export type RoleId = string;
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
  export type NotificationId = string;
  export type NotificationResponse = {
    channel_id: ChannelId;
    created_at: string;
    id: NotificationId;
    kind: string;
    message_id: MessageId;
    read_at?: (string | null) | undefined;
  };
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
  export type PresenceStatus = "ONLINE" | "OFFLINE" | "DND";
  export type PresenceResponse = {
    organization_id: OrganizationId;
    status: PresenceStatus;
    updated_at: string;
    user_id: UserId;
  };
  export type ProductId = string;
  export type ProductResponse = {
    created_at: string;
    default_vat_rate_bp?: (number | null) | undefined;
    description?: (string | null) | undefined;
    id: ProductId;
    name: string;
    organization_id: OrganizationId;
    sku?: (string | null) | undefined;
    unit: ServiceRateUnit;
    unit_price_cents: number;
    updated_at: string;
  };
  export type ProjectProfitability = {
    customer_id?: (null | CustomerId) | undefined;
    equipment_cost_cents: number;
    expenses_cents: number;
    labour_cost_cents: number;
    margin_cents?: (number | null) | undefined;
    members_without_rate: Array<MemberId>;
    name: string;
    occupied_minutes: number;
    overlapping_minutes: number;
    planned_minutes: number;
    project_id: ProjectId;
    quoted_cents?: (number | null) | undefined;
  };
  export type ProfitabilityResponse = {
    double_booked: Array<ProjectProfitability>;
    incomplete: Array<ProjectProfitability>;
    least_profitable: Array<ProjectProfitability>;
    members: Array<MemberProfitability>;
    most_profitable: Array<ProjectProfitability>;
    projects: Array<ProjectProfitability>;
  };
  export type ProjectBillingSummaryResponse = {
    billed_cents: number;
    project_id: ProjectId;
    quoted_cents?: (number | null) | undefined;
    remaining_cents?: (number | null) | undefined;
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
  export type RhythmSlotRequest = { ends_minute: number; starts_minute: number; weekday: number };
  export type PutRhythmRequest = {
    effective_from: string;
    effective_to?: (string | null) | undefined;
    slots: Array<RhythmSlotRequest>;
  };
  export type WorkSlotRequest = { ends_minute: number; starts_minute: number; work_date: string };
  export type PutWorkSlotsRequest = { slots: Array<WorkSlotRequest> };
  export type QuoteLineResponse = {
    created_at: string;
    id: QuoteLineId;
    label: string;
    notes?: (string | null) | undefined;
    organization_id: OrganizationId;
    photo_keys: Array<string>;
    quantity: string;
    quote_id: QuoteId;
    service_rate_id?: (null | ServiceRateId) | undefined;
    unit: ServiceRateUnit;
    unit_price_cents: number;
    updated_at: string;
    vat_rate_bp?: (number | null) | undefined;
  };
  export type QuoteStatus = "DRAFT" | "SENT" | "ACCEPTED" | "DECLINED" | "CANCELLED";
  export type QuoteVatBreakdownLineResponse = { rate_bp: number; vat_cents: number };
  export type QuoteResponse = {
    created_at: string;
    customer_context_id: CustomerContextId;
    customer_id: CustomerId;
    gross_cents: number;
    id: QuoteId;
    lines: Array<QuoteLineResponse>;
    net_cents: number;
    organization_id: OrganizationId;
    reference?: (string | null) | undefined;
    status: QuoteStatus;
    title: string;
    updated_at: string;
    vat_breakdown: Array<QuoteVatBreakdownLineResponse>;
  };
  export type TaskProposalResponse = {
    quote_line_id: QuoteLineId;
    suggested_minutes?: (number | null) | undefined;
    title: string;
  };
  export type QuotePlanProposalResponse = { quote: QuoteResponse; tasks: Array<TaskProposalResponse> };
  export type QuotePlanResponse = { project: PlannedProjectResponse; tasks: Array<PlannedTaskResponse> };
  export type RecordInvoicePaymentRequest = {
    allow_exceeding_total?: boolean | undefined;
    amount_cents: number;
    method: string;
    note?: (string | null) | undefined;
    paid_on: string;
    reference?: (string | null) | undefined;
  };
  export type RecoverTimeEntryRequest = { ended_at: string };
  export type RecurrenceRuleResponse =
    | { frequency: "DAILY" }
    | { frequency: "WEEKLY"; weekdays: Array<number> }
    | { day_of_month: number; frequency: "MONTHLY" };
  export type ReplaceProjectTemplateTasksRequest = { tasks: Array<ProjectTemplateTaskShapeRequest> };
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
  export type RunResponse = {
    created_at: string;
    error?: (string | null) | undefined;
    finished_at?: (string | null) | undefined;
    id: string;
    next_attempt_at?: (string | null) | undefined;
    organization_id: OrganizationId;
    started_at?: (string | null) | undefined;
    status: string;
    trigger_event_id?: (string | null) | undefined;
    trigger_payload?: unknown | undefined;
    workflow_id: string;
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
    started_at?: (string | null) | undefined;
    status: string;
  };
  export type RunDetailResponse = RunResponse & { steps: Array<RunStepResponse> };
  export type SaveWorkflowVersionRequest = { graph: GraphDto };
  export type ServiceRateResponse = {
    created_at: string;
    default_vat_rate_bp?: (number | null) | undefined;
    id: ServiceRateId;
    label: string;
    organization_id: OrganizationId;
    rate_cents: number;
    unit: ServiceRateUnit;
    updated_at: string;
  };
  export type SetEmployeeCostBasisRequest = {
    effective_from: string;
    hourly_rate_cents?: (number | null) | undefined;
    is_salaried: boolean;
    monthly_cost_cents?: (number | null) | undefined;
    weekly_contract_minutes: number;
  };
  export type SetPresenceRequest = { status: PresenceStatus };
  export type StartRunRequest = Partial<{ trigger_payload: unknown }>;
  export type StartTimeEntryRequest = { task_id: TaskId };
  export type StartedRunResponse = { run_id: string };
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
  export type UnreadResponse = { channel_ids: Array<ChannelId> };
  export type UpdateAbsenceRequest = Partial<{
    all_day: boolean | null;
    ends_at: string | null;
    kind: null | AbsenceKind;
    note: string | null;
    starts_at: string | null;
  }>;
  export type UpdateCategoryRequest = { name: string; position: number };
  export type UpdateChannelRequest = {
    category_id?: (null | CategoryId) | undefined;
    name: string;
    position: number;
    topic?: (string | null) | undefined;
  };
  export type UpdateCredentialRequest = Partial<{ data: unknown; name: string | null }>;
  export type UpdateCustomerContactRequest = {
    email?: (string | null) | undefined;
    first_name: string;
    is_primary: boolean;
    last_name: string;
    phone?: (string | null) | undefined;
    role?: (string | null) | undefined;
  };
  export type UpdateCustomerContextRequest = {
    address_line?: (string | null) | undefined;
    city?: (string | null) | undefined;
    label: string;
    photo_key?: (string | null) | undefined;
    postal_code?: (string | null) | undefined;
  };
  export type UpdateCustomerRequest = {
    email?: (string | null) | undefined;
    name: string;
    phone?: (string | null) | undefined;
    pipeline_stage: CustomerPipelineStage;
    registration_number?: (string | null) | undefined;
    status: CustomerStatus;
  };
  export type UpdateEquipmentRequest = { hourly_rate_cents: number; name: string };
  export type UpdateInvoiceRequest = {
    customer_context_id: CustomerContextId;
    customer_id: CustomerId;
    delivery_address?: (null | DeliveryAddressRequest) | undefined;
    due_at?: (string | null) | undefined;
    lines: Array<InvoiceLineRequest>;
    notes?: (string | null) | undefined;
    operation_nature?: (null | OperationNature) | undefined;
    project_id?: (null | ProjectId) | undefined;
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
  export type UpdateOrganizationRequest = { field_clock_enabled: boolean; name: string; slug: string };
  export type UpdateProductRequest = {
    default_vat_rate_bp?: (number | null) | undefined;
    description?: (string | null) | undefined;
    name: string;
    sku?: (string | null) | undefined;
    unit: ServiceRateUnit;
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
    customer_context_id: CustomerContextId;
    customer_id: CustomerId;
    lines: Array<QuoteLineRequest>;
    status: QuoteStatus;
    title: string;
  };
  export type UpdateQuoteStatusRequest = { status: QuoteStatus };
  export type UpdateServiceRateRequest = {
    default_vat_rate_bp?: (number | null) | undefined;
    label: string;
    rate_cents: number;
    unit: ServiceRateUnit;
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
    label_ids: Array<TaskLabelId> | null;
    parent_task_id: null | TaskId;
    project_id: null | ProjectId;
    starts_at: string | null;
    status: null | TaskStatus;
    title: string | null;
  }>;
  export type UpdateThreadRequest = { archived: boolean; name: string };
  export type UpdateWebhookRequest = { avatar_url?: (string | null) | undefined; name: string };
  export type UpdateWorkflowRequest = Partial<{
    description: string | null;
    enabled: boolean | null;
    name: string | null;
  }>;
  export type UpsertEmployeeProfileRequest = {
    hourly_rate_cents?: (number | null) | undefined;
    is_salaried: boolean;
    monthly_cost_cents?: (number | null) | undefined;
    weekly_contract_minutes: number;
  };
  export type UpsertOverwriteRequest = { allow: number; deny: number };
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
  export type WorkedHoursResponse = { members: Array<WorkedHoursRow>; total_planned_minutes: number };
  export type WorkflowVersionResponse = {
    created_at: string;
    created_by?: (string | null) | undefined;
    graph: GraphDto;
    id: string;
    version: number;
    workflow_id: string;
  };
  export type WorkflowDetailResponse = {
    created_at: string;
    current_version?: (null | WorkflowVersionResponse) | undefined;
    description?: (string | null) | undefined;
    enabled: boolean;
    id: string;
    name: string;
    organization_id: OrganizationId;
    updated_at: string;
  };
  export type WorkflowResponse = {
    created_at: string;
    current_version_id?: (string | null) | undefined;
    description?: (string | null) | undefined;
    enabled: boolean;
    id: string;
    name: string;
    organization_id: OrganizationId;
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
          topic?: (string | null) | undefined;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
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
          city?: (string | null) | undefined;
          created_at: string;
          customer_id: Schemas.CustomerId;
          id: Schemas.CustomerContextId;
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
          city?: (string | null) | undefined;
          created_at: string;
          customer_id: Schemas.CustomerId;
          id: Schemas.CustomerContextId;
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
          created_at: string;
          email?: (string | null) | undefined;
          id: Schemas.CustomerId;
          name: string;
          organization_id: Schemas.OrganizationId;
          phone?: (string | null) | undefined;
          pipeline_stage: Schemas.CustomerPipelineStage;
          registration_number?: (string | null) | undefined;
          status: Schemas.CustomerStatus;
          updated_at: string;
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
          created_at: string;
          email?: (string | null) | undefined;
          id: Schemas.CustomerId;
          name: string;
          organization_id: Schemas.OrganizationId;
          phone?: (string | null) | undefined;
          pipeline_stage: Schemas.CustomerPipelineStage;
          registration_number?: (string | null) | undefined;
          status: Schemas.CustomerStatus;
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
          city?: (string | null) | undefined;
          created_at: string;
          customer_id: Schemas.CustomerId;
          id: Schemas.CustomerContextId;
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
          city?: (string | null) | undefined;
          created_at: string;
          customer_id: Schemas.CustomerId;
          id: Schemas.CustomerContextId;
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
          created_at: string;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          delivery_address?: (null | Schemas.DeliveryAddressResponse) | undefined;
          due_at?: (string | null) | undefined;
          gross_cents: number;
          id: Schemas.InvoiceId;
          issued_at?: (string | null) | undefined;
          kind: Schemas.InvoiceKind;
          lines: Array<Schemas.InvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number?: (string | null) | undefined;
          operation_nature?: (null | Schemas.OperationNature) | undefined;
          organization_id: Schemas.OrganizationId;
          project_id?: (null | Schemas.ProjectId) | undefined;
          source_invoice_id?: (null | Schemas.InvoiceId) | undefined;
          status: Schemas.InvoiceStatus;
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
          created_at: string;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          delivery_address?: (null | Schemas.DeliveryAddressResponse) | undefined;
          due_at?: (string | null) | undefined;
          gross_cents: number;
          id: Schemas.InvoiceId;
          issued_at?: (string | null) | undefined;
          kind: Schemas.InvoiceKind;
          lines: Array<Schemas.InvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number?: (string | null) | undefined;
          operation_nature?: (null | Schemas.OperationNature) | undefined;
          organization_id: Schemas.OrganizationId;
          project_id?: (null | Schemas.ProjectId) | undefined;
          source_invoice_id?: (null | Schemas.InvoiceId) | undefined;
          status: Schemas.InvoiceStatus;
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
          created_at: string;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          delivery_address?: (null | Schemas.DeliveryAddressResponse) | undefined;
          due_at?: (string | null) | undefined;
          gross_cents: number;
          id: Schemas.InvoiceId;
          issued_at?: (string | null) | undefined;
          kind: Schemas.InvoiceKind;
          lines: Array<Schemas.InvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number?: (string | null) | undefined;
          operation_nature?: (null | Schemas.OperationNature) | undefined;
          organization_id: Schemas.OrganizationId;
          project_id?: (null | Schemas.ProjectId) | undefined;
          source_invoice_id?: (null | Schemas.InvoiceId) | undefined;
          status: Schemas.InvoiceStatus;
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
          created_at: string;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          delivery_address?: (null | Schemas.DeliveryAddressResponse) | undefined;
          due_at?: (string | null) | undefined;
          gross_cents: number;
          id: Schemas.InvoiceId;
          issued_at?: (string | null) | undefined;
          kind: Schemas.InvoiceKind;
          lines: Array<Schemas.InvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number?: (string | null) | undefined;
          operation_nature?: (null | Schemas.OperationNature) | undefined;
          organization_id: Schemas.OrganizationId;
          project_id?: (null | Schemas.ProjectId) | undefined;
          source_invoice_id?: (null | Schemas.InvoiceId) | undefined;
          status: Schemas.InvoiceStatus;
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
          created_at: string;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          delivery_address?: (null | Schemas.DeliveryAddressResponse) | undefined;
          due_at?: (string | null) | undefined;
          gross_cents: number;
          id: Schemas.InvoiceId;
          issued_at?: (string | null) | undefined;
          kind: Schemas.InvoiceKind;
          lines: Array<Schemas.InvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number?: (string | null) | undefined;
          operation_nature?: (null | Schemas.OperationNature) | undefined;
          organization_id: Schemas.OrganizationId;
          project_id?: (null | Schemas.ProjectId) | undefined;
          source_invoice_id?: (null | Schemas.InvoiceId) | undefined;
          status: Schemas.InvoiceStatus;
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
          created_at: string;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          delivery_address?: (null | Schemas.DeliveryAddressResponse) | undefined;
          due_at?: (string | null) | undefined;
          gross_cents: number;
          id: Schemas.InvoiceId;
          issued_at?: (string | null) | undefined;
          kind: Schemas.InvoiceKind;
          lines: Array<Schemas.InvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number?: (string | null) | undefined;
          operation_nature?: (null | Schemas.OperationNature) | undefined;
          organization_id: Schemas.OrganizationId;
          project_id?: (null | Schemas.ProjectId) | undefined;
          source_invoice_id?: (null | Schemas.InvoiceId) | undefined;
          status: Schemas.InvoiceStatus;
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
          method: string;
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
          method: string;
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
    };
  };
  export type get_ExportInvoicePdf = {
    method: "GET";
    path: "/api/v1/invoices/{invoice_id}/pdf";
    requestFormat: "json";
    parameters: {
      path: { invoice_id: string };
    };
    responses: { 200: unknown; 401: unknown; 403: unknown; 404: unknown; 409: unknown };
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
  export type get_ListAbsences = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/absences";
    requestFormat: "json";
    parameters: {
      query: Partial<{ page: number; per_page: number }>;
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          all_day: boolean;
          created_at: string;
          ends_at: string;
          id: Schemas.AbsenceId;
          kind: Schemas.AbsenceKind;
          member_id: Schemas.MemberId;
          note?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          starts_at: string;
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
          ends_at: string;
          id: Schemas.AbsenceId;
          kind: Schemas.AbsenceKind;
          member_id: Schemas.MemberId;
          note?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          starts_at: string;
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
          ends_at: string;
          id: Schemas.AbsenceId;
          kind: Schemas.AbsenceKind;
          member_id: Schemas.MemberId;
          note?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          starts_at: string;
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
          created_at: string;
          id: string;
          kind: string;
          name: string;
          organization_id: Schemas.OrganizationId;
          origin: string;
          updated_at: string;
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
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown; 409: unknown };
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
          created_at: string;
          id: string;
          kind: string;
          name: string;
          organization_id: Schemas.OrganizationId;
          origin: string;
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
          finished_at?: (string | null) | undefined;
          id: string;
          next_attempt_at?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          started_at?: (string | null) | undefined;
          status: string;
          trigger_event_id?: (string | null) | undefined;
          trigger_payload?: unknown | undefined;
          workflow_id: string;
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
        data: Schemas.RunResponse & { steps: Array<Schemas.RunStepResponse> };
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
          finished_at?: (string | null) | undefined;
          id: string;
          next_attempt_at?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          started_at?: (string | null) | undefined;
          status: string;
          trigger_event_id?: (string | null) | undefined;
          trigger_payload?: unknown | undefined;
          workflow_id: string;
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
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          created_at: string;
          current_version_id?: (string | null) | undefined;
          description?: (string | null) | undefined;
          enabled: boolean;
          id: string;
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
          created_at: string;
          current_version_id?: (string | null) | undefined;
          description?: (string | null) | undefined;
          enabled: boolean;
          id: string;
          name: string;
          organization_id: Schemas.OrganizationId;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
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
          created_at: string;
          current_version?: (null | Schemas.WorkflowVersionResponse) | undefined;
          description?: (string | null) | undefined;
          enabled: boolean;
          id: string;
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
          created_at: string;
          current_version_id?: (string | null) | undefined;
          description?: (string | null) | undefined;
          enabled: boolean;
          id: string;
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
      422: { code: string; details: Schemas.GraphInvalidDetails; message: string; status: number };
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
        data: Array<{
          created_at: string;
          email?: (string | null) | undefined;
          id: Schemas.CustomerId;
          name: string;
          organization_id: Schemas.OrganizationId;
          phone?: (string | null) | undefined;
          pipeline_stage: Schemas.CustomerPipelineStage;
          registration_number?: (string | null) | undefined;
          status: Schemas.CustomerStatus;
          updated_at: string;
        }>;
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
          created_at: string;
          email?: (string | null) | undefined;
          id: Schemas.CustomerId;
          name: string;
          organization_id: Schemas.OrganizationId;
          phone?: (string | null) | undefined;
          pipeline_stage: Schemas.CustomerPipelineStage;
          registration_number?: (string | null) | undefined;
          status: Schemas.CustomerStatus;
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
          created_at: string;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          delivery_address?: (null | Schemas.DeliveryAddressResponse) | undefined;
          due_at?: (string | null) | undefined;
          gross_cents: number;
          id: Schemas.InvoiceId;
          issued_at?: (string | null) | undefined;
          kind: Schemas.InvoiceKind;
          lines: Array<Schemas.InvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number?: (string | null) | undefined;
          operation_nature?: (null | Schemas.OperationNature) | undefined;
          organization_id: Schemas.OrganizationId;
          project_id?: (null | Schemas.ProjectId) | undefined;
          source_invoice_id?: (null | Schemas.InvoiceId) | undefined;
          status: Schemas.InvoiceStatus;
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
          created_at: string;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          delivery_address?: (null | Schemas.DeliveryAddressResponse) | undefined;
          due_at?: (string | null) | undefined;
          gross_cents: number;
          id: Schemas.InvoiceId;
          issued_at?: (string | null) | undefined;
          kind: Schemas.InvoiceKind;
          lines: Array<Schemas.InvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number?: (string | null) | undefined;
          operation_nature?: (null | Schemas.OperationNature) | undefined;
          organization_id: Schemas.OrganizationId;
          project_id?: (null | Schemas.ProjectId) | undefined;
          source_invoice_id?: (null | Schemas.InvoiceId) | undefined;
          status: Schemas.InvoiceStatus;
          updated_at: string;
          vat_breakdown: Array<Schemas.InvoiceVatBreakdownLineResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      409: unknown;
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
          created_at: string;
          default_vat_rate_bp?: (number | null) | undefined;
          description?: (string | null) | undefined;
          id: Schemas.ProductId;
          name: string;
          organization_id: Schemas.OrganizationId;
          sku?: (string | null) | undefined;
          unit: Schemas.ServiceRateUnit;
          unit_price_cents: number;
          updated_at: string;
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
          created_at: string;
          default_vat_rate_bp?: (number | null) | undefined;
          description?: (string | null) | undefined;
          id: Schemas.ProductId;
          name: string;
          organization_id: Schemas.OrganizationId;
          sku?: (string | null) | undefined;
          unit: Schemas.ServiceRateUnit;
          unit_price_cents: number;
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
      query: Partial<{ page: number; per_page: number; customer_id: string; include_archived: boolean }>;
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
      query: Partial<{ page: number; per_page: number }>;
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          created_at: string;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          gross_cents: number;
          id: Schemas.QuoteId;
          lines: Array<Schemas.QuoteLineResponse>;
          net_cents: number;
          organization_id: Schemas.OrganizationId;
          reference?: (string | null) | undefined;
          status: Schemas.QuoteStatus;
          title: string;
          updated_at: string;
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
          created_at: string;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          gross_cents: number;
          id: Schemas.QuoteId;
          lines: Array<Schemas.QuoteLineResponse>;
          net_cents: number;
          organization_id: Schemas.OrganizationId;
          reference?: (string | null) | undefined;
          status: Schemas.QuoteStatus;
          title: string;
          updated_at: string;
          vat_breakdown: Array<Schemas.QuoteVatBreakdownLineResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      409: unknown;
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
          double_booked: Array<Schemas.ProjectProfitability>;
          incomplete: Array<Schemas.ProjectProfitability>;
          least_profitable: Array<Schemas.ProjectProfitability>;
          members: Array<Schemas.MemberProfitability>;
          most_profitable: Array<Schemas.ProjectProfitability>;
          projects: Array<Schemas.ProjectProfitability>;
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
        data: { members: Array<Schemas.WorkedHoursRow>; total_planned_minutes: number };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
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
          created_at: string;
          default_vat_rate_bp?: (number | null) | undefined;
          id: Schemas.ServiceRateId;
          label: string;
          organization_id: Schemas.OrganizationId;
          rate_cents: number;
          unit: Schemas.ServiceRateUnit;
          updated_at: string;
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
          created_at: string;
          default_vat_rate_bp?: (number | null) | undefined;
          id: Schemas.ServiceRateId;
          label: string;
          organization_id: Schemas.OrganizationId;
          rate_cents: number;
          unit: Schemas.ServiceRateUnit;
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
      query: Partial<{ page: number; per_page: number; parent_task_id: string }>;
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
          created_at: string;
          default_vat_rate_bp?: (number | null) | undefined;
          description?: (string | null) | undefined;
          id: Schemas.ProductId;
          name: string;
          organization_id: Schemas.OrganizationId;
          sku?: (string | null) | undefined;
          unit: Schemas.ServiceRateUnit;
          unit_price_cents: number;
          updated_at: string;
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
          created_at: string;
          default_vat_rate_bp?: (number | null) | undefined;
          description?: (string | null) | undefined;
          id: Schemas.ProductId;
          name: string;
          organization_id: Schemas.OrganizationId;
          sku?: (string | null) | undefined;
          unit: Schemas.ServiceRateUnit;
          unit_price_cents: number;
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
          created_at: string;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          delivery_address?: (null | Schemas.DeliveryAddressResponse) | undefined;
          due_at?: (string | null) | undefined;
          gross_cents: number;
          id: Schemas.InvoiceId;
          issued_at?: (string | null) | undefined;
          kind: Schemas.InvoiceKind;
          lines: Array<Schemas.InvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number?: (string | null) | undefined;
          operation_nature?: (null | Schemas.OperationNature) | undefined;
          organization_id: Schemas.OrganizationId;
          project_id?: (null | Schemas.ProjectId) | undefined;
          source_invoice_id?: (null | Schemas.InvoiceId) | undefined;
          status: Schemas.InvoiceStatus;
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
          created_at: string;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          delivery_address?: (null | Schemas.DeliveryAddressResponse) | undefined;
          due_at?: (string | null) | undefined;
          gross_cents: number;
          id: Schemas.InvoiceId;
          issued_at?: (string | null) | undefined;
          kind: Schemas.InvoiceKind;
          lines: Array<Schemas.InvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number?: (string | null) | undefined;
          operation_nature?: (null | Schemas.OperationNature) | undefined;
          organization_id: Schemas.OrganizationId;
          project_id?: (null | Schemas.ProjectId) | undefined;
          source_invoice_id?: (null | Schemas.InvoiceId) | undefined;
          status: Schemas.InvoiceStatus;
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
          created_at: string;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          delivery_address?: (null | Schemas.DeliveryAddressResponse) | undefined;
          due_at?: (string | null) | undefined;
          gross_cents: number;
          id: Schemas.InvoiceId;
          issued_at?: (string | null) | undefined;
          kind: Schemas.InvoiceKind;
          lines: Array<Schemas.InvoiceLineResponse>;
          net_cents: number;
          notes?: (string | null) | undefined;
          number?: (string | null) | undefined;
          operation_nature?: (null | Schemas.OperationNature) | undefined;
          organization_id: Schemas.OrganizationId;
          project_id?: (null | Schemas.ProjectId) | undefined;
          source_invoice_id?: (null | Schemas.InvoiceId) | undefined;
          status: Schemas.InvoiceStatus;
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
          created_at: string;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          gross_cents: number;
          id: Schemas.QuoteId;
          lines: Array<Schemas.QuoteLineResponse>;
          net_cents: number;
          organization_id: Schemas.OrganizationId;
          reference?: (string | null) | undefined;
          status: Schemas.QuoteStatus;
          title: string;
          updated_at: string;
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
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
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
          created_at: string;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          gross_cents: number;
          id: Schemas.QuoteId;
          lines: Array<Schemas.QuoteLineResponse>;
          net_cents: number;
          organization_id: Schemas.OrganizationId;
          reference?: (string | null) | undefined;
          status: Schemas.QuoteStatus;
          title: string;
          updated_at: string;
          vat_breakdown: Array<Schemas.QuoteVatBreakdownLineResponse>;
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
          created_at: string;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          gross_cents: number;
          id: Schemas.QuoteId;
          lines: Array<Schemas.QuoteLineResponse>;
          net_cents: number;
          organization_id: Schemas.OrganizationId;
          reference?: (string | null) | undefined;
          status: Schemas.QuoteStatus;
          title: string;
          updated_at: string;
          vat_breakdown: Array<Schemas.QuoteVatBreakdownLineResponse>;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
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
          created_at: string;
          default_vat_rate_bp?: (number | null) | undefined;
          id: Schemas.ServiceRateId;
          label: string;
          organization_id: Schemas.OrganizationId;
          rate_cents: number;
          unit: Schemas.ServiceRateUnit;
          updated_at: string;
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
          created_at: string;
          default_vat_rate_bp?: (number | null) | undefined;
          id: Schemas.ServiceRateId;
          label: string;
          organization_id: Schemas.OrganizationId;
          rate_cents: number;
          unit: Schemas.ServiceRateUnit;
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
    "/api/v1/equipment/{equipment_id}": Endpoints.patch_UpdateEquipment;
    "/api/v1/field/assignment-reports/{assignment_report_id}": Endpoints.patch_AmendAssignmentReport;
    "/api/v1/invoices/{invoice_id}": Endpoints.patch_UpdateInvoice;
    "/api/v1/members/{member_id}": Endpoints.patch_UpdateMember;
    "/api/v1/organizations/{organization_id}": Endpoints.patch_UpdateOrganization;
    "/api/v1/organizations/{organization_id}/absences/{absence_id}": Endpoints.patch_PatchAbsence;
    "/api/v1/organizations/{organization_id}/automation/credentials/{credential_id}": Endpoints.patch_UpdateAutomationCredential;
    "/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}": Endpoints.patch_UpdateWorkflow;
    "/api/v1/organizations/{organization_id}/legal-identity": Endpoints.patch_UpdateOrganizationLegalIdentity;
    "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}": Endpoints.patch_PatchProjectTemplate;
    "/api/v1/organizations/{organization_id}/projects/{project_id}": Endpoints.patch_PatchProject;
    "/api/v1/organizations/{organization_id}/task-labels/{label_id}": Endpoints.patch_UpdateTaskLabel;
    "/api/v1/organizations/{organization_id}/tasks/{task_id}": Endpoints.patch_PatchTask;
    "/api/v1/organizations/{organization_id}/tasks/{task_id}/comments/{comment_id}": Endpoints.patch_UpdateTaskComment;
    "/api/v1/products/{product_id}": Endpoints.patch_UpdateProduct;
    "/api/v1/quotes/{quote_id}": Endpoints.patch_UpdateQuote;
    "/api/v1/quotes/{quote_id}/status": Endpoints.patch_UpdateQuoteStatus;
    "/api/v1/service-rates/{service_rate_id}": Endpoints.patch_UpdateServiceRate;
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
    "/api/v1/equipment/{equipment_id}": Endpoints.delete_DeleteEquipment;
    "/api/v1/field/assignment-reports/{assignment_report_id}": Endpoints.delete_WithdrawAssignmentReport;
    "/api/v1/invitations/{invitation_id}": Endpoints.delete_RevokeInvitation;
    "/api/v1/invoice-payments/{invoice_payment_id}": Endpoints.delete_DeleteInvoicePayment;
    "/api/v1/members/{member_id}": Endpoints.delete_DeleteMember;
    "/api/v1/members/{member_id}/employee-profile": Endpoints.delete_RemoveEmployeeProfile;
    "/api/v1/organizations/{organization_id}": Endpoints.delete_DeleteOrganization;
    "/api/v1/organizations/{organization_id}/absences/{absence_id}": Endpoints.delete_DeleteAbsence;
    "/api/v1/organizations/{organization_id}/automation/credentials/{credential_id}": Endpoints.delete_DeleteAutomationCredential;
    "/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}": Endpoints.delete_DeleteWorkflow;
    "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}": Endpoints.delete_ArchiveProjectTemplate;
    "/api/v1/organizations/{organization_id}/projects/{project_id}": Endpoints.delete_ArchiveProject;
    "/api/v1/organizations/{organization_id}/task-labels/{label_id}": Endpoints.delete_DeleteTaskLabel;
    "/api/v1/organizations/{organization_id}/tasks/{task_id}": Endpoints.delete_DeleteTask;
    "/api/v1/organizations/{organization_id}/tasks/{task_id}/comments/{comment_id}": Endpoints.delete_DeleteTaskComment;
    "/api/v1/products/{product_id}": Endpoints.delete_DeleteProduct;
    "/api/v1/quotes/{quote_id}": Endpoints.delete_DeleteQuote;
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
    "/api/v1/chat/organizations/{organization_id}/notifications": Endpoints.get_ListNotifications;
    "/api/v1/chat/organizations/{organization_id}/unread": Endpoints.get_ListUnreadChannels;
    "/api/v1/customer-contacts/{customer_contact_id}": Endpoints.get_GetCustomerContact;
    "/api/v1/customer-contexts/{customer_context_id}": Endpoints.get_GetCustomerContext;
    "/api/v1/customers/{customer_id}": Endpoints.get_GetCustomer;
    "/api/v1/customers/{customer_id}/contacts": Endpoints.get_ListCustomerContacts;
    "/api/v1/customers/{customer_id}/customer-contexts": Endpoints.get_ListCustomerContexts;
    "/api/v1/employees/{employee_id}/cost-bases": Endpoints.get_ListEmployeeCostBases;
    "/api/v1/equipment/{equipment_id}": Endpoints.get_GetEquipment;
    "/api/v1/files/url": Endpoints.get_GetFileUrl;
    "/api/v1/invoices/{invoice_id}": Endpoints.get_GetInvoice;
    "/api/v1/invoices/{invoice_id}/balance": Endpoints.get_GetInvoiceBalance;
    "/api/v1/invoices/{invoice_id}/credit-notes": Endpoints.get_ListInvoiceCreditNotes;
    "/api/v1/invoices/{invoice_id}/payments": Endpoints.get_ListInvoicePayments;
    "/api/v1/invoices/{invoice_id}/pdf": Endpoints.get_ExportInvoicePdf;
    "/api/v1/members/{member_id}": Endpoints.get_GetMember;
    "/api/v1/members/{member_id}/work-time": Endpoints.get_GetWorkTime;
    "/api/v1/organizations": Endpoints.get_ListOrganizations;
    "/api/v1/organizations/{organization_id}": Endpoints.get_GetOrganization;
    "/api/v1/organizations/{organization_id}/absences": Endpoints.get_ListAbsences;
    "/api/v1/organizations/{organization_id}/assignment-reports": Endpoints.get_ListAssignmentReports;
    "/api/v1/organizations/{organization_id}/automation/connectors": Endpoints.get_ListConnectors;
    "/api/v1/organizations/{organization_id}/automation/credentials": Endpoints.get_ListAutomationCredentials;
    "/api/v1/organizations/{organization_id}/automation/events": Endpoints.get_ListAutomationEvents;
    "/api/v1/organizations/{organization_id}/automation/runs": Endpoints.get_ListRuns;
    "/api/v1/organizations/{organization_id}/automation/runs/{run_id}": Endpoints.get_GetRun;
    "/api/v1/organizations/{organization_id}/automation/settings": Endpoints.get_GetAutomationSettings;
    "/api/v1/organizations/{organization_id}/automation/workflows": Endpoints.get_ListWorkflows;
    "/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}": Endpoints.get_GetWorkflow;
    "/api/v1/organizations/{organization_id}/customers": Endpoints.get_ListCustomers;
    "/api/v1/organizations/{organization_id}/employee-profiles": Endpoints.get_ListEmployeeProfiles;
    "/api/v1/organizations/{organization_id}/equipment": Endpoints.get_ListEquipment;
    "/api/v1/organizations/{organization_id}/field/assignment-reports": Endpoints.get_ListMyAssignmentReports;
    "/api/v1/organizations/{organization_id}/field/current": Endpoints.get_GetCurrentTimeEntry;
    "/api/v1/organizations/{organization_id}/field/tasks": Endpoints.get_ListMyFieldTasks;
    "/api/v1/organizations/{organization_id}/invitations": Endpoints.get_ListInvitations;
    "/api/v1/organizations/{organization_id}/invoices": Endpoints.get_ListInvoices;
    "/api/v1/organizations/{organization_id}/invoices/outstanding": Endpoints.get_ListOutstandingBalanceByCustomer;
    "/api/v1/organizations/{organization_id}/members": Endpoints.get_ListMembers;
    "/api/v1/organizations/{organization_id}/planning": Endpoints.get_GetPlanning;
    "/api/v1/organizations/{organization_id}/planning/availability": Endpoints.get_GetPlanningAvailability;
    "/api/v1/organizations/{organization_id}/products": Endpoints.get_ListProducts;
    "/api/v1/organizations/{organization_id}/project-templates": Endpoints.get_ListProjectTemplates;
    "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}": Endpoints.get_GetProjectTemplate;
    "/api/v1/organizations/{organization_id}/projects": Endpoints.get_ListProjects;
    "/api/v1/organizations/{organization_id}/projects/{project_id}": Endpoints.get_GetProject;
    "/api/v1/organizations/{organization_id}/quotes": Endpoints.get_ListQuotes;
    "/api/v1/organizations/{organization_id}/reporting/profitability": Endpoints.get_GetProfitability;
    "/api/v1/organizations/{organization_id}/reporting/worked-hours": Endpoints.get_GetWorkedHours;
    "/api/v1/organizations/{organization_id}/service-rates": Endpoints.get_ListServiceRates;
    "/api/v1/organizations/{organization_id}/task-labels": Endpoints.get_ListTaskLabels;
    "/api/v1/organizations/{organization_id}/task-recurrences": Endpoints.get_ListTaskRecurrences;
    "/api/v1/organizations/{organization_id}/tasks": Endpoints.get_ListTasks;
    "/api/v1/organizations/{organization_id}/tasks/{task_id}": Endpoints.get_GetTask;
    "/api/v1/organizations/{organization_id}/tasks/{task_id}/comments": Endpoints.get_ListTaskComments;
    "/api/v1/products/{product_id}": Endpoints.get_GetProduct;
    "/api/v1/projects/{project_id}/billing-summary": Endpoints.get_GetProjectBillingSummary;
    "/api/v1/projects/{project_id}/invoices": Endpoints.get_ListProjectInvoices;
    "/api/v1/quotes/{quote_id}": Endpoints.get_GetQuote;
    "/api/v1/quotes/{quote_id}/pdf": Endpoints.get_ExportQuotePdf;
    "/api/v1/quotes/{quote_id}/plan-proposal": Endpoints.get_GetQuotePlanProposal;
    "/api/v1/service-rates/{service_rate_id}": Endpoints.get_GetServiceRate;
    "/api/v1/users/@me/organizations": Endpoints.get_ListMyOrganizations;
  };
  post: {
    "/api/v1/chat/channels/{channel_id}/messages": Endpoints.post_CreateMessage;
    "/api/v1/chat/channels/{channel_id}/threads": Endpoints.post_CreateThread;
    "/api/v1/chat/channels/{channel_id}/typing": Endpoints.post_StartTyping;
    "/api/v1/chat/channels/{channel_id}/webhooks": Endpoints.post_CreateWebhook;
    "/api/v1/chat/organizations/{organization_id}/categories": Endpoints.post_CreateCategory;
    "/api/v1/chat/organizations/{organization_id}/channels": Endpoints.post_CreateChannel;
    "/api/v1/chat/webhooks/{webhook_id}/{token}": Endpoints.post_ExecuteWebhook;
    "/api/v1/customers/{customer_id}/contacts": Endpoints.post_CreateCustomerContact;
    "/api/v1/customers/{customer_id}/customer-contexts": Endpoints.post_CreateCustomerContext;
    "/api/v1/employees/{employee_id}/cost-bases": Endpoints.post_SetEmployeeCostBasis;
    "/api/v1/field/time-entries/{time_entry_id}/photos": Endpoints.post_AttachTimeEntryPhoto;
    "/api/v1/field/time-entries/{time_entry_id}/recover": Endpoints.post_RecoverTimeEntry;
    "/api/v1/field/time-entries/{time_entry_id}/stop": Endpoints.post_StopTimeEntry;
    "/api/v1/files": Endpoints.post_UploadFile;
    "/api/v1/invitations/{token}/accept": Endpoints.post_AcceptInvitation;
    "/api/v1/invoices/{invoice_id}/cancel": Endpoints.post_CancelInvoice;
    "/api/v1/invoices/{invoice_id}/credit-notes": Endpoints.post_IssueCreditNote;
    "/api/v1/invoices/{invoice_id}/issue": Endpoints.post_IssueInvoice;
    "/api/v1/invoices/{invoice_id}/payments": Endpoints.post_RecordInvoicePayment;
    "/api/v1/organizations": Endpoints.post_CreateOrganization;
    "/api/v1/organizations/{organization_id}/absences": Endpoints.post_CreateAbsence;
    "/api/v1/organizations/{organization_id}/automation/credentials": Endpoints.post_CreateAutomationCredential;
    "/api/v1/organizations/{organization_id}/automation/credentials/{credential_id}/rotate": Endpoints.post_RotateAutomationCredential;
    "/api/v1/organizations/{organization_id}/automation/runs/{run_id}/replay": Endpoints.post_ReplayRun;
    "/api/v1/organizations/{organization_id}/automation/workflows": Endpoints.post_CreateWorkflow;
    "/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}/runs": Endpoints.post_StartRun;
    "/api/v1/organizations/{organization_id}/customers": Endpoints.post_CreateCustomer;
    "/api/v1/organizations/{organization_id}/equipment": Endpoints.post_CreateEquipment;
    "/api/v1/organizations/{organization_id}/field/assignments/{task_assignment_id}/report": Endpoints.post_ReportAssignment;
    "/api/v1/organizations/{organization_id}/field/day-end": Endpoints.post_EndWorkingDay;
    "/api/v1/organizations/{organization_id}/field/time-entries": Endpoints.post_StartTimeEntry;
    "/api/v1/organizations/{organization_id}/field/time-entries/declare": Endpoints.post_DeclareTimeEntry;
    "/api/v1/organizations/{organization_id}/invitations": Endpoints.post_CreateInvitation;
    "/api/v1/organizations/{organization_id}/invoices": Endpoints.post_CreateInvoice;
    "/api/v1/organizations/{organization_id}/members": Endpoints.post_CreateMember;
    "/api/v1/organizations/{organization_id}/products": Endpoints.post_CreateProduct;
    "/api/v1/organizations/{organization_id}/project-templates": Endpoints.post_CreateProjectTemplate;
    "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}/instantiate": Endpoints.post_InstantiateProjectTemplate;
    "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}/restore": Endpoints.post_RestoreProjectTemplate;
    "/api/v1/organizations/{organization_id}/projects": Endpoints.post_CreateProject;
    "/api/v1/organizations/{organization_id}/projects/{project_id}/restore": Endpoints.post_RestoreProject;
    "/api/v1/organizations/{organization_id}/quotes": Endpoints.post_CreateQuote;
    "/api/v1/organizations/{organization_id}/service-rates": Endpoints.post_CreateServiceRate;
    "/api/v1/organizations/{organization_id}/task-labels": Endpoints.post_CreateTaskLabel;
    "/api/v1/organizations/{organization_id}/task-recurrences": Endpoints.post_CreateTaskRecurrence;
    "/api/v1/organizations/{organization_id}/tasks": Endpoints.post_CreateTask;
    "/api/v1/organizations/{organization_id}/tasks/bulk-assign": Endpoints.post_BulkAssignTasks;
    "/api/v1/organizations/{organization_id}/tasks/{task_id}/comments": Endpoints.post_CreateTaskComment;
    "/api/v1/projects/{project_id}/invoices/deposit": Endpoints.post_IssueProjectDeposit;
    "/api/v1/projects/{project_id}/invoices/final": Endpoints.post_IssueProjectFinalInvoice;
    "/api/v1/quotes/{quote_id}/plan": Endpoints.post_CreateQuotePlan;
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
    "/api/v1/organizations/{organization_id}/automation/settings": Endpoints.put_UpdateAutomationSettings;
    "/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}/versions": Endpoints.put_SaveWorkflowVersion;
    "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}/tasks": Endpoints.put_ReplaceProjectTemplateTasks;
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
