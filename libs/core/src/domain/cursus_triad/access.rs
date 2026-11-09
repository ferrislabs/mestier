use chrono::{DateTime, Utc};

use crate::{OrganizationId, ParticipantId, TriadId, TutorRank};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriadRole {
    Apprentice,
    MainTutor,
    SecondaryTutor,
}

impl TriadRole {
    fn precedence(self) -> u8 {
        match self {
            Self::Apprentice => 2,
            Self::MainTutor => 1,
            Self::SecondaryTutor => 0,
        }
    }
}

impl From<TutorRank> for TriadRole {
    fn from(rank: TutorRank) -> Self {
        match rank {
            TutorRank::Main => Self::MainTutor,
            TutorRank::Secondary => Self::SecondaryTutor,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriadAccess {
    Active,
    ReadOnly { since: DateTime<Utc> },
}

impl TriadAccess {
    fn is_active(self) -> bool {
        matches!(self, Self::Active)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TriadLink {
    pub triad_id: TriadId,
    pub role: TriadRole,
    pub access: TriadAccess,
}

impl TriadLink {
    fn outranks(&self, other: &Self) -> bool {
        match (self.access.is_active(), other.access.is_active()) {
            (true, false) => true,
            (false, true) => false,
            _ => self.role.precedence() > other.role.precedence(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TutorTriadLink {
    pub triad_id: TriadId,
    pub rank: TutorRank,
    pub archived_at: Option<DateTime<Utc>>,
}

impl From<TutorTriadLink> for TriadLink {
    fn from(link: TutorTriadLink) -> Self {
        Self {
            triad_id: link.triad_id,
            role: link.rank.into(),
            access: match link.archived_at {
                Some(since) => TriadAccess::ReadOnly { since },
                None => TriadAccess::Active,
            },
        }
    }
}

pub fn merge_links(links: impl IntoIterator<Item = TriadLink>) -> Vec<TriadLink> {
    let mut merged: Vec<TriadLink> = Vec::new();
    for link in links {
        match merged
            .iter_mut()
            .find(|existing| existing.triad_id == link.triad_id)
        {
            Some(existing) if link.outranks(existing) => *existing = link,
            Some(_) => {}
            None => merged.push(link),
        }
    }

    merged
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticipantScope {
    pub participant_id: ParticipantId,
    pub organization_id: OrganizationId,
    pub links: Vec<TriadLink>,
}

impl ParticipantScope {
    pub fn new(
        participant_id: ParticipantId,
        organization_id: OrganizationId,
        apprentice_triads: &[TriadId],
        tutor_links: &[TutorTriadLink],
    ) -> Self {
        let links = merge_links(
            apprentice_triads
                .iter()
                .map(|triad_id| TriadLink {
                    triad_id: *triad_id,
                    role: TriadRole::Apprentice,
                    access: TriadAccess::Active,
                })
                .chain(tutor_links.iter().copied().map(TriadLink::from)),
        );

        Self {
            participant_id,
            organization_id,
            links,
        }
    }

    pub fn link(&self, triad_id: TriadId) -> Option<&TriadLink> {
        self.links.iter().find(|link| link.triad_id == triad_id)
    }
}
