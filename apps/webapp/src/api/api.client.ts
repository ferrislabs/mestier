export namespace Schemas {
  // <Schemas>
  export type AbsenceKind = "LEAVE" | "SICK" | "UNAVAILABLE";
  export type EmployeeId = string;
  export type EmployeeAbsenceId = string;
  export type OrganizationId = string;
  export type AbsenceResponse = {
    all_day: boolean;
    created_at: string;
    employee_id: EmployeeId;
    ends_at: string;
    id: EmployeeAbsenceId;
    kind: AbsenceKind;
    note?: (string | null) | undefined;
    organization_id: OrganizationId;
    starts_at: string;
    updated_at: string;
  };
  export type UserId = string;
  export type AssigneeRefRequest = { employee_id: EmployeeId; kind: "employee" } | { kind: "member"; user_id: UserId };
  export type AttachmentResponse = { filename: string; mime_type: string; size_bytes: number; storage_key: string };
  export type AuthorType = "USER" | "WEBHOOK" | "SYSTEM";
  export type WorkOrderId = string;
  export type ConflictResponse =
    | { ends_at: string; kind: "absence"; note?: (string | null) | undefined; reason: AbsenceKind; starts_at: string }
    | { ends_at: string; kind: "outside_work_hours"; starts_at: string }
    | { ends_at: string; kind: "overlapping_work_order"; starts_at: string; work_order_id: WorkOrderId };
  export type AvailabilityResourceResponse = {
    available: boolean;
    conflicts: Array<ConflictResponse>;
    resource_id: string;
  };
  export type AvailabilityResponse = { resources: Array<AvailabilityResourceResponse> };
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
  export type CreateAbsenceRequest = {
    all_day?: boolean | undefined;
    employee_id: EmployeeId;
    ends_at: string;
    kind: AbsenceKind;
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
    first_name: string;
    last_name: string;
    phone?: (string | null) | undefined;
    pipeline_stage: CustomerPipelineStage;
    status: CustomerStatus;
  };
  export type CreateEmployeeRequest = {
    hourly_rate_cents?: (number | null) | undefined;
    name: string;
    user_id?: (null | UserId) | undefined;
    weekly_contract_minutes?: number | undefined;
  };
  export type CreateEquipmentRequest = { hourly_rate_cents: number; name: string };
  export type CreateMessageAttachment = {
    filename: string;
    mime_type: string;
    size_bytes: number;
    storage_key: string;
  };
  export type CreateMessageRequest = { attachments?: Array<CreateMessageAttachment> | undefined; content: string };
  export type CreateOrganizationRequest = { name: string; slug: string };
  export type ServiceRateUnit = "HOUR" | "ML" | "M2";
  export type CreateProductRequest = {
    description?: (string | null) | undefined;
    name: string;
    sku?: (string | null) | undefined;
    unit: ServiceRateUnit;
    unit_price_cents: number;
  };
  export type CustomerContextId = string;
  export type CustomerId = string;
  export type ServiceRateId = string;
  export type QuoteLineRequest = {
    label: string;
    notes?: (string | null) | undefined;
    photo_keys: Array<string>;
    quantity: string;
    service_rate_id?: (null | ServiceRateId) | undefined;
    unit: ServiceRateUnit;
    unit_price_cents: number;
  };
  export type CreateQuoteRequest = {
    customer_context_id: CustomerContextId;
    customer_id: CustomerId;
    lines: Array<QuoteLineRequest>;
    title: string;
  };
  export type CreateServiceRateRequest = { label: string; rate_cents: number; unit: ServiceRateUnit };
  export type CreateThreadRequest = { name: string; origin_message_id?: (null | MessageId) | undefined };
  export type CreateWebhookRequest = { avatar_url?: (string | null) | undefined; name: string };
  export type QuoteId = string;
  export type CreateWorkOrderRequest = {
    all_day?: boolean | undefined;
    customer_context_id: CustomerContextId;
    customer_id: CustomerId;
    ends_at: string;
    note?: (string | null) | undefined;
    quote_id?: (null | QuoteId) | undefined;
    starts_at: string;
    title?: (string | null) | undefined;
  };
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
  export type CustomerResponse = {
    created_at: string;
    email?: (string | null) | undefined;
    first_name: string;
    id: CustomerId;
    last_name: string;
    organization_id: OrganizationId;
    phone?: (string | null) | undefined;
    pipeline_stage: CustomerPipelineStage;
    status: CustomerStatus;
    updated_at: string;
  };
  export type EmployeeResponse = {
    created_at: string;
    hourly_rate_cents?: (number | null) | undefined;
    id: EmployeeId;
    name: string;
    organization_id: OrganizationId;
    updated_at: string;
    user_id?: (null | UserId) | undefined;
    weekly_contract_minutes: number;
  };
  export type EmployeeRhythmId = string;
  export type EmployeeWorkSlotId = string;
  export type EquipmentId = string;
  export type EquipmentResponse = {
    created_at: string;
    hourly_rate_cents: number;
    id: EquipmentId;
    name: string;
    organization_id: OrganizationId;
    updated_at: string;
  };
  export type ExecuteWebhookRequest = { components?: (Array<Component> | null) | undefined; content: string };
  export type FileUploadResponse = { key: string; mime_type: string; size_bytes: number };
  export type MarkChannelReadRequest = { message_id: MessageId };
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
  export type OrganizationResponse = {
    created_at: string;
    id: OrganizationId;
    name: string;
    owner_id: UserId;
    slug: string;
    updated_at: string;
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
  export type WorkOrderStatus = "PLANNED" | "IN_PROGRESS" | "DONE" | "CANCELLED";
  export type WorkOrderResponse = {
    all_day: boolean;
    created_at: string;
    customer_context_id: CustomerContextId;
    customer_id: CustomerId;
    employee_ids: Array<EmployeeId>;
    ends_at: string;
    id: WorkOrderId;
    note?: (string | null) | undefined;
    organization_id: OrganizationId;
    quote_id?: (null | QuoteId) | undefined;
    starts_at: string;
    status: WorkOrderStatus;
    title?: (string | null) | undefined;
    updated_at: string;
  };
  export type PatchWorkOrderResponse = { created_employees: Array<EmployeeResponse>; work_order: WorkOrderResponse };
  export type PlanningEntryResponse =
    | {
        all_day: boolean;
        context_label: string;
        customer_name: string;
        employee_ids: Array<EmployeeId>;
        ends_at: string;
        id: WorkOrderId;
        kind: "work_order";
        note?: (string | null) | undefined;
        starts_at: string;
        status: WorkOrderStatus;
        title?: (string | null) | undefined;
      }
    | {
        absence_kind: AbsenceKind;
        all_day: boolean;
        employee_id: EmployeeId;
        ends_at: string;
        id: EmployeeAbsenceId;
        kind: "absence";
        note?: (string | null) | undefined;
        starts_at: string;
      };
  export type PlanningResourceKindResponse = "employee" | "member";
  export type PlanningResourceResponse = {
    display_name: string;
    employee_id?: (null | EmployeeId) | undefined;
    hourly_rate_cents?: (number | null) | undefined;
    kind: PlanningResourceKindResponse;
    resource_id: string;
    user_id?: (null | UserId) | undefined;
    weekly_contract_minutes: number;
  };
  export type PlanningWorkTimeDayResponse = { date: string; intervals: Array<MinuteIntervalResponse> };
  export type PlanningWorkTimeResponse = { days: Array<PlanningWorkTimeDayResponse>; employee_id: EmployeeId };
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
    description?: (string | null) | undefined;
    id: ProductId;
    name: string;
    organization_id: OrganizationId;
    sku?: (string | null) | undefined;
    unit: ServiceRateUnit;
    unit_price_cents: number;
    updated_at: string;
  };
  export type RhythmSlotRequest = { ends_minute: number; starts_minute: number; weekday: number };
  export type PutRhythmRequest = {
    effective_from: string;
    effective_to?: (string | null) | undefined;
    slots: Array<RhythmSlotRequest>;
  };
  export type WorkSlotRequest = { ends_minute: number; starts_minute: number; work_date: string };
  export type PutWorkSlotsRequest = { slots: Array<WorkSlotRequest> };
  export type QuoteLineId = string;
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
  };
  export type QuoteStatus = "DRAFT" | "SENT" | "ACCEPTED" | "DECLINED" | "CANCELLED";
  export type QuoteResponse = {
    created_at: string;
    customer_context_id: CustomerContextId;
    customer_id: CustomerId;
    id: QuoteId;
    lines: Array<QuoteLineResponse>;
    organization_id: OrganizationId;
    reference: string;
    status: QuoteStatus;
    title: string;
    total_cents: number;
    updated_at: string;
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
  export type ServiceRateResponse = {
    created_at: string;
    id: ServiceRateId;
    label: string;
    organization_id: OrganizationId;
    rate_cents: number;
    unit: ServiceRateUnit;
    updated_at: string;
  };
  export type SetPresenceRequest = { status: PresenceStatus };
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
    first_name: string;
    last_name: string;
    phone?: (string | null) | undefined;
    pipeline_stage: CustomerPipelineStage;
    status: CustomerStatus;
  };
  export type UpdateEmployeeRequest = {
    hourly_rate_cents?: (number | null) | undefined;
    name: string;
    user_id?: (null | UserId) | undefined;
    weekly_contract_minutes?: number | undefined;
  };
  export type UpdateEquipmentRequest = { hourly_rate_cents: number; name: string };
  export type UpdateMessageRequest = { content: string };
  export type UpdateOrganizationRequest = { name: string; slug: string };
  export type UpdateProductRequest = {
    description?: (string | null) | undefined;
    name: string;
    sku?: (string | null) | undefined;
    unit: ServiceRateUnit;
    unit_price_cents: number;
  };
  export type UpdateQuoteRequest = {
    customer_context_id: CustomerContextId;
    customer_id: CustomerId;
    lines: Array<QuoteLineRequest>;
    status: QuoteStatus;
    title: string;
  };
  export type UpdateQuoteStatusRequest = { status: QuoteStatus };
  export type UpdateServiceRateRequest = { label: string; rate_cents: number; unit: ServiceRateUnit };
  export type UpdateThreadRequest = { archived: boolean; name: string };
  export type UpdateWebhookRequest = { avatar_url?: (string | null) | undefined; name: string };
  export type UpdateWorkOrderRequest = Partial<{
    all_day: boolean | null;
    assignees: Array<AssigneeRefRequest> | null;
    ends_at: string | null;
    note: string | null;
    starts_at: string | null;
    status: null | WorkOrderStatus;
    title: string | null;
  }>;
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
  export type WorkSlotResponse = {
    employee_id: EmployeeId;
    ends_minute: number;
    id: EmployeeWorkSlotId;
    organization_id: OrganizationId;
    starts_minute: number;
    work_date: string;
  };
  export type WorkTimeResponse = { rhythms: Array<RhythmResponse>; work_slots: Array<WorkSlotResponse> };

  // </Schemas>
}

