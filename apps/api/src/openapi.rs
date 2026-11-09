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
use handlers_support as support;

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
        components.add_security_scheme(
            "intake_key",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .description(Some(
                        "An organization API key, created with POST /organizations/{id}/support/api-keys and sent as `Authorization: Bearer <key>`. It names the organization: no route of the intake takes an organization id.",
                    ))
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
        automation::catalogue::credential_kinds_handler,
        automation::credential::list::handler,
        automation::credential::create::handler,
        automation::credential::update::handler,
        automation::credential::delete::handler,
        automation::credential::rotate::handler,
        automation::credential::oauth::start,
        automation::credential::oauth::disconnect,
        automation::credential::oauth::callback,
        automation::collection::list::handler,
        automation::collection::create::handler,
        automation::collection::get_one::handler,
        automation::collection::replace::handler,
        automation::collection::delete::handler,
        automation::folder::list::handler,
        automation::folder::create::handler,
        automation::folder::update::handler,
        automation::folder::delete::handler,
        automation::workflow::list::handler,
        automation::workflow::create::handler,
        automation::workflow::get_one::handler,
        automation::workflow::update::handler,
        automation::workflow::delete::handler,
        automation::workflow::save_version::handler,
        automation::workflow::dry_run::handler,
        automation::webhook::receive,
        automation::run::list::handler,
        automation::run::get_one::handler,
        automation::run::replay::handler,
        automation::run::start::handler,
        automation::settings::get,
        automation::settings::put,
        automation::expression::evaluate,
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
        field::field::propose_intervention::handler,
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
        customer::deal::create::handler,
        customer::deal::list::handler,
        customer::deal::summary::handler,
        customer::deal::get_one::handler,
        customer::deal::update::handler,
        customer::deal::delete::handler,
        customer::deal::move_stage::handler,
        member::member::list::handler,
        member::member::create::handler,
        member::member::get_one::handler,
        member::member::permissions::handler,
        member::member::me::handler,
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
        member::role::unassign::handler,
        organization::create::handler,
        organization::list::handler,
        organization::list_mine::handler,
        organization::get_one::handler,
        organization::update::handler,
        organization::update_legal_identity::handler,
        organization::soft_delete::handler,
        organization::billing_settings::get_handler,
        organization::billing_settings::put_handler,
        organization::numbering::get_handler,
        organization::numbering::put_handler,
        quote::quote::create::handler,
        quote::quote::list::handler,
        quote::quote::get_one::handler,
        quote::quote::update::handler,
        quote::quote::update_status::handler,
        quote::quote::revise::handler,
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
        invoice::invoice::delete::handler,
        invoice::invoice::send::handler,
        invoice::invoice::remind::handler,
        invoice::invoice::note::handler,
        invoice::quote::create_invoice::handler,
        invoice::quote::invoicing::handler,
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
        invoice::treasury::expense::list_handler,
        invoice::treasury::expense::create_handler,
        invoice::treasury::expense::get_handler,
        invoice::treasury::expense::update_handler,
        invoice::treasury::expense::delete_handler,
        invoice::treasury::expense::pay_handler,
        invoice::treasury::expense::reimburse_handler,
        invoice::treasury::category::list_handler,
        invoice::treasury::category::create_handler,
        invoice::treasury::category::update_handler,
        invoice::treasury::category::delete_handler,
        invoice::treasury::ledger::read_handler,
        invoice::treasury::ledger::record_handler,
        invoice::treasury::ledger::allocate_handler,
        purchase::supplier::list::handler,
        purchase::supplier::create::handler,
        purchase::supplier::get_one::handler,
        purchase::supplier::update::handler,
        purchase::supplier::archive::archive_handler,
        purchase::supplier::archive::restore_handler,
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
        reference::member_profile::list_profiles,
        reference::member_profile::get_profile,
        reference::member_profile::put_profile,
        reference::member_profile::archive,
        reference::member_profile::restore,
        reference::member_contract::list_organization_contracts,
        reference::member_contract::list_member_contracts,
        reference::member_contract::create_contract,
        reference::member_contract::update_contract,
        reference::member_contract::delete_contract,
        reference::member_document::list_documents,
        reference::member_document::add_document,
        reference::member_document::delete_document,
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
        reference::custom_unit::create::handler,
        reference::custom_unit::list::handler,
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
        planning::project::channel::create::handler,
        planning::project::channel::get_one::handler,
        planning::maintenance_contract::list::handler,
        planning::maintenance_contract::create::handler,
        planning::maintenance_contract::get_one::handler,
        planning::maintenance_contract::update::handler,
        planning::maintenance_contract::archive::handler,
        planning::maintenance_contract::duplicate::handler,
        planning::doe::list::handler,
        planning::doe::get_project::handler,
        planning::doe::create::handler,
        planning::doe::get_one::handler,
        planning::doe::update::handler,
        planning::doe::draft_supply::handler,
        planning::doe::pdf::handler,
        planning::doe::docx::handler,
        planning::doe::delete::handler,
        planning::photo_library::upload::handler,
        planning::photo_library::list::handler,
        planning::photo_library::get_one::handler,
        planning::photo_library::thumbnail::handler,
        planning::photo_library::update::handler,
        planning::photo_library::delete::handler,
        planning::photo_library::place::handler,
        planning::doe_model::create::handler,
        planning::doe_model::list::handler,
        planning::doe_model::get_one::handler,
        planning::doe_model::update::handler,
        planning::doe_model::archive::handler,
        planning::doe_model::restore::handler,
        planning::doe_model::from_starter::handler,
        planning::doe_model::starters::handler,
        planning::doe_model::settings::get_handler,
        planning::doe_model::settings::put_handler,
        planning::doe_model::preview::handler,
        planning::doe_model::preview::pages_handler,
        planning::recap_pdf::handler,
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
        planning::maintenance_intervention::record::handler,
        planning::maintenance_intervention::list::handler,
        planning::maintenance_intervention::validate::handler,
        planning::maintenance_intervention::reject::handler,
        planning::maintenance_overview::grid::handler,
        planning::maintenance_overview::recap::handler,
        planning::maintenance_overview::plan::handler,
        planning::assignment_report::list::handler,
        planning::assignment_report::resolve::handler,
        reference::absence::create::handler,
        reference::absence::list::handler,
        reference::absence::update::handler,
        reference::absence::soft_delete::handler,
        reference::absence::approve::handler,
        reference::absence::reject::handler,
        reference::absence::cancel::handler,
        reference::absence_type::list_types,
        reference::absence_type::create_type,
        reference::absence_type::update_type,
        reference::absence_type::archive_type,
        reference::absence_type::restore_type,
        reference::leave_account::list_accounts,
        reference::leave_account::create_account,
        reference::leave_account::update_account,
        reference::leave_account::archive_account,
        reference::leave_account::restore_account,
        reference::leave_adjustment::list_adjustments,
        reference::leave_adjustment::create_adjustment,
        reference::leave_adjustment::delete_adjustment,
        reference::surcharge::list_surcharges,
        reference::surcharge::create_surcharge,
        reference::surcharge::update_surcharge,
        reference::surcharge::archive_surcharge,
        reference::surcharge::restore_surcharge,
        reference::extra_period::list_extra_periods,
        reference::extra_period::create_extra_period,
        reference::extra_period::approve_extra_period,
        reference::extra_period::reject_extra_period,
        reference::extra_period::cancel_extra_period,
        reference::day_override::list_day_overrides,
        reference::day_override::put_day_override,
        reference::day_override::delete_day_override,
        reference::planning_feed::planning_feed,
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
        discord::channel::reorder::handler,
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
        discord::directory::list::handler,
        discord::directory::roles::handler,
        discord::direct_channel::list::handler,
        discord::direct_channel::open::handler,
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
        discord::notification::activity::handler,
        discord::notification::level::set_handler,
        discord::notification::level::list_handler,
        discord::notification::mark_read::handler,
        discord::notification::mark_all_read::handler,
        support::ticket::list::handler,
        support::ticket::get_one::handler,
        support::ticket::transition::handler,
        support::ticket::assign::handler,
        support::ticket::add_badge::handler,
        support::ticket::remove_badge::handler,
        support::thread::list::handler,
        support::thread::add_entry::handler,
        support::thread::edit_draft::handler,
        support::thread::delete_draft::handler,
        support::thread::send_draft::handler,
        support::thread::attach::handler,
        support::thread::attachment_url::handler,
        support::settings::get::handler,
        support::settings::replace::handler,
        support::api_key::create::handler,
        support::api_key::list::handler,
        support::api_key::revoke::handler,
        support::intake::create_ticket::handler,
        support::intake::list_tickets::handler,
        support::intake::get_ticket::handler,
        support::intake::post_message::handler,
        support::intake::attach_file::handler,
        support::intake::attachment_url::handler,
    ),
    components(schemas(
        support::response::TicketResponse,
        support::response::BadgeResponse,
        support::ticket::transition::TransitionTicketRequest,
        support::ticket::assign::AssignTicketRequest,
        support::ticket::add_badge::AddTicketBadgeRequest,
        support::response::TicketEntryResponse,
        support::response::AttachmentResponse,
        support::response::AttachmentUrlResponse,
        support::response::SupportSettingsResponse,
        support::settings::replace::ReplaceSupportSettingsRequest,
        support::thread::add_entry::AddTicketEntryRequest,
        support::thread::edit_draft::EditSupportDraftRequest,
        support::response::SupportApiKeyResponse,
        support::response::CreatedSupportApiKeyResponse,
        support::api_key::create::CreateSupportApiKeyRequest,
        support::intake::types::CreateIntakeTicketRequest,
        support::intake::types::PostIntakeMessageRequest,
        support::intake::types::IntakeTicketResponse,
        support::intake::types::IntakeEntryResponse,
        support::intake::types::IntakeEntryKind,
        support::intake::types::IntakeAttachmentResponse,
        support::intake::types::IntakeAttachmentUrlResponse,
        automation::credential::create::CreateCredentialRequest,
        automation::credential::create::CredentialOriginRequest,
        automation::credential::update::UpdateCredentialRequest,
        automation::credential::oauth::StartOauthRequest,
        automation::credential::oauth::StartOauthResponse,
        automation::credential::oauth::CallbackQuery,
        automation::collection::CollectionRequest,
        automation::collection::CollectionResponse,
        automation::collection::CollectionSummaryResponse,
        automation::collection::HistoryEntryResponse,
        automation::collection::CollectionSourceDto,
        automation::collection::OperationDto,
        automation::collection::ParameterDto,
        automation::collection::ParameterLocationDto,
        automation::collection::HttpMethodDto,
        automation::collection::RequestBodyDto,
        automation::collection::SecuritySchemeDto,
        automation::collection::SecurityKindDto,
        automation::collection::ApiKeyLocationDto,
        automation::folder::create::CreateFolderRequest,
        automation::folder::update::UpdateFolderRequest,
        automation::workflow::create::CreateWorkflowRequest,
        automation::workflow::update::UpdateWorkflowRequest,
        automation::workflow::save_version::SaveWorkflowVersionRequest,
        automation::workflow::save_version::GraphInvalidBody,
        automation::workflow::save_version::GraphInvalidDetails,
        automation::workflow::dry_run::DryRunRequest,
        automation::workflow::dry_run::DryRunResponse,
        automation::workflow::dry_run::DryRunStepResponse,
        automation::workflow::dry_run::DryRunStepStatus,
        automation::run::replay::ReplayRunRequest,
        automation::run::start::StartRunRequest,
        automation::run::start::StartedRunResponse,
        automation::response::CredentialResponse,
        automation::response::CredentialWithSecretResponse,
        automation::credential::dto::CredentialStatusResponse,
        automation::credential::dto::CredentialKindResponse,
        automation::credential::dto::CredentialUsageResponse,
        automation::credential::dto::CredentialFieldResponse,
        automation::credential::dto::CredentialOptionResponse,
        automation::credential::dto::CredentialShowWhenResponse,
        automation::response::FolderResponse,
        automation::response::ConnectorsResponse,
        automation::response::ConnectorDescriptorResponse,
        automation::response::AuthSchemeResponse,
        automation::response::FieldResponse,
        automation::response::FieldKindResponse,
        automation::response::SelectOptionResponse,
        automation::response::VisibleWhenResponse,
        automation::response::AuthRequirementResponse,
        automation::response::ExecutionModeResponse,
        automation::response::EventDescriptorResponse,
        automation::response::WorkflowResponse,
        automation::response::WorkflowDetailResponse,
        automation::response::WorkflowVersionResponse,
        automation::response::GraphDto,
        automation::response::PlacedConnectorDto,
        automation::response::EdgeDto,
        automation::response::BranchDto,
        automation::response::PlacedTriggerDto,
        automation::response::TriggerKindDto,
        automation::webhook::WebhookAccepted,
        automation::webhook::WebhookBodyInvalid,
        automation::response::WebhookMethodDto,
        automation::response::ContractKeyDto,
        automation::response::ContractKindDto,
        automation::response::OnInvalidDto,
        automation::response::FrequencyDto,
        automation::response::WeekdayDto,
        automation::response::GraphErrorResponse,
        automation::response::RunResponse,
        automation::response::RunDetailResponse,
        automation::response::RunStepResponse,
        automation::response::AutomationSettingsBody,
        automation::expression::EvaluateExpressionRequest,
        automation::expression::EvaluateContextBody,
        automation::expression::LoopFrameBody,
        automation::expression::EvaluateExpressionResponse,
        organization::create::CreateOrganizationRequest,
        organization::update::UpdateOrganizationRequest,
        organization::update_legal_identity::UpdateLegalIdentityRequest,
        organization::update_legal_identity::VatStatusRequest,
        organization::response::OrganizationResponse,
        organization::response::VatStatusResponse,
        organization::billing_settings::BillingSettingsResponse,
        organization::billing_settings::ReplaceBillingSettingsRequest,
        organization::numbering::NumberSeriesKind,
        organization::numbering::SeriesNumberingResponse,
        organization::numbering::SetNextNumberRequest,
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
        field::field::propose_intervention::ProposeInterventionRequest,
        field::response::FieldInterventionResponse,
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
        customer::response::CustomerListItemResponse,
        customer::response::CustomerResponse,
        customer::response::PrimaryContactResponse,
        customer::response::CustomerContactResponse,
        customer::response::CustomerContextResponse,
        customer::deal::create::CreateDealRequest,
        customer::deal::update::UpdateDealRequest,
        customer::deal::move_stage::MoveDealRequest,
        customer::response::DealResponse,
        customer::response::DealSummaryResponse,
        customer::response::StageTotalResponse,
        quote::quote::create::CreateQuoteRequest,
        quote::quote::create::QuoteLineRequest,
        quote::quote::update::UpdateQuoteRequest,
        quote::quote::update_status::UpdateQuoteStatusRequest,
        quote::quote::plan::CreateQuotePlanRequest,
        quote::quote::plan::PlannedTaskRequest,
        quote::response::QuoteLineResponse,
        quote::response::QuoteVatBreakdownLineResponse,
        quote::response::RecurringTotalResponse,
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
        invoice::invoice::credit_note::CreditLineBody,
        invoice::invoice::note::NoteInvoiceRequest,
        invoice::quote::create_invoice::CreateInvoiceFromQuoteRequest,
        invoice::quote::create_invoice::QuoteInvoiceKind,
        invoice::response::InvoiceEventResponse,
        invoice::response::QuoteInvoicingResponse,
        invoice::response::QuoteInvoicingBlockerResponse,
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
        invoice::invoice::export_pdf::InvoiceDocumentFormatQuery,
        invoice::treasury::expense::CreateExpenseRequest,
        invoice::treasury::expense::UpdateExpenseRequest,
        invoice::treasury::expense::PayExpenseRequest,
        invoice::treasury::category::ExpenseCategoryRequest,
        invoice::treasury::ledger::RecordReceiptRequest,
        invoice::treasury::ledger::AllocateReceiptRequest,
        invoice::treasury::response::ExpenseResponse,
        invoice::treasury::response::ExpenseCategoryResponse,
        invoice::treasury::response::ReceiptResponse,
        invoice::treasury::response::LedgerResponse,
        invoice::treasury::response::LedgerEntryResponse,
        invoice::treasury::response::LedgerSummaryResponse,
        invoice::treasury::response::MonthTrendResponse,
        invoice::treasury::response::VatSummaryResponse,
        invoice::treasury::response::PayablesResponse,
        purchase::supplier::create::CreateSupplierRequest,
        purchase::supplier::update::UpdateSupplierRequest,
        purchase::supplier::response::SupplierResponse,
        purchase::supplier::profile::SupplierContactRequest,
        purchase::supplier::profile::SupplierAddressRequest,
        purchase::supplier::profile::SupplierContactResponse,
        purchase::supplier::profile::SupplierAddressResponse,
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
        member::response::MyMemberResponse,
        member::invitation::create::CreateInvitationRequest,
        member::response::InvitationResponse,
        member::response::CreatedInvitationResponse,
        member::role::create::CreateRoleRequest,
        member::role::update::UpdateRoleRequest,
        member::role::assign::AssignRoleRequest,
        member::response::RoleResponse,
        member::response::MemberRoleIdsResponse,
        reference::employee::upsert_profile::UpsertEmployeeProfileRequest,
        reference::member_profile::PutMemberProfileRequest,
        reference::member_profile::ArchiveMemberRequest,
        reference::member_profile::PostalAddressBody,
        reference::member_profile::MemberProfileResponse,
        reference::member_profile::MemberProfileAddressResponse,
        reference::member_contract::PutMemberContractRequest,
        reference::member_contract::MemberContractResponse,
        reference::member_document::AddMemberDocumentRequest,
        reference::member_document::MemberDocumentResponse,
        reference::ContractType,
        reference::DocumentKind,
        reference::ScheduleInput,
        reference::WorkDayInput,
        reference::LunchInput,
        reference::BlockInput,
        reference::ScheduleUnit,
        reference::cost_basis::create::SetEmployeeCostBasisRequest,
        reference::cost_basis::update::CorrectEmployeeCostBasisRequest,
        reference::cost_basis::EmployeeCostBasisResponse,
        reference::equipment::create::CreateEquipmentRequest,
        reference::equipment::update::UpdateEquipmentRequest,
        reference::service_rate::create::CreateServiceRateRequest,
        reference::service_rate::update::UpdateServiceRateRequest,
        reference::product::create::CreateProductRequest,
        reference::product::update::UpdateProductRequest,
        reference::custom_unit::create::CreateCustomUnitRequest,
        reference::response::CustomUnitResponse,
        reference::ServiceRateUnit,
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
        planning::project::channel::create::CreateProjectChannelRequest,
        planning::response::ProjectChannelResponse,
        planning::maintenance_contract::create::CreateMaintenanceContractRequest,
        planning::maintenance_contract::request::ContractBody,
        planning::maintenance_contract::request::SeasonRequest,
        planning::maintenance_contract::request::CadenceRequest,
        planning::maintenance_contract::request::WindowRequest,
        planning::maintenance_contract::request::MaintenanceLineRequest,
        planning::maintenance_contract::response::MaintenanceContractResponse,
        planning::doe::response::DoeResponse,
        planning::doe::response::DoeSummaryResponse,
        planning::doe::create::CreateDoeRequest,
        planning::doe::update::UpdateDoeRequest,
        planning::doe::draft_supply::DraftSupplyRequest,
        planning::photo_library::response::LibraryPhotoResponse,
        planning::photo_library::response::PlacementReportResponse,
        planning::photo_library::response::PlacementResultResponse,
        planning::photo_library::response::PlacementStatusResponse,
        planning::photo_library::update::UpdateLibraryPhotoRequest,
        planning::photo_library::place::PlacePhotosRequest,
        planning::photo_library::place::PlacementItem,
        planning::photo_library::place::PlacementTarget,
        planning::doe_model::create::CreateDoeModelRequest,
        planning::doe_model::update::UpdateDoeModelRequest,
        planning::doe_model::from_starter::CreateDoeModelFromStarterRequest,
        planning::doe_model::settings::UpdateDoeSettingsRequest,
        planning::doe_model::preview::PreviewDoeStyleRequest,
        planning::doe_model::preview::DoeStylePreviewPagesResponse,
        planning::doe_model::response::DoeModelResponse,
        planning::doe_model::response::DoeModelStarterResponse,
        planning::doe_model::response::DoeSettingsResponse,
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
        planning::maintenance_intervention::InterventionResponse,
        planning::maintenance_intervention::record::RecordInterventionRequest,
        planning::maintenance_intervention::reject::RejectInterventionRequest,
        planning::maintenance_intervention::list::StatusFilter,
        planning::maintenance_overview::plan::PlanMaintenanceLinesRequest,
        planning::maintenance_overview::schemas::Cadence,
        planning::maintenance_overview::schemas::ContractSite,
        planning::maintenance_overview::schemas::ContractStatus,
        planning::maintenance_overview::schemas::Due,
        planning::maintenance_overview::schemas::DueStatus,
        planning::maintenance_overview::schemas::Grid,
        planning::maintenance_overview::schemas::GridContract,
        planning::maintenance_overview::schemas::GridLine,
        planning::maintenance_overview::schemas::GridVisit,
        planning::maintenance_overview::schemas::IntervalDays,
        planning::maintenance_overview::schemas::InterventionStatus,
        planning::maintenance_overview::schemas::IsoWeekNumber,
        planning::maintenance_overview::schemas::Recap,
        planning::maintenance_overview::schemas::RecapGroup,
        planning::maintenance_overview::schemas::RecapLine,
        planning::maintenance_overview::schemas::Season,
        planning::maintenance_overview::schemas::SiteContact,
        planning::maintenance_overview::schemas::Window,
        planning::maintenance_overview::schemas::Windows,
        planning::assignment_report::resolve::ResolveAssignmentReportRequest,
        planning::assignment_report::AssignmentReportResponse,
        planning::response::PatchTaskResponse,
        reference::absence::create::CreateAbsenceRequest,
        reference::absence::update::UpdateAbsenceRequest,
        reference::absence::AbsenceResponse,
        reference::absence::reject::RejectAbsenceRequest,
        reference::absence_type::AbsenceTypeRequest,
        reference::absence_type::AbsenceTypeResponse,
        reference::leave_account::AcquisitionBody,
        reference::leave_account::LeaveAccountRequest,
        reference::leave_account::LeaveAccountResponse,
        reference::leave_adjustment::CreateLeaveAdjustmentRequest,
        reference::leave_adjustment::LeaveAdjustmentResponse,
        reference::surcharge::SurchargeRequest,
        reference::surcharge::SurchargeResponse,
        reference::extra_period::CreateExtraPeriodRequest,
        reference::extra_period::RejectExtraPeriodRequest,
        reference::extra_period::ExtraPeriodResponse,
        reference::day_override::PutDayOverrideRequest,
        reference::day_override::DayOverrideResponse,
        reference::planning_feed::FeedMemberResponse,
        reference::planning_feed::HrPlanningFeedResponse,
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
        discord::channel::reorder::ReorderChannelsRequest,
        discord::channel::reorder::ChannelPlacementRequest,
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
        discord::response::DirectoryEntryResponse,
        discord::response::ChatRoleResponse,
        discord::response::DirectChannelResponse,
        discord::direct_channel::open::OpenDirectChannelRequest,
        discord::read_state::unread::UnreadChannelResponse,
        discord::response::PresenceResponse,
        discord::read_state::unread::UnreadResponse,
        discord::response::OverwriteResponse,
        discord::response::UpsertOverwriteRequest,
        discord::notification::list::NotificationResponse,
        discord::notification::activity::ActivityEntryResponse,
        discord::notification::level::SetNotificationLevelRequest,
        discord::notification::level::NotificationLevelResponse,
    )),
    modifiers(&SecurityAddon),
    tags(
        (name = "automation", description = "Workflow engine — connector and event catalogues, credentials, workflows, runs, settings"),
        (name = "organizations", description = "Organizations management"),
        (name = "reporting", description = "Profitability and worked hours"),
        (name = "field", description = "Field app: clocking on and off, day end"),
        (name = "files", description = "File uploads and storage"),
        (name = "customers", description = "Customers and customer contexts management"),
        (name = "deals", description = "Sales opportunities of customers and the pipeline summary"),
        (name = "quotes", description = "Quotes and quote lines management"),
        (name = "invoices", description = "Invoices, credit notes and payments"),
        (name = "treasury", description = "Expenses, expense categories, unallocated receipts and the treasury journal"),
        (name = "purchasing", description = "Supplier invoices — import, review, and cost allocation to projects"),
        (name = "members", description = "Organization seats — free or occupied, no account required"),
        (name = "reference", description = "Reference cost catalog"),
        (name = "planning", description = "Team planning — tasks, assignments, working time and availability"),
        (name = "support", description = "Support tickets — kanban, thread, intake by organization API key"),
        (name = "discord", description = "Chat — categories, channels, threads, messages, reactions, webhooks, presence, typing, read states, channel permission overwrites, notifications"),
    )
)]
pub struct ApiDoc;

