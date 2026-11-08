use crate::{Absence, AbsenceType, DayOverride, Member, MemberContract};

#[derive(Debug, Clone)]
pub struct FeedMember {
    pub member: Member,
    pub archived: bool,
}

#[derive(Debug, Clone)]
pub struct HrPlanningFeed {
    pub members: Vec<FeedMember>,
    pub contracts: Vec<MemberContract>,
    pub absences: Vec<Absence>,
    pub overrides: Vec<DayOverride>,
    pub absence_types: Vec<AbsenceType>,
}