export namespace Endpoints {
  // <Endpoints>

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
          first_name: string;
          id: Schemas.CustomerId;
          last_name: string;
          organization_id: Schemas.OrganizationId;
          phone?: (string | null) | undefined;
          pipeline_stage: Schemas.CustomerPipelineStage;
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
          first_name: string;
          id: Schemas.CustomerId;
          last_name: string;
          organization_id: Schemas.OrganizationId;
          phone?: (string | null) | undefined;
          pipeline_stage: Schemas.CustomerPipelineStage;
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
  export type get_GetEmployee = {
    method: "GET";
    path: "/api/v1/employees/{employee_id}";
    requestFormat: "json";
    parameters: {
      path: { employee_id: string };
    };
    responses: {
      200: {
        data: {
          created_at: string;
          hourly_rate_cents?: (number | null) | undefined;
          id: Schemas.EmployeeId;
          name: string;
          organization_id: Schemas.OrganizationId;
          updated_at: string;
          user_id?: (null | Schemas.UserId) | undefined;
          weekly_contract_minutes: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type delete_DeleteEmployee = {
    method: "DELETE";
    path: "/api/v1/employees/{employee_id}";
    requestFormat: "json";
    parameters: {
      path: { employee_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type patch_UpdateEmployee = {
    method: "PATCH";
    path: "/api/v1/employees/{employee_id}";
    requestFormat: "json";
    parameters: {
      path: { employee_id: string };

      body: Schemas.UpdateEmployeeRequest;
    };
    responses: {
      200: {
        data: {
          created_at: string;
          hourly_rate_cents?: (number | null) | undefined;
          id: Schemas.EmployeeId;
          name: string;
          organization_id: Schemas.OrganizationId;
          updated_at: string;
          user_id?: (null | Schemas.UserId) | undefined;
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
          created_at: string;
          id: Schemas.OrganizationId;
          name: string;
          owner_id: Schemas.UserId;
          slug: string;
          updated_at: string;
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
          created_at: string;
          id: Schemas.OrganizationId;
          name: string;
          owner_id: Schemas.UserId;
          slug: string;
          updated_at: string;
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
          created_at: string;
          id: Schemas.OrganizationId;
          name: string;
          owner_id: Schemas.UserId;
          slug: string;
          updated_at: string;
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
          created_at: string;
          id: Schemas.OrganizationId;
          name: string;
          owner_id: Schemas.UserId;
          slug: string;
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
          employee_id: Schemas.EmployeeId;
          ends_at: string;
          id: Schemas.EmployeeAbsenceId;
          kind: Schemas.AbsenceKind;
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
          employee_id: Schemas.EmployeeId;
          ends_at: string;
          id: Schemas.EmployeeAbsenceId;
          kind: Schemas.AbsenceKind;
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
          employee_id: Schemas.EmployeeId;
          ends_at: string;
          id: Schemas.EmployeeAbsenceId;
          kind: Schemas.AbsenceKind;
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
          first_name: string;
          id: Schemas.CustomerId;
          last_name: string;
          organization_id: Schemas.OrganizationId;
          phone?: (string | null) | undefined;
          pipeline_stage: Schemas.CustomerPipelineStage;
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
          first_name: string;
          id: Schemas.CustomerId;
          last_name: string;
          organization_id: Schemas.OrganizationId;
          phone?: (string | null) | undefined;
          pipeline_stage: Schemas.CustomerPipelineStage;
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
  export type get_ListEmployees = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/employees";
    requestFormat: "json";
    parameters: {
      query: Partial<{ page: number; per_page: number }>;
      path: { organization_id: string };
    };
    responses: {
      200: {
        data: Array<{
          created_at: string;
          hourly_rate_cents?: (number | null) | undefined;
          id: Schemas.EmployeeId;
          name: string;
          organization_id: Schemas.OrganizationId;
          updated_at: string;
          user_id?: (null | Schemas.UserId) | undefined;
          weekly_contract_minutes: number;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateEmployee = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/employees";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateEmployeeRequest;
    };
    responses: {
      201: {
        data: {
          created_at: string;
          hourly_rate_cents?: (number | null) | undefined;
          id: Schemas.EmployeeId;
          name: string;
          organization_id: Schemas.OrganizationId;
          updated_at: string;
          user_id?: (null | Schemas.UserId) | undefined;
          weekly_contract_minutes: number;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      409: unknown;
    };
  };
  export type put_PutRhythm = {
    method: "PUT";
    path: "/api/v1/organizations/{organization_id}/employees/{employee_id}/rhythm";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; employee_id: string };

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
    path: "/api/v1/organizations/{organization_id}/employees/{employee_id}/work-slots";
    requestFormat: "json";
    parameters: {
      query: { from: string; to: string };
      path: { organization_id: string; employee_id: string };

      body: Schemas.PutWorkSlotsRequest;
    };
    responses: {
      200: {
        data: Array<{
          employee_id: Schemas.EmployeeId;
          ends_minute: number;
          id: Schemas.EmployeeWorkSlotId;
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
    path: "/api/v1/organizations/{organization_id}/employees/{employee_id}/work-time";
    requestFormat: "json";
    parameters: {
      query: { from: string; to: string };
      path: { organization_id: string; employee_id: string };
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
          id: Schemas.QuoteId;
          lines: Array<Schemas.QuoteLineResponse>;
          organization_id: Schemas.OrganizationId;
          reference: string;
          status: Schemas.QuoteStatus;
          title: string;
          total_cents: number;
          updated_at: string;
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
          id: Schemas.QuoteId;
          lines: Array<Schemas.QuoteLineResponse>;
          organization_id: Schemas.OrganizationId;
          reference: string;
          status: Schemas.QuoteStatus;
          title: string;
          total_cents: number;
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
          created_at: string;
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
  export type get_ListWorkOrders = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/work-orders";
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
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          employee_ids: Array<Schemas.EmployeeId>;
          ends_at: string;
          id: Schemas.WorkOrderId;
          note?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          starts_at: string;
          status: Schemas.WorkOrderStatus;
          title?: (string | null) | undefined;
          updated_at: string;
        }>;
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
    };
  };
  export type post_CreateWorkOrder = {
    method: "POST";
    path: "/api/v1/organizations/{organization_id}/work-orders";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string };

      body: Schemas.CreateWorkOrderRequest;
    };
    responses: {
      201: {
        data: {
          all_day: boolean;
          created_at: string;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          employee_ids: Array<Schemas.EmployeeId>;
          ends_at: string;
          id: Schemas.WorkOrderId;
          note?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          starts_at: string;
          status: Schemas.WorkOrderStatus;
          title?: (string | null) | undefined;
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
  export type get_GetWorkOrder = {
    method: "GET";
    path: "/api/v1/organizations/{organization_id}/work-orders/{work_order_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; work_order_id: string };
    };
    responses: {
      200: {
        data: {
          all_day: boolean;
          created_at: string;
          customer_context_id: Schemas.CustomerContextId;
          customer_id: Schemas.CustomerId;
          employee_ids: Array<Schemas.EmployeeId>;
          ends_at: string;
          id: Schemas.WorkOrderId;
          note?: (string | null) | undefined;
          organization_id: Schemas.OrganizationId;
          quote_id?: (null | Schemas.QuoteId) | undefined;
          starts_at: string;
          status: Schemas.WorkOrderStatus;
          title?: (string | null) | undefined;
          updated_at: string;
        };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      401: unknown;
      403: unknown;
      404: unknown;
    };
  };
  export type delete_DeleteWorkOrder = {
    method: "DELETE";
    path: "/api/v1/organizations/{organization_id}/work-orders/{work_order_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; work_order_id: string };
    };
    responses: { 204: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type patch_PatchWorkOrder = {
    method: "PATCH";
    path: "/api/v1/organizations/{organization_id}/work-orders/{work_order_id}";
    requestFormat: "json";
    parameters: {
      path: { organization_id: string; work_order_id: string };

      body: Schemas.UpdateWorkOrderRequest;
    };
    responses: {
      200: {
        data: { created_employees: Array<Schemas.EmployeeResponse>; work_order: Schemas.WorkOrderResponse };
        pagination?: (null | Schemas.PaginationMetadata) | undefined;
      };
      400: unknown;
      401: unknown;
      403: unknown;
      404: unknown;
      409: unknown;
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
          id: Schemas.QuoteId;
          lines: Array<Schemas.QuoteLineResponse>;
          organization_id: Schemas.OrganizationId;
          reference: string;
          status: Schemas.QuoteStatus;
          title: string;
          total_cents: number;
          updated_at: string;
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
          id: Schemas.QuoteId;
          lines: Array<Schemas.QuoteLineResponse>;
          organization_id: Schemas.OrganizationId;
          reference: string;
          status: Schemas.QuoteStatus;
          title: string;
          total_cents: number;
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
  export type get_ExportQuotePdf = {
    method: "GET";
    path: "/api/v1/quotes/{quote_id}/pdf";
    requestFormat: "json";
    parameters: {
      path: { quote_id: string };
    };
    responses: { 200: unknown; 401: unknown; 403: unknown; 404: unknown };
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
          id: Schemas.QuoteId;
          lines: Array<Schemas.QuoteLineResponse>;
          organization_id: Schemas.OrganizationId;
          reference: string;
          status: Schemas.QuoteStatus;
          title: string;
          total_cents: number;
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
  export type get_ListMyOrganizations = {
    method: "GET";
    path: "/api/v1/users/@me/organizations";
    requestFormat: "json";
    parameters: never;
    responses: {
      200: {
        data: Array<{
          created_at: string;
          id: Schemas.OrganizationId;
          name: string;
          owner_id: Schemas.UserId;
          slug: string;
          updated_at: string;
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
    "/api/v1/employees/{employee_id}": Endpoints.delete_DeleteEmployee;
    "/api/v1/equipment/{equipment_id}": Endpoints.delete_DeleteEquipment;
    "/api/v1/organizations/{organization_id}": Endpoints.delete_DeleteOrganization;
    "/api/v1/organizations/{organization_id}/absences/{absence_id}": Endpoints.delete_DeleteAbsence;
    "/api/v1/organizations/{organization_id}/work-orders/{work_order_id}": Endpoints.delete_DeleteWorkOrder;
    "/api/v1/products/{product_id}": Endpoints.delete_DeleteProduct;
    "/api/v1/quotes/{quote_id}": Endpoints.delete_DeleteQuote;
    "/api/v1/service-rates/{service_rate_id}": Endpoints.delete_DeleteServiceRate;
  };
  patch: {
    "/api/v1/chat/categories/{category_id}": Endpoints.patch_UpdateCategory;
    "/api/v1/chat/channels/{channel_id}": Endpoints.patch_UpdateChannel;
    "/api/v1/chat/messages/{message_id}": Endpoints.patch_UpdateMessage;
    "/api/v1/chat/threads/{channel_id}": Endpoints.patch_UpdateThread;
    "/api/v1/chat/webhooks/{webhook_id}": Endpoints.patch_UpdateWebhook;
    "/api/v1/customer-contacts/{customer_contact_id}": Endpoints.patch_UpdateCustomerContact;
    "/api/v1/customer-contexts/{customer_context_id}": Endpoints.patch_UpdateCustomerContext;
    "/api/v1/customers/{customer_id}": Endpoints.patch_UpdateCustomer;
    "/api/v1/employees/{employee_id}": Endpoints.patch_UpdateEmployee;
    "/api/v1/equipment/{equipment_id}": Endpoints.patch_UpdateEquipment;
    "/api/v1/organizations/{organization_id}": Endpoints.patch_UpdateOrganization;
    "/api/v1/organizations/{organization_id}/absences/{absence_id}": Endpoints.patch_PatchAbsence;
    "/api/v1/organizations/{organization_id}/work-orders/{work_order_id}": Endpoints.patch_PatchWorkOrder;
    "/api/v1/products/{product_id}": Endpoints.patch_UpdateProduct;
    "/api/v1/quotes/{quote_id}": Endpoints.patch_UpdateQuote;
    "/api/v1/quotes/{quote_id}/status": Endpoints.patch_UpdateQuoteStatus;
    "/api/v1/service-rates/{service_rate_id}": Endpoints.patch_UpdateServiceRate;
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
    "/api/v1/employees/{employee_id}": Endpoints.get_GetEmployee;
    "/api/v1/equipment/{equipment_id}": Endpoints.get_GetEquipment;
    "/api/v1/organizations": Endpoints.get_ListOrganizations;
    "/api/v1/organizations/{organization_id}": Endpoints.get_GetOrganization;
    "/api/v1/organizations/{organization_id}/absences": Endpoints.get_ListAbsences;
    "/api/v1/organizations/{organization_id}/customers": Endpoints.get_ListCustomers;
    "/api/v1/organizations/{organization_id}/employees": Endpoints.get_ListEmployees;
    "/api/v1/organizations/{organization_id}/employees/{employee_id}/work-time": Endpoints.get_GetWorkTime;
    "/api/v1/organizations/{organization_id}/equipment": Endpoints.get_ListEquipment;
    "/api/v1/organizations/{organization_id}/planning": Endpoints.get_GetPlanning;
    "/api/v1/organizations/{organization_id}/planning/availability": Endpoints.get_GetPlanningAvailability;
    "/api/v1/organizations/{organization_id}/products": Endpoints.get_ListProducts;
    "/api/v1/organizations/{organization_id}/quotes": Endpoints.get_ListQuotes;
    "/api/v1/organizations/{organization_id}/service-rates": Endpoints.get_ListServiceRates;
    "/api/v1/organizations/{organization_id}/work-orders": Endpoints.get_ListWorkOrders;
    "/api/v1/organizations/{organization_id}/work-orders/{work_order_id}": Endpoints.get_GetWorkOrder;
    "/api/v1/products/{product_id}": Endpoints.get_GetProduct;
    "/api/v1/quotes/{quote_id}": Endpoints.get_GetQuote;
    "/api/v1/quotes/{quote_id}/pdf": Endpoints.get_ExportQuotePdf;
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
    "/api/v1/files": Endpoints.post_UploadFile;
    "/api/v1/organizations": Endpoints.post_CreateOrganization;
    "/api/v1/organizations/{organization_id}/absences": Endpoints.post_CreateAbsence;
    "/api/v1/organizations/{organization_id}/customers": Endpoints.post_CreateCustomer;
    "/api/v1/organizations/{organization_id}/employees": Endpoints.post_CreateEmployee;
    "/api/v1/organizations/{organization_id}/equipment": Endpoints.post_CreateEquipment;
    "/api/v1/organizations/{organization_id}/products": Endpoints.post_CreateProduct;
    "/api/v1/organizations/{organization_id}/quotes": Endpoints.post_CreateQuote;
    "/api/v1/organizations/{organization_id}/service-rates": Endpoints.post_CreateServiceRate;
    "/api/v1/organizations/{organization_id}/work-orders": Endpoints.post_CreateWorkOrder;
  };
  put: {
    "/api/v1/chat/channels/{channel_id}/permissions/everyone": Endpoints.put_UpsertEveryoneOverwrite;
    "/api/v1/chat/channels/{channel_id}/permissions/{target_type}/{target_id}": Endpoints.put_UpsertTargetOverwrite;
    "/api/v1/chat/channels/{channel_id}/read": Endpoints.put_MarkChannelRead;
    "/api/v1/chat/messages/{message_id}/reactions/{emoji}": Endpoints.put_AddReaction;
    "/api/v1/chat/notifications/{notification_id}/read": Endpoints.put_MarkNotificationRead;
    "/api/v1/chat/organizations/{organization_id}/notifications/read-all": Endpoints.put_MarkAllNotificationsRead;
    "/api/v1/chat/organizations/{organization_id}/presence": Endpoints.put_SetPresence;
    "/api/v1/organizations/{organization_id}/employees/{employee_id}/rhythm": Endpoints.put_PutRhythm;
    "/api/v1/organizations/{organization_id}/employees/{employee_id}/work-slots": Endpoints.put_PutWorkSlots;
  };
};

// </EndpointByMethod>

// <EndpointByMethod.Shorthands>
export type DeleteEndpoints = EndpointByMethod["delete"];
export type PatchEndpoints = EndpointByMethod["patch"];
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