#[cfg(test)]
mod tests {
    use serde_json::json;
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

    #[test]
    fn graph_nodes_carry_their_operation_reference_in_the_document() {
        let document = serde_json::to_value(ApiDoc::openapi()).expect("the document serializes");
        let schemas = &document["components"]["schemas"];

        let node = &schemas["PlacedConnectorDto"]["properties"];
        assert!(node["operation"].is_object(), "{node}");
        let reference = &schemas["OperationRefDto"];
        let mut required: Vec<&str> = reference["required"]
            .as_array()
            .expect("the reference has required properties")
            .iter()
            .map(|name| name.as_str().unwrap())
            .collect();
        required.sort_unstable();
        assert_eq!(required, ["collection_id", "operation_id"]);
        let mut properties: Vec<&str> = reference["properties"]
            .as_object()
            .expect("the reference has properties")
            .keys()
            .map(String::as_str)
            .collect();
        properties.sort_unstable();
        assert_eq!(properties, ["collection_id", "operation_id"]);
    }

    #[test]
    fn trigger_kinds_describe_the_webhook_and_the_schedule_in_the_document() {
        let document = serde_json::to_value(ApiDoc::openapi()).expect("the document serializes");
        let schemas = &document["components"]["schemas"];

        let kind = serde_json::to_string(&schemas["TriggerKindDto"]).unwrap();
        for variant in ["Manual", "Events", "Webhook", "Schedule"] {
            assert!(
                kind.contains(&format!("\"{variant}\"")),
                "{variant}: {kind}"
            );
        }

        let webhook = &schemas["TriggerKindDto"]["oneOf"]
            .as_array()
            .expect("a tagged enum is a oneOf")
            .iter()
            .find_map(|variant| variant["properties"].get("Webhook"))
            .expect("the webhook variant is described")["properties"];
        for field in ["method", "path", "credential_id", "contract", "on_invalid"] {
            assert!(webhook[field].is_object(), "webhook.{field}: {webhook}");
        }
        let schedule = &schemas["TriggerKindDto"]["oneOf"]
            .as_array()
            .unwrap()
            .iter()
            .find_map(|variant| variant["properties"].get("Schedule"))
            .expect("the schedule variant is described")["properties"];
        for field in ["frequency", "time", "weekday", "timezone"] {
            assert!(schedule[field].is_object(), "schedule.{field}: {schedule}");
        }

        let methods: Vec<&str> = schemas["WebhookMethodDto"]["enum"]
            .as_array()
            .expect("the methods are an enum")
            .iter()
            .map(|method| method.as_str().unwrap())
            .collect();
        assert_eq!(methods, ["GET", "POST", "PUT", "PATCH", "DELETE"]);
        for schema in [
            "ContractKeyDto",
            "ContractKindDto",
            "OnInvalidDto",
            "FrequencyDto",
            "WeekdayDto",
        ] {
            assert!(schemas[schema].is_object(), "{schema} is missing");
        }
        let required: Vec<&str> = schemas["ContractKeyDto"]["required"]
            .as_array()
            .unwrap()
            .iter()
            .map(|name| name.as_str().unwrap())
            .collect();
        assert!(required.contains(&"name") && required.contains(&"kind"));
        assert!(!required.contains(&"example"));
    }

