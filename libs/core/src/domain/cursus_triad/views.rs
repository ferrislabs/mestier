use chrono::{DateTime, Utc};

use crate::{CompanyTutorId, Contract, ParticipantId, Promotion, Triad, TriadLink, TutorRank};

#[derive(Debug, Clone, PartialEq)]
pub struct TriadSummary {
    pub triad: Triad,
    pub promotion_label: String,
    pub training_name: String,
    pub apprentice_name: String,
    pub pedagogic_tutor_name: Option<String>,
    pub current_host_company_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContractTutorView {
    pub company_tutor_id: CompanyTutorId,
    pub participant_id: ParticipantId,
    pub name: String,
    pub rank: TutorRank,
    pub archived_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContractView {
    pub contract: Contract,
    pub host_company_name: String,
    pub closure_reason_label: Option<String>,
    pub tutors: Vec<ContractTutorView>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TriadDetail {
    pub triad: Triad,
    pub promotion: Promotion,
    pub training_name: String,
    pub apprentice_name: String,
    pub apprentice_email: String,
    pub pedagogic_tutor_name: Option<String>,
    pub closure_reason_label: Option<String>,
    pub contracts: Vec<ContractView>,
}

impl TriadDetail {
    pub fn restricted_to_tutor(mut self, participant_id: ParticipantId) -> Self {
        self.contracts.retain(|view| {
            view.tutors
                .iter()
                .any(|tutor| tutor.participant_id == participant_id)
        });

        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MyTriad {
    pub summary: TriadSummary,
    pub link: TriadLink,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MyTriadDetail {
    pub detail: TriadDetail,
    pub link: TriadLink,
}
