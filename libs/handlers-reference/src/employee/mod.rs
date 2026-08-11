//! The contractual profile of a member.
//!
//! There is no `/employees` collection any more: a profile is not addressed on
//! its own, it is attached to or detached from a seat. What used to be five
//! routes — create, list, get, patch, delete — is two, both hanging off the
//! member they describe.

pub mod list;
pub mod remove_profile;
pub mod upsert_profile;
