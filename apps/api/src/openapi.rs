use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

use handlers_automation as automation;
use handlers_customer as customer;
use handlers_discord as discord;
use handlers_field as field;
use handlers_files as files;
use handlers_invoice as invoice;
use handlers_member as member;
use handlers_organization as organization;
use handlers_planning as planning;
use handlers_purchase as purchase;
use handlers_quote as quote;
use handlers_reference as reference;
use handlers_reporting as reporting;

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

#[derive(OpenApi)]
#[openapi(
    info(title = "Mestier API", description = "API for Mestier", version = "0.1.0"),
    paths(
        automation::catalogue::connectors,
        automation::catalogue::events,
        automation::credential::list::handler,
        automation::credential::create::handler,
        automation::credential::update::handler,
        automation::credential::delete::handler,
        automation::credential::rotate::handler,
        automation::workflow::list::handler,
        automation::workflow::create::handler,
        automation::workflow::get_one::handler,
        automation::workflow::update::handler,
        automation::workflow::delete::handler,
        automation::workflow::save_version::handler,
        automation::run::list::handler,
        automation::run::get_one::handler,
        automation::run::replay::handler,
        automation::run::start::handler,
        automation::settings::get,
        automation::settings::put,
        reporting::reporting::profitability::handler,
        reporting::reporting::worked_hours::handler,
        field::field::my_tasks::handler,
        field::field::current::handler,
        field::field::start::handler,
        field::field::declare::handler,
        field::field::stop::handler,
        field::field::recover::handler,
        field::field::attach_photo::handler,
        field::field::end_day::handler,
        field::field::report_assignment::handler,
        field::field::amend_report::handler,
        field::field::withdraw_report::handler,
        field::field::list_reports::handler,
        files::upload::handler,
        files::url::handler,
        customer::customer::create::handler,
        customer::customer::list::handler,
        customer::customer::get_one::handler,
        customer::customer::update::handler,
        customer::customer::soft_delete::handler,
        customer::customer_contact::create::handler,
        customer::customer_contact::list::handler,
        customer::customer_contact::get_one::handler,
        customer::customer_contact::update::handler,
        customer::customer_contact::soft_delete::handler,
        customer::customer_context::create::handler,
        customer::customer_context::list::handler,
        customer::customer_context::get_one::handler,
        customer::customer_context::update::handler,
        customer::customer_context::soft_delete::handler,
        member::member::list::handler,
        member::member::create::handler,
        member::member::get_one::handler,
        member::member::permissions::handler,
        member::member::update::handler,
        member::member::soft_delete::handler,
        member::invitation::create::handler,
        member::invitation::list::handler,
        member::invitation::revoke::handler,
        member::invitation::accept::handler,
        member::role::list::handler,
        member::role::create::handler,
        member::role::update::handler,
        member::role::delete::handler,
        member::role::list_for_member::handler,
        member::role::assign::handler,
        organization::create::handler,
        organization::list::handler,
        organization::list_mine::handler,
        organization::get_one::handler,
        organization::update::handler,
        organization::update_legal_identity::handler,
        organization::soft_delete::handler,
        quote::quote::create::handler,
        quote::quote::list::handler,
        quote::quote::get_one::handler,
        quote::quote::update::handler,
        quote::quote::update_status::handler,
        quote::quote::soft_delete::handler,
        quote::quote::export_pdf::handler,
        quote::quote::plan_proposal::handler,
        quote::quote::plan::handler,
        invoice::invoice::create::handler,
        invoice::invoice::list::handler,
        invoice::invoice::outstanding::handler,
        invoice::invoice::get_one::handler,
        invoice::invoice::update::handler,
        invoice::invoice::issue::handler,
        invoice::invoice::cancel::handler,
        invoice::invoice::credit_note::handler,
        invoice::invoice::credit_note::list_handler,
        invoice::invoice::balance::handler,
        invoice::invoice::export_pdf::handler,
        invoice::payment::record::handler,
        invoice::payment::list::handler,
        invoice::payment::delete::handler,
        invoice::project::list::handler,
        invoice::project::billing_summary::handler,
        invoice::project::deposit::handler,
        invoice::project::final_invoice::handler,
        purchase::supplier_invoice::import::handler,
        purchase::supplier_invoice::list::handler,
        purchase::supplier_invoice::get_one::handler,
        purchase::supplier_invoice::update::handler,
        purchase::supplier_invoice::confirm::handler,
        purchase::supplier_invoice::reject::handler,
        purchase::allocation::replace::handler,
        purchase::allocation::project_costs::handler,
        reference::employee::list::handler,
        reference::employee::upsert_profile::handler,
        reference::employee::remove_profile::handler,
        reference::cost_basis::list::handler,
        reference::cost_basis::create::handler,
        reference::cost_basis::update::handler,
        reference::equipment::create::handler,
        reference::equipment::list::handler,
        reference::equipment::get_one::handler,
        reference::equipment::update::handler,
        reference::equipment::soft_delete::handler,
        reference::service_rate::create::handler,
        reference::service_rate::list::handler,
        reference::service_rate::get_one::handler,
        reference::service_rate::update::handler,
        reference::service_rate::soft_delete::handler,
        reference::product::create::handler,
        reference::product::list::handler,
        reference::product::get_one::handler,
        reference::product::update::handler,
        reference::product::soft_delete::handler,
        planning::task::create::handler,
        planning::task::list::handler,
        planning::task::get_one::handler,
        planning::task::update::handler,
        planning::task::soft_delete::handler,
        planning::task::bulk_assign::handler,
        planning::task_recurrence::create::handler,
        planning::task_recurrence::list::handler,
        planning::task_recurrence::update::handler,
        planning::task_recurrence::delete::handler,
        planning::project::create::handler,
        planning::project::list::handler,
        planning::project::get_one::handler,
        planning::project::update::handler,
        planning::project::archive::handler,
        planning::project::restore::handler,
        planning::project_template::create::handler,
        planning::project_template::list::handler,
        planning::project_template::get_one::handler,
        planning::project_template::update::handler,
        planning::project_template::replace_tasks::handler,
        planning::project_template::archive::handler,
        planning::project_template::restore::handler,
        planning::project_template::instantiate::handler,
        planning::task_label::create::handler,
        planning::task_label::list::handler,
        planning::task_label::update::handler,
        planning::task_label::delete::handler,
        planning::task_comment::list::handler,
        planning::task_comment::create::handler,
        planning::task_comment::update::handler,
        planning::task_comment::delete::handler,
        planning::assignment_report::list::handler,
        planning::assignment_report::resolve::handler,
        reference::absence::create::handler,
        reference::absence::list::handler,
        reference::absence::update::handler,
        reference::absence::soft_delete::handler,
        planning::work_time::get_work_time::handler,
        planning::work_time::put_rhythm::handler,
        planning::work_time::put_work_slots::handler,
        planning::planning::get_planning::handler,
        planning::planning::get_availability::handler,
        discord::category::list::handler,
        discord::category::create::handler,
        discord::category::update::handler,
        discord::category::delete::handler,
        discord::channel::list::handler,
        discord::channel::create::handler,
        discord::channel::get_one::handler,
        discord::channel::update::handler,
        discord::channel::delete::handler,
        discord::thread::list::handler,
        discord::thread::create::handler,
        discord::thread::update::handler,
        discord::thread::delete::handler,
        discord::message::list::handler,
        discord::message::create::handler,
        discord::message::update::handler,
        discord::message::delete::handler,
        discord::reaction::add::handler,
        discord::reaction::remove::handler,
        discord::reaction::list::handler,
        discord::webhook::list::handler,
        discord::webhook::create::handler,
        discord::webhook::update::handler,
        discord::webhook::delete::handler,
        discord::webhook::execute::handler,
        discord::presence::set::handler,
        discord::typing::start::handler,
        discord::read_state::mark::handler,
        discord::read_state::unread::handler,
        discord::channel::permissions::list::handler,
        discord::channel::permissions::upsert_everyone::handler,
        discord::channel::permissions::upsert_target::handler,
        discord::channel::permissions::delete_everyone::handler,
        discord::channel::permissions::delete_target::handler,
        discord::notification::list::handler,
        discord::notification::mark_read::handler,
        discord::notification::mark_all_read::handler,
    ),
    components(schemas(
        automation::credential::create::CreateCredentialRequest,
        automation::credential::create::CredentialOriginRequest,
        automation::credential::update::UpdateCredentialRequest,
        automation::workflow::create::CreateWorkflowRequest,
        automation::workflow::update::UpdateWorkflowRequest,
        automation::workflow::save_version::SaveWorkflowVersionRequest,
        automation::workflow::save_version::GraphInvalidBody,
        automation::workflow::save_version::GraphInvalidDetails,
        automation::run::replay::ReplayRunRequest,
        automation::run::start::StartRunRequest,
        automation::run::start::StartedRunResponse,
        automation::response::CredentialResponse,
        automation::response::CredentialWithSecretResponse,
        automation::response::ConnectorsResponse,
        automation::response::ConnectorDescriptorResponse,
        automation::response::AuthSchemeResponse,
        automation::response::FieldResponse,
        automation::response::FieldKindResponse,
        automation::response::SelectOptionResponse,
        automation::response::VisibleWhenResponse,
        automation::response::AuthRequirementResponse,
        automation::response::EventDescriptorResponse,
        automation::response::WorkflowResponse,
        automation::response::WorkflowDetailResponse,
        automation::response::WorkflowVersionResponse,
        automation::response::GraphDto,
        automation::response::PlacedConnectorDto,
        automation::response::EdgeDto,
        automation::response::BranchDto,
        automation::response::GraphErrorResponse,
        automation::response::RunResponse,
        automation::response::RunDetailResponse,
        automation::response::RunStepResponse,
        automation::response::AutomationSettingsBody,
        organization::create::CreateOrganizationRequest,
        organization::update::UpdateOrganizationRequest,
        organization::update_legal_identity::UpdateLegalIdentityRequest,
        organization::update_legal_identity::VatStatusRequest,
        organization::response::OrganizationResponse,
        organization::response::VatStatusResponse,
        reporting::response::ProfitabilityResponse,
        reporting::response::ProjectProfitabilityResponse,
        reporting::response::MemberProfitabilityResponse,
        reporting::response::WorkedHoursResponse,
        reporting::response::WorkedHoursRow,
        field::response::FieldTaskResponse,
        field::response::TimeEntryResponse,
        field::response::TimeEntryPhotoResponse,
        field::response::DayLogResponse,
        field::field::start::StartTimeEntryRequest,
        field::field::declare::DeclareTimeEntryRequest,
        field::field::recover::RecoverTimeEntryRequest,
        field::field::attach_photo::AttachPhotoRequest,
        field::field::end_day::EndDayRequest,
        field::field::report_assignment::ReportAssignmentRequest,
        field::field::amend_report::AmendAssignmentReportRequest,
        field::response::AssignmentReportResponse,
        files::response::FileUploadResponse,
        files::response::FileUrlResponse,
        customer::customer::create::CreateCustomerRequest,
        customer::customer::update::UpdateCustomerRequest,
        customer::customer_contact::create::CreateCustomerContactRequest,
        customer::customer_contact::update::UpdateCustomerContactRequest,
        customer::customer_context::create::CreateCustomerContextRequest,
        customer::customer_context::update::UpdateCustomerContextRequest,
        customer::response::CustomerResponse,
        customer::response::CustomerContactResponse,
        customer::response::CustomerContextResponse,
        quote::quote::create::CreateQuoteRequest,
        quote::quote::create::QuoteLineRequest,
        quote::quote::update::UpdateQuoteRequest,
        quote::quote::update_status::UpdateQuoteStatusRequest,
        quote::quote::plan::CreateQuotePlanRequest,
        quote::quote::plan::PlannedTaskRequest,
        quote::response::QuoteLineResponse,
        quote::response::QuoteVatBreakdownLineResponse,
        quote::response::QuoteResponse,
        quote::response::TaskProposalResponse,
        quote::response::QuotePlanProposalResponse,
        quote::response::PlannedProjectResponse,
        quote::response::PlannedTaskResponse,
        quote::response::QuotePlanResponse,
        invoice::invoice::create::CreateInvoiceRequest,
        invoice::invoice::create::InvoiceLineRequest,
        invoice::invoice::create::DeliveryAddressRequest,
        invoice::invoice::update::UpdateInvoiceRequest,
        invoice::invoice::issue::IssueInvoiceRequest,
        invoice::invoice::credit_note::IssueCreditNoteRequest,
        invoice::payment::record::RecordInvoicePaymentRequest,
        invoice::project::deposit::IssueDepositRequest,
        invoice::project::final_invoice::IssueFinalInvoiceRequest,
        invoice::response::InvoiceLineResponse,
        invoice::response::InvoiceVatBreakdownLineResponse,
        invoice::response::InvoiceResponse,
        invoice::response::DeliveryAddressResponse,
        invoice::response::InvoicePaymentResponse,
        invoice::response::InvoiceBalanceResponse,
        invoice::response::CustomerOutstandingBalanceResponse,
        invoice::response::ProjectBillingSummaryResponse,
        purchase::supplier_invoice::update::UpdateSupplierInvoiceRequest,
        purchase::supplier_invoice::confirm::ConfirmSupplierInvoiceRequest,
        purchase::supplier_invoice::reject::RejectSupplierInvoiceRequest,
        purchase::allocation::replace::LineAllocationShareRequest,
        purchase::allocation::replace::ReplaceLineAllocationsRequest,
        purchase::response::SupplierInvoiceLineResponse,
        purchase::response::SupplierInvoiceVatBreakdownLineResponse,
        purchase::response::SupplierInvoiceResponse,
        purchase::response::TotalsMismatchResponse,
        purchase::response::ImportSupplierInvoiceResponse,
        purchase::response::SupplierInvoiceLineAllocationResponse,
        purchase::response::ProjectSupplierCostLineResponse,
        purchase::response::ProjectSupplierCostsResponse,
        member::member::create::CreateMemberRequest,
        member::member::update::UpdateMemberRequest,
        member::response::MemberResponse,
        member::response::MemberAccountResponse,
        member::response::MyPermissionsResponse,
        member::invitation::create::CreateInvitationRequest,
        member::response::InvitationResponse,
        member::response::CreatedInvitationResponse,
        member::role::create::CreateRoleRequest,
        member::role::update::UpdateRoleRequest,
        member::role::assign::AssignRoleRequest,
        member::response::RoleResponse,
        member::response::MemberRoleIdsResponse,
        reference::employee::upsert_profile::UpsertEmployeeProfileRequest,
        reference::cost_basis::create::SetEmployeeCostBasisRequest,
        reference::cost_basis::update::CorrectEmployeeCostBasisRequest,
        reference::cost_basis::EmployeeCostBasisResponse,
        reference::equipment::create::CreateEquipmentRequest,
        reference::equipment::update::UpdateEquipmentRequest,
        reference::service_rate::create::CreateServiceRateRequest,
        reference::service_rate::update::UpdateServiceRateRequest,
        reference::product::create::CreateProductRequest,
        reference::product::update::UpdateProductRequest,
        planning::task::create::CreateTaskRequest,
        planning::task::update::UpdateTaskRequest,
        planning::task::soft_delete::DeleteScopeRequest,
        planning::task::bulk_assign::BulkAssignTasksRequest,
        planning::task_recurrence::create::RecurrenceRuleRequest,
        planning::task_recurrence::create::CreateTaskRecurrenceRequest,
        planning::task_recurrence::update::UpdateTaskRecurrenceRequest,
        planning::response::RecurrenceRuleResponse,
        planning::response::TaskRecurrenceResponse,
        planning::project::create::CreateProjectRequest,
        planning::project::update::UpdateProjectRequest,
        planning::project_template::create::CreateProjectTemplateRequest,
        planning::project_template::create::ProjectTemplateTaskShapeRequest,
        planning::project_template::update::UpdateProjectTemplateRequest,
        planning::project_template::replace_tasks::ReplaceProjectTemplateTasksRequest,
        planning::project_template::instantiate::InstantiateProjectTemplateRequest,
        planning::response::ProjectTemplateResponse,
        planning::response::ProjectTemplateTaskResponse,
        planning::response::InstantiateProjectTemplateResponse,
        planning::response::ProjectResponse,
        planning::response::TaskResponse,
        planning::response::TaskAssignmentSummary,
        planning::response::BulkAssignTasksResponse,
        planning::task_label::create::CreateTaskLabelRequest,
        planning::task_label::update::UpdateTaskLabelRequest,
        planning::task_label::TaskLabelResponse,
        planning::task_comment::create::CreateTaskCommentRequest,
        planning::task_comment::update::UpdateTaskCommentRequest,
        planning::task_comment::TaskCommentResponse,
        planning::assignment_report::resolve::ResolveAssignmentReportRequest,
        planning::assignment_report::AssignmentReportResponse,
        planning::response::PatchTaskResponse,
        reference::absence::create::CreateAbsenceRequest,
        reference::absence::update::UpdateAbsenceRequest,
        reference::absence::AbsenceResponse,
        planning::work_time::RhythmResponse,
        planning::work_time::RhythmSlotResponse,
        planning::work_time::WorkSlotResponse,
        planning::work_time::WorkTimeResponse,
        planning::work_time::put_rhythm::PutRhythmRequest,
        planning::work_time::put_rhythm::RhythmSlotRequest,
        planning::work_time::put_work_slots::PutWorkSlotsRequest,
        planning::work_time::put_work_slots::WorkSlotRequest,
        planning::planning::get_planning::PlanningResourceResponse,
        planning::planning::get_planning::PlanningEntryResponse,
        planning::planning::get_planning::MinuteIntervalResponse,
        planning::planning::get_planning::PlanningWorkTimeDayResponse,
        planning::planning::get_planning::PlanningWorkTimeResponse,
        planning::planning::get_planning::PlanningResponse,
        planning::planning::get_availability::ConflictResponse,
        planning::planning::get_availability::AvailabilityResourceResponse,
        planning::planning::get_availability::AvailabilityResponse,
        // `planning::response::EmployeeResponse` is deliberately absent: it is
        // byte-identical to `reference::response::EmployeeResponse` above, and
        // utoipa names schemas after the type, so registering both would
        // collide. See the PR description — if the two ever diverge, this
        // document starts describing the wrong shape silently.
        reference::response::EmployeeResponse,
        reference::response::EquipmentResponse,
        reference::response::ServiceRateResponse,
        reference::response::ProductResponse,
        discord::category::create::CreateCategoryRequest,
        discord::category::update::UpdateCategoryRequest,
        discord::channel::create::CreateChannelRequest,
        discord::channel::update::UpdateChannelRequest,
        discord::thread::create::CreateThreadRequest,
        discord::thread::update::UpdateThreadRequest,
        discord::message::create::CreateMessageRequest,
        discord::message::create::CreateMessageAttachment,
        discord::message::update::UpdateMessageRequest,
        discord::webhook::create::CreateWebhookRequest,
        discord::webhook::update::UpdateWebhookRequest,
        discord::webhook::execute::ExecuteWebhookRequest,
        discord::presence::set::SetPresenceRequest,
        discord::read_state::mark::MarkChannelReadRequest,
        discord::response::CategoryResponse,
        discord::response::ChannelResponse,
        discord::response::MessageResponse,
        discord::response::AttachmentResponse,
        discord::response::ReactionCountResponse,
        discord::response::WebhookResponse,
        discord::response::WebhookCreatedResponse,
        discord::response::PresenceResponse,
        discord::read_state::unread::UnreadResponse,
        discord::response::OverwriteResponse,
        discord::response::UpsertOverwriteRequest,
        discord::notification::list::NotificationResponse,
    )),
    modifiers(&SecurityAddon),
    tags(
        (name = "automation", description = "Workflow engine — connector and event catalogues, credentials, workflows, runs, settings"),
        (name = "organizations", description = "Organizations management"),
        (name = "reporting", description = "Profitability and worked hours"),
        (name = "field", description = "Field app: clocking on and off, day end"),
        (name = "files", description = "File uploads and storage"),
        (name = "customers", description = "Customers and customer contexts management"),
        (name = "quotes", description = "Quotes and quote lines management"),
        (name = "invoices", description = "Invoices, credit notes and payments"),
        (name = "purchasing", description = "Supplier invoices — import, review, and cost allocation to projects"),
        (name = "members", description = "Organization seats — free or occupied, no account required"),
        (name = "reference", description = "Reference cost catalog"),
        (name = "planning", description = "Team planning — tasks, assignments, working time and availability"),
        (name = "discord", description = "Chat — categories, channels, threads, messages, reactions, webhooks, presence, typing, read states, channel permission overwrites, notifications"),
    )
)]
pub struct ApiDoc;