    #[test]
    fn the_public_webhook_route_reaches_the_document_outside_the_authenticated_prefix() {
        let document = serde_json::to_value(ApiDoc::openapi()).expect("the document serializes");

        let operation = &document["paths"]["/webhooks/{path}"]["post"];

        assert_eq!(operation["operationId"], "receiveWebhook");
        assert!(
            operation.get("security").is_none()
                || operation["security"] == serde_json::json!([])
                || operation["security"] == serde_json::json!([{}]),
            "{operation}"
        );
        for status in ["202", "400", "401", "404", "408", "413", "415", "429"] {
            assert!(operation["responses"][status].is_object(), "{status}");
        }
        let schemas = &document["components"]["schemas"];
        assert!(schemas["WebhookAccepted"]["properties"]["run_id"].is_object());
        assert!(schemas["WebhookBodyInvalid"]["properties"]["keys"].is_object());
    }

    #[test]
    fn every_collection_route_reaches_the_document() {
        let document = ApiDoc::openapi();
        let paths = &document.paths.paths;
        let collections = "/api/v1/organizations/{organization_id}/automation/collections";
        let collection =
            "/api/v1/organizations/{organization_id}/automation/collections/{collection_id}";

        for path in [collections, collection] {
            assert!(
                paths.contains_key(path),
                "{path} is missing from the document"
            );
        }

        let json = serde_json::to_value(&document).expect("the document serializes");
        for (path, method, operation_id) in [
            (collections, "get", "listCollections"),
            (collections, "post", "importCollection"),
            (collection, "get", "getCollection"),
            (collection, "put", "replaceCollection"),
            (collection, "delete", "deleteCollection"),
        ] {
            assert_eq!(
                json["paths"][path][method]["operationId"], operation_id,
                "{method} {path}"
            );
        }

        let schemas = &json["components"]["schemas"];
        assert!(
            schemas["CollectionSummaryResponse"]["properties"]["description"].is_null(),
            "the list stays light"
        );
        for field in ["version", "imported_at", "source", "base_url", "security"] {
            assert!(
                schemas["HistoryEntryResponse"]["properties"][field].is_object(),
                "{field} is missing from the history entry"
            );
        }

        for schema in [
            "CollectionRequest",
            "CollectionResponse",
            "CollectionSummaryResponse",
            "HistoryEntryResponse",
            "OperationDto",
            "ParameterDto",
            "RequestBodyDto",
            "SecuritySchemeDto",
            "SecurityKindDto",
            "CollectionSourceDto",
        ] {
            assert!(
                json["components"]["schemas"][schema].is_object(),
                "{schema} is missing from the components"
            );
        }
    }

