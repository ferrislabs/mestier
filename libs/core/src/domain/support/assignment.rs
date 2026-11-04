use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::{MemberId, domain::support::ports::LastAssignment};

pub fn pick_least_recently_assigned(
    members: &[MemberId],
    last_assigned: &[LastAssignment],
) -> Option<MemberId> {
    let mut latest: HashMap<MemberId, DateTime<Utc>> = HashMap::new();
    for (member, assigned_at) in last_assigned {
        if let Some(assigned_at) = assigned_at {
            latest
                .entry(*member)
                .and_modify(|known| *known = (*known).max(*assigned_at))
                .or_insert(*assigned_at);
        }
    }

    members
        .iter()
        .copied()
        .min_by_key(|member| latest.get(member).copied())
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    use super::*;

    fn member(number: u128) -> MemberId {
        MemberId(Uuid::from_u128(number))
    }

    fn at(timestamp: &str) -> DateTime<Utc> {
        timestamp.parse().unwrap()
    }

    #[test]
    fn a_never_assigned_member_is_picked_before_an_assigned_one() {
        let members = [member(1), member(2), member(3)];
        let last_assigned = [
            (member(1), Some(at("2026-10-01T08:00:00Z"))),
            (member(2), None),
            (member(3), Some(at("2026-10-02T08:00:00Z"))),
        ];

        assert_eq!(
            pick_least_recently_assigned(&members, &last_assigned),
            Some(member(2))
        );
    }

    #[test]
    fn a_member_with_no_entry_counts_as_never_assigned() {
        let members = [member(1), member(2)];
        let last_assigned = [(member(1), Some(at("2026-10-01T08:00:00Z")))];

        assert_eq!(
            pick_least_recently_assigned(&members, &last_assigned),
            Some(member(2))
        );
    }

    #[test]
    fn with_nothing_known_about_anyone_the_first_member_wins() {
        let members = [member(7), member(3), member(5)];

        assert_eq!(pick_least_recently_assigned(&members, &[]), Some(member(7)));
    }

    #[test]
    fn among_assigned_members_the_oldest_assignment_wins() {
        let members = [member(1), member(2), member(3)];
        let last_assigned = [
            (member(1), Some(at("2026-10-03T08:00:00Z"))),
            (member(2), Some(at("2026-10-01T08:00:00Z"))),
            (member(3), Some(at("2026-10-02T08:00:00Z"))),
        ];

        assert_eq!(
            pick_least_recently_assigned(&members, &last_assigned),
            Some(member(2))
        );
    }

    #[test]
    fn the_result_does_not_depend_on_the_order_of_the_assignments() {
        let members = [member(1), member(2), member(3)];
        let last_assigned = [
            (member(3), Some(at("2026-10-02T08:00:00Z"))),
            (member(2), Some(at("2026-10-01T08:00:00Z"))),
            (member(1), Some(at("2026-10-03T08:00:00Z"))),
        ];

        assert_eq!(
            pick_least_recently_assigned(&members, &last_assigned),
            Some(member(2))
        );
    }

    #[test]
    fn ties_between_assigned_members_go_to_the_first_in_the_list() {
        let same = Some(at("2026-10-01T08:00:00Z"));
        let last_assigned = [(member(1), same), (member(2), same), (member(3), same)];

        assert_eq!(
            pick_least_recently_assigned(&[member(2), member(3), member(1)], &last_assigned),
            Some(member(2))
        );
        assert_eq!(
            pick_least_recently_assigned(&[member(3), member(1), member(2)], &last_assigned),
            Some(member(3))
        );
    }

    #[test]
    fn ties_between_never_assigned_members_go_to_the_first_in_the_list() {
        let last_assigned = [
            (member(1), Some(at("2026-10-01T08:00:00Z"))),
            (member(2), None),
            (member(3), None),
        ];

        assert_eq!(
            pick_least_recently_assigned(&[member(3), member(2), member(1)], &last_assigned),
            Some(member(3))
        );
        assert_eq!(
            pick_least_recently_assigned(&[member(1), member(2), member(3)], &last_assigned),
            Some(member(2))
        );
    }

    #[test]
    fn no_member_means_no_pick() {
        assert_eq!(pick_least_recently_assigned(&[], &[]), None);
        assert_eq!(
            pick_least_recently_assigned(&[], &[(member(1), None)]),
            None
        );
    }

    #[test]
    fn a_member_listed_twice_does_not_change_the_result() {
        let last_assigned = [
            (member(1), Some(at("2026-10-01T08:00:00Z"))),
            (member(2), Some(at("2026-10-01T08:00:00Z"))),
            (member(3), Some(at("2026-10-02T08:00:00Z"))),
        ];
        let once = [member(1), member(2), member(3)];
        let repeated = [member(1), member(2), member(1), member(3), member(2)];

        assert_eq!(
            pick_least_recently_assigned(&repeated, &last_assigned),
            pick_least_recently_assigned(&once, &last_assigned)
        );
        assert_eq!(
            pick_least_recently_assigned(&repeated, &last_assigned),
            Some(member(1))
        );

        let swapped = [member(2), member(1), member(2), member(3), member(1)];
        assert_eq!(
            pick_least_recently_assigned(&swapped, &last_assigned),
            Some(member(2))
        );
    }

    #[test]
    fn a_member_listed_twice_with_a_never_assigned_twin_is_still_never_assigned() {
        let members = [member(1), member(2), member(2)];
        let last_assigned = [(member(1), Some(at("2026-10-01T08:00:00Z")))];

        assert_eq!(
            pick_least_recently_assigned(&members, &last_assigned),
            Some(member(2))
        );
    }

    #[test]
    fn an_entry_for_a_member_outside_the_list_is_ignored() {
        let members = [member(2), member(3)];
        let last_assigned = [
            (member(1), None),
            (member(2), Some(at("2026-10-01T08:00:00Z"))),
            (member(3), Some(at("2026-10-02T08:00:00Z"))),
        ];

        assert_eq!(
            pick_least_recently_assigned(&members, &last_assigned),
            Some(member(2))
        );
    }

    #[test]
    fn a_single_member_is_always_picked() {
        let last_assigned = [(member(4), Some(at("2026-10-01T08:00:00Z")))];

        assert_eq!(
            pick_least_recently_assigned(&[member(4)], &last_assigned),
            Some(member(4))
        );
    }

    #[test]
    fn several_entries_for_one_member_count_by_the_latest() {
        let members = [member(1), member(2)];
        let last_assigned = [
            (member(1), Some(at("2026-10-01T08:00:00Z"))),
            (member(1), Some(at("2026-10-09T08:00:00Z"))),
            (member(2), Some(at("2026-10-05T08:00:00Z"))),
        ];

        assert_eq!(
            pick_least_recently_assigned(&members, &last_assigned),
            Some(member(2))
        );
    }
}