#[cfg(test)]
mod tests {
    use utoipa::OpenApi;

    use super::ApiDoc;

    /// The `paths(...)` list is hand-maintained, so a route can ship and never
    /// reach the document — and the webapp's client is generated from the
    /// document, not from the router. This catches the omission here rather
    /// than as a missing function three commits later.
    #[test]
    fn every_project_route_reaches_the_document() {
        let document = ApiDoc::openapi();
        let paths = &document.paths.paths;

        let collection = "/api/v1/organizations/{organization_id}/projects";
        let item = "/api/v1/organizations/{organization_id}/projects/{project_id}";
        let restore = "/api/v1/organizations/{organization_id}/projects/{project_id}/restore";

        for path in [collection, item, restore] {
            assert!(
                paths.contains_key(path),
                "{path} is missing from the document"
            );
        }

        // Serialized rather than walked: `PathItem`'s operation map is not part
        // of utoipa's stable surface, and the JSON is what the client generator
        // reads anyway.
        let json = serde_json::to_string(&document).expect("the document serializes");
        for operation_id in [
            "createProject",
            "listProjects",
            "getProject",
            "patchProject",
            "archiveProject",
            "restoreProject",
        ] {
            assert!(
                json.contains(&format!("\"operationId\":\"{operation_id}\"")),
                "{operation_id} is missing from the document"
            );
        }
    }