    /// Same discipline as `every_project_route_reaches_the_document`, for the
    /// template stack #296 adds.
    #[test]
    fn every_support_route_reaches_the_document() {
        let document = ApiDoc::openapi();
        let paths = &document.paths.paths;

        let root = "/api/v1/organizations/{organization_id}/support";
        for path in [
            format!("{root}/tickets"),
            format!("{root}/tickets/{{ticket_id}}"),
            format!("{root}/tickets/{{ticket_id}}/status"),
            format!("{root}/tickets/{{ticket_id}}/assignee"),
            format!("{root}/tickets/{{ticket_id}}/badges"),
            format!("{root}/tickets/{{ticket_id}}/badges/{{label}}"),
            format!("{root}/tickets/{{ticket_id}}/entries"),
            format!("{root}/entries/{{entry_id}}"),
            format!("{root}/entries/{{entry_id}}/send"),
            format!("{root}/entries/{{entry_id}}/attachments"),
            format!("{root}/attachments/{{attachment_id}}/url"),
            format!("{root}/settings"),
            format!("{root}/api-keys"),
            format!("{root}/api-keys/{{key_id}}"),
            "/api/v1/support/intake/tickets".to_owned(),
            "/api/v1/support/intake/tickets/{ticket_id}".to_owned(),
            "/api/v1/support/intake/tickets/{ticket_id}/messages".to_owned(),
            "/api/v1/support/intake/tickets/{ticket_id}/attachments".to_owned(),
            "/api/v1/support/intake/tickets/{ticket_id}/attachments/{attachment_id}/url".to_owned(),
        ] {
            assert!(paths.contains_key(&path), "{path}");
        }

        let json = serde_json::to_string(&document).unwrap();
        for operation_id in [
            "listSupportTickets",
            "getSupportTicket",
            "transitionSupportTicket",
            "assignSupportTicket",
            "addSupportTicketBadge",
            "removeSupportTicketBadge",
            "listSupportTicketEntries",
            "addSupportTicketEntry",
            "editSupportDraft",
            "deleteSupportDraft",
            "sendSupportDraft",
            "attachSupportEntryFile",
            "getSupportAttachmentUrl",
            "getSupportSettings",
            "replaceSupportSettings",
            "createSupportApiKey",
            "listSupportApiKeys",
            "revokeSupportApiKey",
            "createIntakeTicket",
            "listIntakeTickets",
            "getIntakeTicket",
            "postIntakeMessage",
            "attachIntakeFile",
            "getIntakeAttachmentUrl",
        ] {
            assert!(
                json.contains(&format!("\"operationId\":\"{operation_id}\"")),
                "{operation_id}"
            );
        }
    }

