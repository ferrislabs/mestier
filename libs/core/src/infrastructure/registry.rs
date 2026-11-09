//! Repository registry: maps a (domain, backend) pair to a concrete repository
//! implementation. Each `Pg*Repository` declares its membership via
//! `#[mestier_macros::repository(domain = X, backend = Postgres)]`, which expands
//! to a [`RepoFor`] impl. Use-case macros resolve the right type by querying
//! `<Backend as RepoFor<Domain>>::Repo`, so swapping the active backend is a
//! single-line change ([`Backend`] alias below).

use crate::infrastructure::postgres::SharedTx;

/// Domain markers — one zero-sized type per aggregate root.
pub mod domain {
    pub struct Absence;
    pub struct AbsenceType;
    pub struct AssignmentReport;
    pub struct Customer;
    pub struct CustomerContact;
    pub struct CustomUnit;
    pub struct Deal;
    pub struct Doe;
    pub struct DoeModel;
    pub struct DoeSettings;
    pub struct Employee;
    pub struct EmployeeCostBasis;
    pub struct Equipment;
    pub struct Expense;
    pub struct ExpenseCategory;
    pub struct EventDispatch;
    pub struct EventLog;
    pub struct Invitation;
    pub struct Invoice;
    pub struct Ledger;
    pub struct Receipt;
    pub struct MaintenanceContract;
    pub struct MaintenanceIntervention;
    pub struct SupportApiKey;
    pub struct SupportSettings;
    pub struct SupportTicket;
    pub struct SupportTicketEntry;
    pub struct Organization;
    pub struct Planning;
    pub struct Product;
    pub struct Project;
    pub struct ProjectTemplate;
    pub struct Credential;
    pub struct OauthState;
    pub struct CustomerContext;
    pub struct AutomationSettings;
    pub struct Quote;
    pub struct Role;
    pub struct DayOverride;
    pub struct ExtraPeriod;
    pub struct LeaveAccount;
    pub struct Surcharge;
    pub struct LeaveAdjustment;
    pub struct Member;
    pub struct MemberContract;
    pub struct MemberDocument;
    pub struct MemberProfile;
    pub struct ServiceRate;
    pub struct Supplier;
    pub struct SupplierInvoice;
    pub struct SupplierInvoiceAllocation;
    pub struct User;
    pub struct Category;
    pub struct CategoryAccess;
    pub struct Channel;
    pub struct DirectChannel;
    pub struct Message;
    pub struct Webhook;
    pub struct Presence;
    pub struct ReadState;
    pub struct Overwrite;
    pub struct Notification;
    pub struct Task;
    pub struct TaskComment;
    pub struct TaskLabel;
    pub struct TaskRecurrence;
    pub struct EmployeeRhythm;
    pub struct WorkSlot;
    pub struct Profitability;
    pub struct TimeEntry;
    pub struct DayLog;
    pub struct Workflow;
    pub struct WorkflowFolder;
    pub struct Collection;
    pub struct Run;
    pub struct Subscription;
    pub struct WebhookRoute;
    pub struct Schedule;
}

/// Backend markers — one zero-sized type per supported persistence layer.
pub mod backend {
    pub struct Postgres;
}

/// Active backend for the whole process. To migrate (e.g. Postgres → Mongo),
/// change this single alias and provide [`RepoFor`] impls for the new backend.
pub type Backend = backend::Postgres;

/// Resolves a domain to its concrete repository type for a given backend.
pub trait RepoFor<D> {
    type Repo<'tx>;

    fn build<'tx>(tx: &SharedTx<'tx>) -> Self::Repo<'tx>;
}