    /// Same discipline as `every_project_route_reaches_the_document`, for the
    /// template stack #296 adds.
    #[test]
    fn every_project_template_route_reaches_the_document() {
        let document = ApiDoc::openapi();
        let paths = &document.paths.paths;

        let collection = "/api/v1/organizations/{organization_id}/project-templates";
        let item =
            "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}";
        let tasks =
            "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}/tasks";
        let restore = "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}/restore";
        let instantiate = "/api/v1/organizations/{organization_id}/project-templates/{project_template_id}/instantiate";

        for path in [collection, item, tasks, restore, instantiate] {
            assert!(
                paths.contains_key(path),
                "{path} is missing from the document"
            );
        }

        let json = serde_json::to_string(&document).expect("the document serializes");
        for operation_id in [
            "createProjectTemplate",
            "listProjectTemplates",
            "getProjectTemplate",
            "patchProjectTemplate",
            "replaceProjectTemplateTasks",
            "archiveProjectTemplate",
            "restoreProjectTemplate",
            "instantiateProjectTemplate",
        ] {
            assert!(
                json.contains(&format!("\"operationId\":\"{operation_id}\"")),
                "{operation_id} is missing from the document"
            );
        }
    }

    /// The task routes are how a task joins a project, so the field has to be
    /// in the generated schema or the front cannot send it.
    #[test]
    fn the_task_patch_schema_carries_the_project_and_the_expenses() {
        let document = ApiDoc::openapi();
        let schemas = &document
            .components
            .as_ref()
            .expect("the document has components")
            .schemas;

        let schema = serde_json::to_string(
            schemas
                .get("UpdateTaskRequest")
                .expect("UpdateTaskRequest is registered"),
        )
        .expect("the schema serializes");

        for field in ["project_id", "expenses_cents", "expenses_label"] {
            assert!(schema.contains(field), "{field} is missing from {schema}");
        }
    }

    /// Same discipline as `every_project_template_route_reaches_the_document`,
    /// for the quote-handover stack #298 adds.
    #[test]
    fn every_quote_plan_route_reaches_the_document() {
        let document = ApiDoc::openapi();
        let paths = &document.paths.paths;

        let proposal = "/api/v1/quotes/{quote_id}/plan-proposal";
        let plan = "/api/v1/quotes/{quote_id}/plan";

        for path in [proposal, plan] {
            assert!(
                paths.contains_key(path),
                "{path} is missing from the document"
            );
        }

        let json = serde_json::to_string(&document).expect("the document serializes");
        for operation_id in ["getQuotePlanProposal", "createQuotePlan"] {
            assert!(
                json.contains(&format!("\"operationId\":\"{operation_id}\"")),
                "{operation_id} is missing from the document"
            );
        }
    }
}