    #[test]
    fn every_support_intake_route_reaches_the_document_under_the_intake_key_scheme() {
        let document = ApiDoc::openapi();
        let json = serde_json::to_value(&document).unwrap();
        let root = "/api/v1/support/intake";

        for (path, method, operation_id) in [
            (format!("{root}/tickets"), "post", "createIntakeTicket"),
            (format!("{root}/tickets"), "get", "listIntakeTickets"),
            (
                format!("{root}/tickets/{{ticket_id}}"),
                "get",
                "getIntakeTicket",
            ),
            (
                format!("{root}/tickets/{{ticket_id}}/messages"),
                "post",
                "postIntakeMessage",
            ),
            (
                format!("{root}/tickets/{{ticket_id}}/attachments"),
                "post",
                "attachIntakeFile",
            ),
            (
                format!("{root}/tickets/{{ticket_id}}/attachments/{{attachment_id}}/url"),
                "get",
                "getIntakeAttachmentUrl",
            ),
        ] {
            let operation = &json["paths"][&path][method];
            assert_eq!(operation["operationId"], operation_id, "{method} {path}");
            assert_eq!(
                operation["security"],
                serde_json::json!([{ "intake_key": [] }]),
                "{operation_id}"
            );
        }

        let scheme = &json["components"]["securitySchemes"]["intake_key"];
        assert_eq!(scheme["type"], "http");
        assert_eq!(scheme["scheme"], "bearer");
        assert!(
            scheme["description"]
                .as_str()
                .unwrap()
                .contains("POST /organizations/{id}/support/api-keys")
        );
        assert_eq!(
            json["components"]["securitySchemes"]["bearer_auth"]["bearerFormat"],
            "JWT"
        );
        assert!(
            json["paths"][format!("{root}/tickets")]["get"]["description"]
                .as_str()
                .unwrap()
                .contains(
                    "custom_field_key and custom_field_value must be given together; the value of the top-level key is compared as text"
                )
        );
    }

