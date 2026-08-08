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
    pub struct Customer;
    pub struct CustomerContact;
    pub struct Employee;
    pub struct Equipment;
    pub struct EventLog;
    pub struct Organization;
    pub struct Planning;
    pub struct Product;
    pub struct CustomerContext;
    pub struct Quote;
    pub struct Role;
    pub struct Member;
    pub struct ServiceRate;
    pub struct User;
    pub struct Category;
    pub struct Channel;
    pub struct Message;
    pub struct Webhook;
    pub struct Presence;
    pub struct ReadState;
    pub struct Overwrite;
    pub struct Notification;
    pub struct Task;
    pub struct TaskComment;
    pub struct TaskLabel;
    pub struct EmployeeRhythm;
    pub struct EmployeeWorkSlot;
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