    #[test]
    fn the_support_intake_schemas_never_expose_a_member_or_a_storage_key() {
        let document = ApiDoc::openapi();
        let schemas = &document
            .components
            .as_ref()
            .expect("the document has components")
            .schemas;

        for name in [
            "CreateIntakeTicketRequest",
            "PostIntakeMessageRequest",
            "IntakeTicketResponse",
            "IntakeEntryResponse",
            "IntakeAttachmentResponse",
            "IntakeAttachmentUrlResponse",
        ] {
            let schema = serde_json::to_value(
                schemas
                    .get(name)
                    .unwrap_or_else(|| panic!("{name} is registered")),
            )
            .unwrap();
            let properties = schema["properties"]
                .as_object()
                .unwrap_or_else(|| panic!("{name} has properties"));

            for forbidden in [
                "author_member_id",
                "assignee_member_id",
                "assigned_at",
                "file_key",
            ] {
                assert!(!properties.contains_key(forbidden), "{name}.{forbidden}");
            }
        }
    }

    #[test]
    fn the_support_intake_schemas_have_the_contracted_properties() {
        let document = ApiDoc::openapi();
        let schemas = &document
            .components
            .as_ref()
            .expect("the document has components")
            .schemas;
        let properties = |name: &str| -> Vec<String> {
            let schema = serde_json::to_value(
                schemas
                    .get(name)
                    .unwrap_or_else(|| panic!("{name} is registered")),
            )
            .unwrap();
            let mut keys: Vec<String> = schema["properties"]
                .as_object()
                .unwrap_or_else(|| panic!("{name} has properties"))
                .keys()
                .cloned()
                .collect();
            keys.sort_unstable();

            keys
        };

        assert_eq!(
            properties("CreateIntakeTicketRequest"),
            ["body", "custom_fields", "external_id", "subject"]
        );
        assert_eq!(properties("PostIntakeMessageRequest"), ["body"]);
        assert_eq!(
            properties("IntakeTicketResponse"),
            [
                "badges",
                "body",
                "created_at",
                "custom_fields",
                "entries",
                "external_id",
                "id",
                "number",
                "status",
                "subject"
            ]
        );
        assert_eq!(
            properties("IntakeEntryResponse"),
            ["attachments", "body", "created_at", "id", "kind"]
        );
        assert_eq!(
            properties("IntakeAttachmentResponse"),
            ["created_at", "file_name", "id", "mime_type", "size_bytes"]
        );
        assert_eq!(
            properties("IntakeAttachmentUrlResponse"),
            ["expires_at", "url"]
        );
    }

    #[test]
    fn the_intake_entry_kind_lists_only_what_a_client_may_see() {
        let document = ApiDoc::openapi();
        let json = serde_json::to_value(&document).unwrap();
        let schemas = &json["components"]["schemas"];

        assert_eq!(
            schemas["IntakeEntryResponse"]["properties"]["kind"]["$ref"],
            "#/components/schemas/IntakeEntryKind"
        );
        assert_eq!(
            schemas["IntakeEntryKind"]["enum"],
            serde_json::json!(["client_message", "reply"])
        );
        assert!(schemas["IntakeTicketResponse"]["properties"]["updated_at"].is_null());
        assert!(
            !schemas["IntakeTicketResponse"]["required"]
                .as_array()
                .unwrap()
                .iter()
                .any(|name| name == "updated_at")
        );
        let text = serde_json::to_string(&schemas["IntakeEntryKind"]).unwrap();
        assert!(!text.contains("internal_note"));
        assert!(!text.contains("draft"));
    }

    #[test]
    fn the_support_api_key_schemas_expose_the_secret_only_at_creation() {
        let document = ApiDoc::openapi();
        let schemas = &document
            .components
            .as_ref()
            .expect("the document has components")
            .schemas;
        let properties = |name: &str| -> Vec<String> {
            let schema = serde_json::to_value(
                schemas
                    .get(name)
                    .unwrap_or_else(|| panic!("{name} is registered")),
            )
            .expect("the schema serializes");
            schema["properties"]
                .as_object()
                .unwrap_or_else(|| panic!("{name} has properties"))
                .keys()
                .cloned()
                .collect()
        };

        let created = properties("CreatedSupportApiKeyResponse");
        let listed = properties("SupportApiKeyResponse");
        let request = properties("CreateSupportApiKeyRequest");

        for field in ["id", "name", "prefix", "key", "created_at"] {
            assert!(created.iter().any(|name| name == field), "{field}");
        }
        for field in [
            "id",
            "name",
            "prefix",
            "created_at",
            "last_used_at",
            "revoked_at",
        ] {
            assert!(listed.iter().any(|name| name == field), "{field}");
        }
        for field in ["key", "secret", "secret_hash", "plaintext", "hash"] {
            assert!(!listed.iter().any(|name| name == field), "{field}");
        }
        assert_eq!(request, ["name"]);
    }

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

    #[test]
    fn the_quote_lifecycle_routes_and_schemas_reach_the_document() {
        let document = ApiDoc::openapi();
        let json = serde_json::to_string(&document).expect("the document serializes");

        assert!(
            document
                .paths
                .paths
                .contains_key("/api/v1/quotes/{quote_id}/revision")
        );
        for name in [
            "\"operationId\":\"reviseQuote\"",
            "\"RecurringTotalResponse\"",
            "\"BillingFrequency\"",
            "\"IssuerSnapshot\"",
            "\"RecipientSnapshot\"",
            "\"SUPERSEDED\"",
        ] {
            assert!(json.contains(name), "{name} is missing from the document");
        }
    }

    /// The visit loop, the grid and the recap: the client is generated from
    /// this document, and the grid and recap bodies are only reachable through
    /// the paths that return them.
    #[test]
    fn the_maintenance_visit_loop_and_its_views_reach_the_document() {
        let document = ApiDoc::openapi();
        let json = serde_json::to_string(&document).expect("the document serializes");

        for operation_id in [
            "recordMaintenanceIntervention",
            "listMaintenanceInterventions",
            "validateMaintenanceIntervention",
            "rejectMaintenanceIntervention",
            "getMaintenanceGrid",
            "getMaintenanceRecap",
            "planMaintenanceLines",
            "proposeMaintenanceIntervention",
        ] {
            assert!(
                json.contains(&format!("\"operationId\":\"{operation_id}\"")),
                "{operation_id} is missing from the document"
            );
        }

        let schemas = &document
            .components
            .as_ref()
            .expect("the document has components")
            .schemas;
        for schema in ["Grid", "Recap", "Due", "DueStatus", "InterventionResponse"] {
            assert!(schemas.contains_key(schema), "{schema} is not registered");
        }

        let mut dangling: Vec<&str> = json
            .split("#/components/schemas/")
            .skip(1)
            .filter_map(|rest| rest.split('"').next())
            .filter(|name| !schemas.contains_key(*name))
            .collect();
        dangling.sort_unstable();
        dangling.dedup();
        assert!(
            dangling.is_empty(),
            "referenced but never registered: {dangling:?}"
        );
    }

    #[test]
    fn every_automation_folder_route_reaches_the_document() {
        let document = ApiDoc::openapi();
        let paths = &document.paths.paths;

        let collection = "/api/v1/organizations/{organization_id}/automation/folders";
        let item = "/api/v1/organizations/{organization_id}/automation/folders/{folder_id}";
        for path in [collection, item] {
            assert!(
                paths.contains_key(path),
                "{path} is missing from the document"
            );
        }

        let json = serde_json::to_string(&document).expect("the document serializes");
        for operation_id in [
            "listWorkflowFolders",
            "createWorkflowFolder",
            "updateWorkflowFolder",
            "deleteWorkflowFolder",
        ] {
            assert!(
                json.contains(&format!("\"operationId\":\"{operation_id}\"")),
                "{operation_id} is missing from the document"
            );
        }

        let schemas = &document
            .components
            .as_ref()
            .expect("the document has components")
            .schemas;
        for schema in [
            "FolderResponse",
            "CreateFolderRequest",
            "UpdateFolderRequest",
        ] {
            assert!(schemas.contains_key(schema), "{schema} is not registered");
        }
    }

    #[test]
    fn workflows_carry_their_folder_tags_and_archive_state_in_the_document() {
        let document = ApiDoc::openapi();
        let schemas = &document
            .components
            .as_ref()
            .expect("the document has components")
            .schemas;

        for (schema, fields) in [
            ("WorkflowResponse", ["folder_id", "tags", "archived_at"]),
            (
                "WorkflowDetailResponse",
                ["folder_id", "tags", "archived_at"],
            ),
            ("CreateWorkflowRequest", ["enabled", "folder_id", "tags"]),
            ("UpdateWorkflowRequest", ["folder_id", "tags", "archived"]),
        ] {
            let json = serde_json::to_value(schemas.get(schema).expect(schema)).unwrap();
            for field in fields {
                assert!(
                    json["properties"].get(field).is_some(),
                    "{schema} lacks {field}: {json}"
                );
            }
        }
    }

    #[test]
    fn listing_workflows_documents_the_archived_filter() {
        let document = ApiDoc::openapi();
        let json = serde_json::to_value(&document).expect("the document serializes");

        let parameters = json["paths"]["/api/v1/organizations/{organization_id}/automation/workflows"]
            ["get"]["parameters"]
            .as_array()
            .expect("the list operation has parameters");

        assert!(
            parameters
                .iter()
                .any(|parameter| parameter["name"] == "archived" && parameter["in"] == "query"),
            "{parameters:?}"
        );
    }

    #[test]
    fn the_dry_run_route_and_its_shapes_reach_the_document() {
        let document = ApiDoc::openapi();
        let path = "/api/v1/organizations/{organization_id}/automation/workflows/dry-run";

        let item = document
            .paths
            .paths
            .get(path)
            .unwrap_or_else(|| panic!("{path} is missing from the document"));
        let json = serde_json::to_value(item).expect("the path item serializes");
        assert_eq!(json["post"]["operationId"], "dryRunWorkflow");
        assert!(json["post"]["responses"].get("200").is_some());
        assert!(json["post"]["responses"].get("422").is_some());

        let schemas = &document
            .components
            .as_ref()
            .expect("the document has components")
            .schemas;
        for schema in [
            "DryRunRequest",
            "DryRunResponse",
            "DryRunStepResponse",
            "DryRunStepStatus",
        ] {
            assert!(schemas.contains_key(schema), "{schema} is not registered");
        }
        let request = serde_json::to_value(schemas.get("DryRunRequest").unwrap()).unwrap();
        let required = request["required"].as_array().expect("required fields");
        assert_eq!(required, &[json!("graph")]);
        for field in ["graph", "trigger_id", "trigger_payload"] {
            assert!(request["properties"].get(field).is_some(), "{field}");
        }
        let step = serde_json::to_value(schemas.get("DryRunStepResponse").unwrap()).unwrap();
        for field in [
            "connector_id",
            "iteration_path",
            "status",
            "input",
            "output",
            "error",
        ] {
            assert!(step["properties"].get(field).is_some(), "{field}");
        }
        let required = step["required"].as_array().expect("required fields");
        assert!(required.iter().any(|name| name == "iteration_path"));
        let status = serde_json::to_value(schemas.get("DryRunStepStatus").unwrap()).unwrap();
        assert_eq!(status["enum"], json!(["succeeded", "failed", "skipped"]));
    }

    #[test]
    fn graph_nodes_carry_their_name_description_and_disabled_flag_in_the_document() {
        let document = ApiDoc::openapi();
        let schemas = &document
            .components
            .as_ref()
            .expect("the document has components")
            .schemas;

        for (schema, fields) in [
            (
                "PlacedConnectorDto",
                vec!["name", "description", "disabled"],
            ),
            ("PlacedTriggerDto", vec!["name", "description"]),
        ] {
            let json = serde_json::to_value(schemas.get(schema).expect(schema)).unwrap();
            for field in &fields {
                assert!(
                    json["properties"].get(*field).is_some(),
                    "{schema} lacks {field}: {json}"
                );
            }
            let required = json["required"].as_array().cloned().unwrap_or_default();
            for field in &fields {
                assert!(
                    !required.iter().any(|name| name == field),
                    "{schema}.{field} must stay optional for clients written before it existed"
                );
            }
        }
    }

    #[test]
    fn runs_carry_their_number_progress_and_trigger_fields_in_the_document() {
        let document = ApiDoc::openapi();
        let schemas = &document
            .components
            .as_ref()
            .expect("the document has components")
            .schemas;
        let fields = [
            "number",
            "steps_done",
            "steps_total",
            "trigger_id",
            "trigger_event_name",
            "failed_connector_id",
            "workflow_version_created_at",
        ];

        let run_schema = serde_json::to_string(schemas.get("RunResponse").unwrap()).unwrap();
        for field in fields {
            assert!(
                run_schema.contains(&format!("\"{field}\"")),
                "RunResponse lacks {field}: {run_schema}"
            );
        }
        let detail_schema =
            serde_json::to_string(schemas.get("RunDetailResponse").unwrap()).unwrap();
        assert!(
            detail_schema.contains("#/components/schemas/RunResponse"),
            "RunDetailResponse must extend RunResponse: {detail_schema}"
        );

        let run = serde_json::to_value(schemas.get("RunResponse").unwrap()).unwrap();
        assert_eq!(run["properties"]["number"]["type"], "integer");
        assert_eq!(run["properties"]["number"]["format"], "int64");
        assert_eq!(run["properties"]["steps_done"]["format"], "int32");
        assert_eq!(run["properties"]["steps_total"]["format"], "int32");
        let required = run["required"].as_array().expect("required fields");
        for field in [
            "number",
            "steps_done",
            "steps_total",
            "workflow_version_created_at",
        ] {
            assert!(
                required.iter().any(|name| name == field),
                "{field} must be required"
            );
        }
        for field in ["trigger_id", "trigger_event_name", "failed_connector_id"] {
            assert!(
                !required.iter().any(|name| name == field),
                "{field} is nullable and must not be required"
            );
        }
    }
}
