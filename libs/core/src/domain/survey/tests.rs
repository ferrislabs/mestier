use proptest::prelude::*;

use super::{Answer, EmailAddress, PhoneNumber, PublicSlug, SurveyError};

#[test]
fn a_blank_answer_is_recognised_per_kind() {
    assert!(Answer::Text("   ".to_string()).is_blank());
    assert!(Answer::Text(String::new()).is_blank());
    assert!(!Answer::Text(" a ".to_string()).is_blank());
    assert!(Answer::Choices(vec![]).is_blank());
    assert!(!Answer::Choices(vec!["A".to_string()]).is_blank());
    assert!(Answer::Photos(vec![]).is_blank());
    assert!(!Answer::Photos(vec!["k".to_string()]).is_blank());
    assert!(!Answer::Number(0.0).is_blank());
    assert!(!Answer::Choice("A".to_string()).is_blank());
}

#[test]
fn a_public_slug_accepts_letters_digits_dashes_and_underscores() {
    let slug = PublicSlug::new("Ab3-x_9Zq".to_string()).unwrap();
    assert_eq!(slug.as_str(), "Ab3-x_9Zq");
    assert!(PublicSlug::try_from("a".repeat(64)).is_ok());
}

#[test]
fn a_public_slug_outside_its_shape_is_refused() {
    for raw in [
        String::new(),
        "short7".to_string(),
        "a".repeat(65),
        "has space1".to_string(),
        "accentué12".to_string(),
        "slash/abc".to_string(),
    ] {
        assert_eq!(
            PublicSlug::new(raw.clone()),
            Err(SurveyError::PublicSlugInvalid),
            "{raw}"
        );
    }
}

#[test]
fn a_public_slug_is_validated_on_deserialization() {
    assert!(serde_json::from_str::<PublicSlug>("\"bad slug\"").is_err());
    let slug = PublicSlug::new("abcdefgh".to_string()).unwrap();
    let json = serde_json::to_string(&slug).unwrap();
    assert_eq!(serde_json::from_str::<PublicSlug>(&json).unwrap(), slug);
}

#[test]
fn a_survey_error_is_a_conflict_for_the_core() {
    assert_eq!(
        common::CoreError::from(SurveyError::SurveyClosed).to_string(),
        common::CoreError::Conflict(SurveyError::SurveyClosed.to_string()).to_string()
    );
}

#[test]
fn an_email_is_trimmed_and_lowercased() {
    let email = EmailAddress::try_from("  Jane.Doe@Example.COM ").unwrap();
    assert_eq!(email.as_str(), "jane.doe@example.com");
}

#[test]
fn an_email_without_an_at_sign_is_refused() {
    assert_eq!(
        EmailAddress::try_from("jane.example.com"),
        Err(SurveyError::EmailInvalid)
    );
}

#[test]
fn a_phone_keeps_only_a_plus_and_digits() {
    let phone = PhoneNumber::try_from(" +33 (0)6-12 34 56 78 ").unwrap();
    assert_eq!(phone.as_str(), "+330612345678");
}

#[test]
fn a_phone_with_five_digits_is_refused() {
    assert_eq!(
        PhoneNumber::try_from("12 345"),
        Err(SurveyError::PhoneInvalid)
    );
}

#[test]
fn a_phone_with_letters_is_refused() {
    assert_eq!(
        PhoneNumber::try_from("06 12 34 56 7a"),
        Err(SurveyError::PhoneInvalid)
    );
}

#[test]
fn an_invalid_email_cannot_be_deserialized() {
    assert!(serde_json::from_str::<EmailAddress>("\"x\"").is_err());
}

#[test]
fn an_invalid_phone_cannot_be_deserialized() {
    assert!(serde_json::from_str::<PhoneNumber>("\"x\"").is_err());
}

#[test]
fn an_email_survives_a_serde_round_trip() {
    let email = EmailAddress::try_from("jane@example.com").unwrap();
    let json = serde_json::to_string(&email).unwrap();
    assert_eq!(serde_json::from_str::<EmailAddress>(&json).unwrap(), email);
}

#[test]
fn a_phone_survives_a_serde_round_trip() {
    let phone = PhoneNumber::try_from("+33 6 12 34 56 78").unwrap();
    let json = serde_json::to_string(&phone).unwrap();
    assert_eq!(serde_json::from_str::<PhoneNumber>(&json).unwrap(), phone);
}

#[test]
fn a_phone_with_six_digits_is_accepted() {
    assert!(PhoneNumber::try_from("123456").is_ok());
}

#[test]
fn a_phone_with_fifteen_digits_is_accepted() {
    assert!(PhoneNumber::try_from("123456789012345").is_ok());
}

#[test]
fn a_phone_with_sixteen_digits_is_refused() {
    assert_eq!(
        PhoneNumber::try_from("1234567890123456"),
        Err(SurveyError::PhoneInvalid)
    );
}

#[test]
fn a_plus_that_is_not_first_is_refused() {
    assert_eq!(
        PhoneNumber::try_from("12+345678"),
        Err(SurveyError::PhoneInvalid)
    );
}

proptest! {
    #[test]
    fn an_email_normalization_is_idempotent(raw in "[ \\t]{0,3}[A-Za-z0-9.]{1,8}@[A-Za-z0-9.]{1,8}[ \\t]{0,3}") {
        let once = EmailAddress::try_from(raw.as_str()).unwrap();
        let twice = EmailAddress::try_from(once.as_str());
        prop_assert_eq!(twice, Ok(once));
    }
}

mod graph {
    use serde::Deserialize;
    use uuid::Uuid;

    use super::super::{
        Answer, Answers, Branch, ConditionNode, Edge, EndNode, Graph, GraphPath, Node, QuestionId,
        QuestionKind, QuestionNode, SurveyError,
    };

    fn id(n: u128) -> QuestionId {
        QuestionId(Uuid::from_u128(n))
    }

    fn question(n: u128, kind: QuestionKind, options: &[&str]) -> Node {
        Node::Question(QuestionNode {
            id: id(n),
            kind,
            label: format!("Question {n}"),
            required: true,
            options: options.iter().map(|option| option.to_string()).collect(),
            max_photos: None,
            identifier: false,
        })
    }

    fn choice(n: u128) -> Node {
        question(n, QuestionKind::SingleChoice, &["Oui", "Non"])
    }

    fn text(n: u128) -> Node {
        question(n, QuestionKind::Text, &[])
    }

    fn condition(n: u128, on: u128, option: &str, checked: bool) -> Node {
        Node::Condition(ConditionNode {
            id: id(n),
            question: id(on),
            option: option.to_string(),
            checked,
        })
    }

    fn end(n: u128) -> Node {
        Node::End(EndNode {
            id: id(n),
            message: None,
        })
    }

    fn next(from: u128, to: u128) -> Edge {
        Edge {
            from: id(from),
            to: id(to),
            branch: Branch::Next,
        }
    }

    fn then(from: u128, to: u128) -> Edge {
        Edge {
            from: id(from),
            to: id(to),
            branch: Branch::Then,
        }
    }

    fn otherwise(from: u128, to: u128) -> Edge {
        Edge {
            from: id(from),
            to: id(to),
            branch: Branch::Else,
        }
    }

    fn branching() -> Graph {
        Graph::new(
            vec![
                choice(1),
                condition(2, 1, "Oui", true),
                text(3),
                text(4),
                end(5),
            ],
            vec![
                next(1, 2),
                then(2, 3),
                otherwise(2, 4),
                next(3, 5),
                next(4, 5),
            ],
            id(1),
        )
        .unwrap()
    }

    fn answered(question: u128, answer: Answer) -> Answers {
        Answers::from([(id(question), answer)])
    }

    #[test]
    fn a_graph_without_nodes_is_refused() {
        assert_eq!(
            Graph::new(vec![], vec![], id(1)),
            Err(SurveyError::GraphEmpty)
        );
    }

    #[test]
    fn a_graph_without_an_end_is_refused() {
        assert_eq!(
            Graph::new(vec![text(1), text(2)], vec![next(1, 2)], id(1)),
            Err(SurveyError::GraphWithoutEnd)
        );
    }

    #[test]
    fn a_cycle_is_refused() {
        assert_eq!(
            Graph::new(
                vec![choice(1), text(2), condition(3, 1, "Oui", true), end(4)],
                vec![next(1, 2), next(2, 3), then(3, 1), otherwise(3, 4)],
                id(1),
            ),
            Err(SurveyError::GraphCycle(id(1)))
        );
    }

    #[test]
    fn a_node_with_no_way_to_an_end_is_refused() {
        assert_eq!(
            Graph::new(
                vec![choice(1), condition(2, 1, "Oui", true), text(3), end(4)],
                vec![next(1, 2), then(2, 3), otherwise(2, 4)],
                id(1),
            ),
            Err(SurveyError::GraphDeadEnd(id(3)))
        );
    }

    #[test]
    fn a_condition_without_its_else_branch_is_refused() {
        assert_eq!(
            Graph::new(
                vec![choice(1), condition(2, 1, "Oui", true), end(3)],
                vec![next(1, 2), then(2, 3)],
                id(1),
            ),
            Err(SurveyError::GraphDeadEnd(id(2)))
        );
    }

    #[test]
    fn a_node_unreachable_from_the_start_is_refused() {
        assert_eq!(
            Graph::new(
                vec![text(1), text(2), end(3)],
                vec![next(1, 3), next(2, 3)],
                id(1),
            ),
            Err(SurveyError::GraphUnreachableNode(id(2)))
        );
    }

    #[test]
    fn an_unknown_start_is_refused() {
        assert_eq!(
            Graph::new(vec![text(1), end(2)], vec![next(1, 2)], id(9)),
            Err(SurveyError::GraphUnknownStart)
        );
    }

    #[test]
    fn a_start_that_is_not_a_question_is_refused() {
        assert_eq!(
            Graph::new(
                vec![choice(1), condition(2, 1, "Oui", true), end(3)],
                vec![next(1, 2), then(2, 3), otherwise(2, 3)],
                id(2),
            ),
            Err(SurveyError::GraphStartNotQuestion)
        );
    }

    #[test]
    fn two_nodes_sharing_an_id_are_refused() {
        assert_eq!(
            Graph::new(vec![text(1), end(1)], vec![], id(1)),
            Err(SurveyError::GraphDuplicateNode(id(1)))
        );
    }

    #[test]
    fn an_edge_to_an_unknown_node_is_refused() {
        assert_eq!(
            Graph::new(vec![text(1), end(2)], vec![next(1, 9)], id(1)),
            Err(SurveyError::GraphUnknownEdgeNode(id(9)))
        );
    }

    #[test]
    fn a_question_with_a_then_branch_is_refused() {
        assert_eq!(
            Graph::new(vec![text(1), end(2)], vec![then(1, 2)], id(1)),
            Err(SurveyError::GraphInvalidBranch(id(1)))
        );
    }

    #[test]
    fn an_edge_leaving_an_end_is_refused() {
        assert_eq!(
            Graph::new(vec![text(1), end(2)], vec![next(1, 2), next(2, 1)], id(1)),
            Err(SurveyError::GraphInvalidBranch(id(2)))
        );
    }

    #[test]
    fn a_question_with_two_exits_is_refused() {
        assert_eq!(
            Graph::new(
                vec![text(1), end(2), end(3)],
                vec![next(1, 2), next(1, 3)],
                id(1),
            ),
            Err(SurveyError::GraphDuplicateExit(id(1)))
        );
    }

    #[test]
    fn a_condition_on_an_unknown_question_is_refused() {
        assert_eq!(
            Graph::new(
                vec![choice(1), condition(2, 9, "Oui", true), end(3)],
                vec![next(1, 2), then(2, 3), otherwise(2, 3)],
                id(1),
            ),
            Err(SurveyError::ConditionUnknownQuestion(id(2)))
        );
    }

    #[test]
    fn a_condition_on_an_unknown_option_is_refused() {
        assert_eq!(
            Graph::new(
                vec![choice(1), condition(2, 1, "Peut-être", true), end(3)],
                vec![next(1, 2), then(2, 3), otherwise(2, 3)],
                id(1),
            ),
            Err(SurveyError::ConditionUnknownOption(id(2)))
        );
    }

    #[test]
    fn a_condition_on_a_text_question_is_refused() {
        assert_eq!(
            Graph::new(
                vec![text(1), condition(2, 1, "Oui", true), end(3)],
                vec![next(1, 2), then(2, 3), otherwise(2, 3)],
                id(1),
            ),
            Err(SurveyError::ConditionOnNonChoice(id(2)))
        );
    }

    #[test]
    fn a_condition_on_a_condition_is_refused() {
        assert_eq!(
            Graph::new(
                vec![
                    choice(1),
                    condition(2, 1, "Oui", true),
                    condition(3, 2, "Oui", true),
                    end(4),
                ],
                vec![
                    next(1, 2),
                    then(2, 3),
                    otherwise(2, 3),
                    then(3, 4),
                    otherwise(3, 4),
                ],
                id(1),
            ),
            Err(SurveyError::ConditionOnNonChoice(id(3)))
        );
    }

    #[test]
    fn a_condition_on_a_question_not_asked_on_every_path_is_refused() {
        assert_eq!(
            Graph::new(
                vec![
                    choice(1),
                    condition(2, 1, "Oui", true),
                    choice(3),
                    text(4),
                    condition(5, 3, "Oui", true),
                    end(6),
                ],
                vec![
                    next(1, 2),
                    then(2, 3),
                    otherwise(2, 4),
                    next(3, 5),
                    next(4, 5),
                    then(5, 6),
                    otherwise(5, 6),
                ],
                id(1),
            ),
            Err(SurveyError::ConditionQuestionNotAsked(id(5)))
        );
    }

    #[test]
    fn a_condition_on_a_question_asked_after_it_is_refused() {
        assert_eq!(
            Graph::new(
                vec![text(1), condition(2, 3, "Oui", true), choice(3), end(4)],
                vec![next(1, 2), then(2, 3), otherwise(2, 3), next(3, 4)],
                id(1),
            ),
            Err(SurveyError::ConditionQuestionNotAsked(id(2)))
        );
    }

    #[test]
    fn two_branches_may_rejoin_on_a_common_question() {
        let graph = Graph::new(
            vec![
                choice(1),
                condition(2, 1, "Oui", true),
                text(3),
                text(4),
                text(5),
                end(6),
            ],
            vec![
                next(1, 2),
                then(2, 3),
                otherwise(2, 4),
                next(3, 5),
                next(4, 5),
                next(5, 6),
            ],
            id(1),
        )
        .unwrap();
        assert_eq!(
            graph.path(&answered(1, Answer::Choice("Oui".to_string()))),
            GraphPath {
                questions: vec![id(1), id(3), id(5)],
                end: id(6),
            }
        );
    }

    #[test]
    fn the_then_branch_is_followed_when_the_option_is_checked() {
        assert_eq!(
            branching().path(&answered(1, Answer::Choice("Oui".to_string()))),
            GraphPath {
                questions: vec![id(1), id(3)],
                end: id(5),
            }
        );
    }

    #[test]
    fn the_else_branch_is_followed_otherwise() {
        assert_eq!(
            branching().path(&answered(1, Answer::Choice("Non".to_string()))),
            GraphPath {
                questions: vec![id(1), id(4)],
                end: id(5),
            }
        );
    }

    #[test]
    fn an_unanswered_condition_source_takes_the_else_branch() {
        assert_eq!(
            branching().path(&Answers::new()),
            GraphPath {
                questions: vec![id(1), id(4)],
                end: id(5),
            }
        );
    }

    #[test]
    fn a_not_checked_condition_takes_then_when_the_option_is_absent() {
        let graph = Graph::new(
            vec![
                question(1, QuestionKind::MultipleChoice, &["A", "B"]),
                condition(2, 1, "A", false),
                text(3),
                end(4),
            ],
            vec![next(1, 2), then(2, 3), otherwise(2, 4), next(3, 4)],
            id(1),
        )
        .unwrap();
        assert_eq!(
            graph.path(&answered(1, Answer::Choices(vec!["B".to_string()]))),
            GraphPath {
                questions: vec![id(1), id(3)],
                end: id(4),
            }
        );
    }

    #[test]
    fn an_invalid_graph_cannot_be_deserialized() {
        let json = r#"{
            "nodes": [
                {"type": "QUESTION", "id": "00000000-0000-0000-0000-000000000001", "kind": "TEXT", "label": "A", "required": true},
                {"type": "QUESTION", "id": "00000000-0000-0000-0000-000000000002", "kind": "TEXT", "label": "B", "required": true},
                {"type": "END", "id": "00000000-0000-0000-0000-000000000003"}
            ],
            "edges": [
                {"from": "00000000-0000-0000-0000-000000000001", "to": "00000000-0000-0000-0000-000000000002", "branch": "NEXT"},
                {"from": "00000000-0000-0000-0000-000000000002", "to": "00000000-0000-0000-0000-000000000001", "branch": "NEXT"}
            ],
            "start": "00000000-0000-0000-0000-000000000001"
        }"#;
        let error = serde_json::from_str::<Graph>(json).unwrap_err().to_string();
        assert!(
            error.contains(&SurveyError::GraphCycle(id(1)).to_string()),
            "{error}"
        );
    }

    fn shaped(n: u128, kind: QuestionKind, max_photos: Option<u8>, identifier: bool) -> Node {
        Node::Question(QuestionNode {
            id: id(n),
            kind,
            label: format!("Question {n}"),
            required: true,
            options: vec![],
            max_photos,
            identifier,
        })
    }

    fn linear(first: Node, second: Node) -> Result<Graph, SurveyError> {
        Graph::new(
            vec![first, second, end(3)],
            vec![next(1, 2), next(2, 3)],
            id(1),
        )
    }

    #[test]
    fn a_self_loop_is_refused() {
        assert_eq!(
            Graph::new(vec![text(1), end(2)], vec![next(1, 1)], id(1)),
            Err(SurveyError::GraphCycle(id(1)))
        );
    }

    #[test]
    fn a_cycle_away_from_the_start_is_refused() {
        assert_eq!(
            Graph::new(
                vec![
                    choice(1),
                    condition(2, 1, "Oui", true),
                    text(3),
                    text(4),
                    end(5)
                ],
                vec![
                    next(1, 2),
                    then(2, 3),
                    otherwise(2, 5),
                    next(3, 4),
                    next(4, 3),
                ],
                id(1),
            ),
            Err(SurveyError::GraphCycle(id(3)))
        );
    }

    #[test]
    fn a_condition_with_two_then_branches_is_refused() {
        assert_eq!(
            Graph::new(
                vec![choice(1), condition(2, 1, "Oui", true), end(3), end(4)],
                vec![next(1, 2), then(2, 3), then(2, 4), otherwise(2, 3)],
                id(1),
            ),
            Err(SurveyError::GraphDuplicateExit(id(2)))
        );
    }

    #[test]
    fn a_second_identifier_is_refused() {
        assert_eq!(
            linear(
                shaped(1, QuestionKind::Email, None, true),
                shaped(2, QuestionKind::Phone, None, true),
            ),
            Err(SurveyError::GraphSecondIdentifier(id(2)))
        );
    }

    #[test]
    fn an_identifier_on_a_number_question_is_refused() {
        assert_eq!(
            linear(text(1), shaped(2, QuestionKind::Number, None, true)),
            Err(SurveyError::GraphIdentifierOnInvalidKind(id(2)))
        );
    }

    #[test]
    fn an_identifier_on_a_photo_question_is_refused() {
        assert_eq!(
            linear(text(1), shaped(2, QuestionKind::Photo, None, true)),
            Err(SurveyError::GraphIdentifierOnInvalidKind(id(2)))
        );
    }

    #[test]
    fn one_identifier_on_an_email_phone_or_text_question_is_accepted() {
        for kind in [QuestionKind::Email, QuestionKind::Phone, QuestionKind::Text] {
            assert!(linear(shaped(1, kind, None, true), text(2)).is_ok());
        }
    }

    #[test]
    fn a_photo_limit_on_a_text_question_is_refused() {
        assert_eq!(
            linear(text(1), shaped(2, QuestionKind::Text, Some(5), false)),
            Err(SurveyError::GraphMaxPhotosOnInvalidKind(id(2)))
        );
    }

    #[test]
    fn a_photo_limit_of_zero_is_refused() {
        assert_eq!(
            linear(text(1), shaped(2, QuestionKind::Photo, Some(0), false)),
            Err(SurveyError::GraphZeroMaxPhotos(id(2)))
        );
    }

    #[test]
    fn a_photo_question_with_or_without_a_limit_is_accepted() {
        assert!(linear(shaped(1, QuestionKind::Photo, Some(3), false), text(2)).is_ok());
        assert!(linear(shaped(1, QuestionKind::Photo, None, false), text(2)).is_ok());
    }

    #[test]
    fn a_single_choice_without_options_is_refused() {
        assert_eq!(
            linear(text(1), question(2, QuestionKind::SingleChoice, &[])),
            Err(SurveyError::GraphChoiceWithoutOptions(id(2)))
        );
    }

    #[test]
    fn a_multiple_choice_without_options_is_refused() {
        assert_eq!(
            linear(text(1), question(2, QuestionKind::MultipleChoice, &[])),
            Err(SurveyError::GraphChoiceWithoutOptions(id(2)))
        );
    }

    fn labelled(label: &str, options: &[&str]) -> Node {
        Node::Question(QuestionNode {
            id: id(2),
            kind: QuestionKind::SingleChoice,
            label: label.to_string(),
            required: true,
            options: options.iter().map(|option| option.to_string()).collect(),
            max_photos: None,
            identifier: false,
        })
    }

    #[test]
    fn a_blank_question_label_is_refused() {
        assert_eq!(
            linear(text(1), labelled("  ", &["A"])),
            Err(SurveyError::GraphQuestionLabelEmpty(id(2)))
        );
    }

    #[test]
    fn a_blank_option_is_refused() {
        assert_eq!(
            linear(text(1), labelled("Q", &["A", " "])),
            Err(SurveyError::GraphOptionEmpty(id(2)))
        );
    }

    #[test]
    fn a_duplicate_option_is_refused() {
        assert_eq!(
            linear(text(1), labelled("Q", &["A", "B", "A"])),
            Err(SurveyError::GraphDuplicateOption(id(2)))
        );
    }

    #[test]
    fn a_graph_survives_a_serde_round_trip() {
        let graph = branching();
        let json = serde_json::to_string(&graph).unwrap();
        assert_eq!(serde_json::from_str::<Graph>(&json).unwrap(), graph);
    }

    #[derive(Deserialize)]
    struct PathCase {
        name: String,
        graph: Graph,
        answers: Answers,
        expected_questions: Vec<QuestionId>,
        expected_end: QuestionId,
    }

    #[test]
    fn the_shared_path_cases_hold() {
        let cases: Vec<PathCase> =
            serde_json::from_str(include_str!("fixtures/paths.json")).unwrap();
        assert!(cases.len() >= 7);
        for case in cases {
            assert_eq!(
                case.graph.path(&case.answers),
                GraphPath {
                    questions: case.expected_questions,
                    end: case.expected_end,
                },
                "{}",
                case.name
            );
        }
    }
}

mod lifecycle {
    use std::collections::BTreeMap;

    use chrono::{DateTime, TimeZone, Utc};
    use uuid::Uuid;

    use super::super::aggregate::{Closed, Draft, Opened, Survey};
    use super::super::campaign::{Campaign, CampaignStatus, Window};
    use super::super::metadata::Metadata;
    use super::super::response::{Completed, InProgress, Response, ResumeToken};
    use super::super::{
        Answer, Answers, Branch, CampaignId, ConditionNode, Edge, EmailAddress, EndNode, Graph,
        Node, PhoneNumber, PublicSlug, QuestionId, QuestionKind, QuestionNode, ResponseId,
        SurveyError, SurveyId, Visibility,
    };
    use crate::{MemberId, OrganizationId};

    fn at(day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 3, day, 12, 0, 0).unwrap()
    }

    fn qid(n: u128) -> QuestionId {
        QuestionId(Uuid::from_u128(n))
    }

    fn org() -> OrganizationId {
        OrganizationId(Uuid::from_u128(900))
    }

    fn node(n: u128, kind: QuestionKind, required: bool, identifier: bool) -> Node {
        Node::Question(QuestionNode {
            id: qid(n),
            kind,
            label: format!("Question {n}"),
            required,
            options: if kind == QuestionKind::SingleChoice {
                vec!["Oui".to_string(), "Non".to_string()]
            } else {
                Vec::new()
            },
            max_photos: None,
            identifier,
        })
    }

    fn edge(from: u128, to: u128, branch: Branch) -> Edge {
        Edge {
            from: qid(from),
            to: qid(to),
            branch,
        }
    }

    fn end(n: u128) -> Node {
        Node::End(EndNode {
            id: qid(n),
            message: Some("Merci".to_string()),
        })
    }

    fn branching(required_else: bool) -> Graph {
        Graph::new(
            vec![
                node(1, QuestionKind::SingleChoice, true, false),
                Node::Condition(ConditionNode {
                    id: qid(2),
                    question: qid(1),
                    option: "Oui".to_string(),
                    checked: true,
                }),
                node(3, QuestionKind::Text, true, false),
                node(4, QuestionKind::Text, required_else, false),
                end(5),
            ],
            vec![
                edge(1, 2, Branch::Next),
                edge(2, 3, Branch::Then),
                edge(2, 4, Branch::Else),
                edge(3, 5, Branch::Next),
                edge(4, 5, Branch::Next),
            ],
            qid(1),
        )
        .unwrap()
    }

    fn draft() -> Survey<Draft> {
        Survey::new(
            SurveyId(Uuid::from_u128(1)),
            org(),
            "Satisfaction".to_string(),
            Visibility::Public,
        )
        .unwrap()
    }

    fn slug() -> PublicSlug {
        PublicSlug::new("abcdefgh".to_string()).unwrap()
    }

    fn opened() -> Survey<Opened> {
        let mut survey = draft();
        survey.edit_graph(Some(branching(true)));
        survey.open(at(1), Some(slug())).unwrap()
    }

    fn opened_in_campaign() -> Survey<Opened> {
        let mut survey = draft();
        survey.edit_graph(Some(branching(true)));
        survey.edit_campaign(Some(CampaignId(Uuid::from_u128(2))));
        survey.open(at(1), Some(slug())).unwrap()
    }

    fn members_draft() -> Survey<Draft> {
        let mut survey = draft();
        survey.edit_visibility(Visibility::Members);
        survey.edit_graph(Some(branching(true)));
        survey
    }

    fn campaign(starts: Option<u32>, ends: Option<u32>) -> Campaign {
        Campaign::new(
            CampaignId(Uuid::from_u128(2)),
            org(),
            "Printemps".to_string(),
            Window::new(starts.map(at), ends.map(at)).unwrap(),
            Metadata::default(),
            at(1),
        )
        .unwrap()
    }

    fn token() -> ResumeToken {
        ResumeToken::new("resume".to_string()).unwrap()
    }

    fn survey_of(graph: Graph) -> Survey<Opened> {
        let mut survey = draft();
        survey.edit_graph(Some(graph));
        survey.open(at(1), Some(slug())).unwrap()
    }

    fn response_to(survey: &Survey<Opened>) -> Response<InProgress> {
        Response::start(
            ResponseId(Uuid::from_u128(3)),
            survey,
            None,
            Some(token()),
            at(1),
        )
        .unwrap()
    }

    fn response() -> Response<InProgress> {
        response_to(&opened())
    }

    fn text(value: &str) -> Answer {
        Answer::Text(value.to_string())
    }

    fn choice(value: &str) -> Answer {
        Answer::Choice(value.to_string())
    }

    #[test]
    fn a_window_with_an_end_before_its_start_is_refused() {
        assert_eq!(
            Window::new(Some(at(5)), Some(at(2))),
            Err(SurveyError::WindowInverted)
        );
    }

    #[test]
    fn a_window_starting_when_it_ends_is_refused() {
        assert_eq!(
            Window::new(Some(at(5)), Some(at(5))),
            Err(SurveyError::WindowInverted)
        );
    }

    #[test]
    fn a_window_with_only_a_start_is_open_forever_after() {
        let window = Window::new(Some(at(5)), None).unwrap();
        assert!(!window.contains(at(4)));
        assert!(window.contains(at(5)));
        assert!(window.contains(at(30)));
    }

    #[test]
    fn a_window_with_only_an_end_is_open_until_then() {
        let window = Window::new(None, Some(at(5))).unwrap();
        assert!(window.contains(at(1)));
        assert!(!window.contains(at(5)));
    }

    #[test]
    fn a_window_includes_its_start_and_excludes_its_end() {
        let window = Window::new(Some(at(2)), Some(at(5))).unwrap();
        assert!(window.contains(at(2)));
        assert!(window.contains(at(4)));
        assert!(!window.contains(at(5)));
    }

    #[test]
    fn a_window_cannot_be_deserialized_inverted() {
        let json = serde_json::to_value(Window::new(Some(at(2)), Some(at(5))).unwrap()).unwrap();
        let inverted = serde_json::json!({
            "starts_at": json["ends_at"],
            "ends_at": json["starts_at"],
        });
        assert!(serde_json::from_value::<Window>(inverted).is_err());
    }

    #[test]
    fn campaign_status_follows_the_clock() {
        let campaign = campaign(Some(2), Some(5));
        assert_eq!(campaign.status_at(at(1)), CampaignStatus::Scheduled);
        assert_eq!(campaign.status_at(at(2)), CampaignStatus::Running);
        assert_eq!(campaign.status_at(at(5)), CampaignStatus::Ended);
        assert_eq!(
            self::campaign(None, None).status_at(at(1)),
            CampaignStatus::Running
        );
    }

    #[test]
    fn a_campaign_with_a_blank_name_is_refused() {
        assert_eq!(
            Campaign::new(
                CampaignId(Uuid::from_u128(2)),
                org(),
                "  ".to_string(),
                Window::new(None, None).unwrap(),
                Metadata::default(),
                at(1),
            ),
            Err(SurveyError::CampaignNameEmpty)
        );
    }

    #[test]
    fn a_new_campaign_starts_in_the_status_of_its_clock() {
        let campaign = campaign(Some(2), Some(5));
        assert_eq!(campaign.status(), CampaignStatus::Scheduled);
        assert_eq!(campaign.id(), CampaignId(Uuid::from_u128(2)));
        assert_eq!(campaign.organization_id(), org());
        assert_eq!(campaign.name(), "Printemps");
        assert_eq!(campaign.window().starts_at(), Some(at(2)));
        assert_eq!(campaign.metadata(), &Metadata::default());
        assert_eq!(
            self::campaign(None, Some(1)).status(),
            CampaignStatus::Ended
        );
    }

    #[test]
    fn a_campaign_advances_scheduled_then_running_then_ended() {
        let mut campaign = campaign(Some(2), Some(5));
        assert_eq!(campaign.advance(at(1)), None);
        assert_eq!(campaign.advance(at(2)), Some(CampaignStatus::Running));
        assert_eq!(campaign.advance(at(3)), None);
        assert_eq!(campaign.advance(at(5)), Some(CampaignStatus::Ended));
        assert_eq!(campaign.advance(at(6)), None);
        assert_eq!(campaign.status(), CampaignStatus::Ended);
    }

    #[test]
    fn a_campaign_whose_window_passed_jumps_to_ended() {
        let mut campaign = campaign(Some(2), Some(5));
        assert_eq!(campaign.advance(at(9)), Some(CampaignStatus::Ended));
        assert_eq!(campaign.status(), CampaignStatus::Ended);
    }

    #[test]
    fn a_campaign_never_goes_back() {
        let mut campaign = campaign(Some(2), Some(5));
        campaign.advance(at(3));
        assert_eq!(campaign.advance(at(1)), None);
        assert_eq!(campaign.status(), CampaignStatus::Running);
    }

    #[test]
    fn a_rescheduled_campaign_advances_but_never_regresses() {
        let mut running = campaign(Some(1), None);
        assert_eq!(running.status(), CampaignStatus::Running);
        assert_eq!(
            running.reschedule(Window::new(Some(at(8)), None).unwrap(), at(2)),
            None
        );
        assert_eq!(running.status(), CampaignStatus::Running);
        assert_eq!(running.window().starts_at(), Some(at(8)));
        let mut ended = campaign(None, Some(1));
        assert_eq!(
            ended.reschedule(Window::new(None, Some(at(9))).unwrap(), at(2)),
            None
        );
        assert_eq!(ended.status(), CampaignStatus::Ended);
        let mut scheduled = campaign(Some(5), None);
        assert_eq!(
            scheduled.reschedule(Window::new(None, Some(at(3))).unwrap(), at(4)),
            Some(CampaignStatus::Ended)
        );
    }

    #[test]
    fn a_campaign_can_be_renamed_and_its_metadata_edited() {
        let mut campaign = campaign(None, None);
        assert_eq!(
            campaign.rename(" ".to_string()),
            Err(SurveyError::CampaignNameEmpty)
        );
        campaign.rename("Automne".to_string()).unwrap();
        assert_eq!(campaign.name(), "Automne");
        let metadata = Metadata::new(BTreeMap::from([("k".to_string(), "v".to_string())])).unwrap();
        campaign.edit_metadata(metadata.clone());
        assert_eq!(campaign.metadata(), &metadata);
    }

    #[test]
    fn a_campaign_survives_a_serde_round_trip_and_refuses_a_blank_name() {
        let campaign = campaign(Some(2), Some(5));
        let json = serde_json::to_value(&campaign).unwrap();
        assert_eq!(json["status"], serde_json::json!("SCHEDULED"));
        assert_eq!(
            serde_json::from_value::<Campaign>(json.clone()).unwrap(),
            campaign
        );
        let mut blank = json;
        blank["name"] = serde_json::json!(" ");
        assert!(serde_json::from_value::<Campaign>(blank).is_err());
    }

    #[test]
    fn merged_metadata_lets_the_survey_win() {
        let entries = |pairs: &[(&str, &str)]| {
            Metadata::new(
                pairs
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            )
            .unwrap()
        };
        let merged = Metadata::merged(
            &entries(&[("source", "campaign"), ("zone", "nord")]),
            &entries(&[("source", "survey")]),
        );
        assert_eq!(merged.get("source"), Some("survey"));
        assert_eq!(merged.get("zone"), Some("nord"));
    }

    #[test]
    fn metadata_refuses_an_empty_key() {
        let entries = BTreeMap::from([(String::new(), "x".to_string())]);
        assert_eq!(Metadata::new(entries), Err(SurveyError::MetadataEmptyKey));
    }

    #[test]
    fn a_draft_cannot_open_without_a_graph() {
        assert_eq!(
            draft().open(at(1), Some(slug())).err(),
            Some(SurveyError::SurveyWithoutGraph)
        );
    }

    #[test]
    fn an_opened_survey_cannot_be_deserialized_without_a_graph() {
        let json = serde_json::to_string(&draft()).unwrap();
        assert!(serde_json::from_str::<Survey<Opened>>(&json).is_err());
        assert!(serde_json::from_str::<Survey<Draft>>(&json).is_ok());
    }

    #[test]
    fn an_opened_survey_survives_a_serde_round_trip() {
        let survey = opened();
        let json = serde_json::to_string(&survey).unwrap();
        assert_eq!(
            serde_json::from_str::<Survey<Opened>>(&json).unwrap(),
            survey
        );
    }

    #[test]
    fn a_survey_in_a_running_campaign_accepts_a_response() {
        assert_eq!(
            opened_in_campaign().accepts(Some(&campaign(Some(1), Some(9))), at(3)),
            Ok(())
        );
        assert_eq!(opened().accepts(None, at(3)), Ok(()));
    }

    #[test]
    fn a_survey_before_its_campaign_window_refuses() {
        assert_eq!(
            opened_in_campaign().accepts(Some(&campaign(Some(4), None)), at(3)),
            Err(SurveyError::CampaignNotStarted)
        );
    }

    #[test]
    fn a_survey_after_its_campaign_window_refuses() {
        assert_eq!(
            opened_in_campaign().accepts(Some(&campaign(None, Some(3))), at(3)),
            Err(SurveyError::CampaignEnded)
        );
    }

    #[test]
    fn a_survey_in_a_campaign_refuses_without_its_campaign() {
        assert_eq!(
            opened_in_campaign().accepts(None, at(3)),
            Err(SurveyError::SurveyCampaignMismatch)
        );
    }

    #[test]
    fn a_survey_refuses_a_campaign_that_is_not_its_own() {
        let other = Campaign::new(
            CampaignId(Uuid::from_u128(7)),
            org(),
            "Autre".to_string(),
            Window::new(None, None).unwrap(),
            Metadata::default(),
            at(1),
        )
        .unwrap();
        assert_eq!(
            opened_in_campaign().accepts(Some(&other), at(3)),
            Err(SurveyError::SurveyCampaignMismatch)
        );
        assert_eq!(
            opened().accepts(Some(&campaign(None, None)), at(3)),
            Err(SurveyError::SurveyCampaignMismatch)
        );
    }

    #[test]
    fn a_survey_refuses_a_campaign_of_another_organization() {
        let foreign = Campaign::new(
            CampaignId(Uuid::from_u128(2)),
            OrganizationId(Uuid::from_u128(901)),
            "Ailleurs".to_string(),
            Window::new(None, None).unwrap(),
            Metadata::default(),
            at(1),
        )
        .unwrap();
        assert_eq!(
            opened_in_campaign().accepts(Some(&foreign), at(3)),
            Err(SurveyError::SurveyCampaignOrganizationMismatch)
        );
    }

    #[test]
    fn an_opened_survey_exposes_its_definition() {
        assert_eq!(opened().definition(), &branching(true));
    }

    #[test]
    fn a_public_survey_needs_a_slug_to_open() {
        let mut survey = draft();
        survey.edit_graph(Some(branching(true)));
        assert_eq!(
            survey.open(at(1), None).err(),
            Some(SurveyError::SurveyPublicSlugRequired)
        );
    }

    #[test]
    fn a_members_survey_refuses_a_slug() {
        assert_eq!(
            members_draft().open(at(1), Some(slug())).err(),
            Some(SurveyError::SurveyPublicSlugForbidden)
        );
        let opened = members_draft().open(at(1), None).unwrap();
        assert_eq!(opened.public_slug(), None);
    }

    #[test]
    fn an_opened_and_a_closed_survey_carry_their_slug() {
        let opened = opened();
        assert_eq!(opened.public_slug(), Some(&slug()));
        let closed = opened.close(at(2)).unwrap();
        assert_eq!(closed.public_slug(), Some(&slug()));
    }

    #[test]
    fn a_slug_inconsistent_with_the_visibility_cannot_be_deserialized() {
        let mut json = serde_json::to_value(opened()).unwrap();
        json["public_slug"] = serde_json::Value::Null;
        assert!(serde_json::from_value::<Survey<Opened>>(json).is_err());
        let mut json = serde_json::to_value(members_draft().open(at(1), None).unwrap()).unwrap();
        json["public_slug"] = serde_json::json!("abcdefgh");
        assert!(serde_json::from_value::<Survey<Opened>>(json).is_err());
        let mut json = serde_json::to_value(draft()).unwrap();
        json["public_slug"] = serde_json::json!("abcdefgh");
        assert!(serde_json::from_value::<Survey<Draft>>(json).is_err());
    }

    #[test]
    fn a_closed_survey_refuses() {
        let closed: Survey<Closed> = opened().close(at(2)).unwrap();
        assert_eq!(closed.accepts(None, at(3)), Err(SurveyError::SurveyClosed));
        assert_eq!(closed.closed_at(), Some(at(2)));
    }

    #[test]
    fn a_survey_cannot_close_before_it_opened() {
        let before = at(1) - chrono::Duration::days(1);
        assert_eq!(
            opened().close(before).err(),
            Some(SurveyError::SurveyClosedBeforeOpened)
        );
    }

    #[test]
    fn a_closed_survey_survives_a_serde_round_trip() {
        let closed = opened().close(at(2)).unwrap();
        let json = serde_json::to_string(&closed).unwrap();
        assert_eq!(
            serde_json::from_str::<Survey<Closed>>(&json).unwrap(),
            closed
        );
    }

    #[test]
    fn a_draft_with_a_blank_title_cannot_be_deserialized() {
        let mut json = serde_json::to_value(draft()).unwrap();
        json["title"] = serde_json::json!("   ");
        assert!(serde_json::from_value::<Survey<Draft>>(json).is_err());
    }

    fn saved(
        survey: &Survey<Opened>,
        answers: Answers,
    ) -> Result<Response<InProgress>, SurveyError> {
        let mut response = response_to(survey);
        response.save(survey, answers)?;
        Ok(response)
    }

    fn one(kind: QuestionKind, options: &[&str], max_photos: Option<u8>) -> Graph {
        Graph::new(
            vec![
                Node::Question(QuestionNode {
                    id: qid(1),
                    kind,
                    label: "Q".to_string(),
                    required: true,
                    options: options.iter().map(|o| o.to_string()).collect(),
                    max_photos,
                    identifier: false,
                }),
                end(2),
            ],
            vec![edge(1, 2, Branch::Next)],
            qid(1),
        )
        .unwrap()
    }

    fn answer_one(graph: &Graph, answer: Answer) -> Result<(), SurveyError> {
        saved(&survey_of(graph.clone()), Answers::from([(qid(1), answer)])).map(|_| ())
    }

    fn photos(count: usize) -> Answer {
        Answer::Photos((0..count).map(|n| format!("p{n}")).collect())
    }

    #[test]
    fn every_question_kind_accepts_its_own_answer() {
        let day = chrono::NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
        let cases = [
            (QuestionKind::Text, vec![], text("a")),
            (QuestionKind::LongText, vec![], text("a\nb")),
            (QuestionKind::Number, vec![], Answer::Number(1.5)),
            (QuestionKind::SingleChoice, vec!["A", "B"], choice("B")),
            (
                QuestionKind::MultipleChoice,
                vec!["A", "B"],
                Answer::Choices(vec!["A".to_string(), "B".to_string()]),
            ),
            (QuestionKind::Date, vec![], Answer::Date(day)),
            (QuestionKind::Datetime, vec![], Answer::DateTime(at(1))),
            (
                QuestionKind::Phone,
                vec![],
                Answer::Phone(PhoneNumber::try_from("0612345678").unwrap()),
            ),
            (
                QuestionKind::Email,
                vec![],
                Answer::Email(EmailAddress::try_from("a@b.fr").unwrap()),
            ),
            (QuestionKind::Photo, vec![], photos(5)),
        ];
        for (kind, options, answer) in cases {
            assert_eq!(
                answer_one(&one(kind, &options, None), answer),
                Ok(()),
                "{kind:?}"
            );
        }
    }

    #[test]
    fn an_answer_of_the_wrong_kind_is_refused() {
        assert_eq!(
            answer_one(&one(QuestionKind::Text, &[], None), choice("zzz")),
            Err(SurveyError::AnswerKindMismatch(qid(1)))
        );
        assert_eq!(
            answer_one(&one(QuestionKind::Number, &[], None), text("1")),
            Err(SurveyError::AnswerKindMismatch(qid(1)))
        );
    }

    #[test]
    fn a_choice_outside_the_options_is_refused() {
        assert_eq!(
            answer_one(
                &one(QuestionKind::SingleChoice, &["A"], None),
                choice("zzz")
            ),
            Err(SurveyError::AnswerUnknownOption(qid(1)))
        );
        assert_eq!(
            answer_one(
                &one(QuestionKind::MultipleChoice, &["A"], None),
                Answer::Choices(vec!["A".to_string(), "zzz".to_string()])
            ),
            Err(SurveyError::AnswerUnknownOption(qid(1)))
        );
    }

    #[test]
    fn a_repeated_option_is_refused() {
        assert_eq!(
            answer_one(
                &one(QuestionKind::MultipleChoice, &["A", "B"], None),
                Answer::Choices(vec!["A".to_string(), "A".to_string()])
            ),
            Err(SurveyError::AnswerDuplicateOption(qid(1)))
        );
    }

    #[test]
    fn too_many_photos_are_refused() {
        assert_eq!(
            answer_one(&one(QuestionKind::Photo, &[], None), photos(6)),
            Err(SurveyError::AnswerTooManyPhotos(qid(1)))
        );
        assert_eq!(
            answer_one(&one(QuestionKind::Photo, &[], Some(2)), photos(3)),
            Err(SurveyError::AnswerTooManyPhotos(qid(1)))
        );
        assert_eq!(
            answer_one(&one(QuestionKind::Photo, &[], Some(2)), photos(2)),
            Ok(())
        );
    }

    #[test]
    fn an_empty_photo_is_refused() {
        assert_eq!(
            answer_one(
                &one(QuestionKind::Photo, &[], None),
                Answer::Photos(vec![String::new()])
            ),
            Err(SurveyError::AnswerEmptyPhoto(qid(1)))
        );
    }

    #[test]
    fn an_answer_to_an_unknown_question_is_refused_on_save_and_on_complete() {
        let graph = survey_of(branching(true));
        assert_eq!(
            saved(&graph, Answers::from([(qid(99), text("x"))])).err(),
            Some(SurveyError::AnswerUnknownQuestion(qid(99)))
        );
        assert_eq!(
            saved(&graph, Answers::from([(qid(2), text("x"))])).err(),
            Some(SurveyError::AnswerUnknownQuestion(qid(2)))
        );
        let mut json = serde_json::to_value(response()).unwrap();
        json["answers"] = serde_json::json!({ qid(99).to_string(): { "Text": "x" } });
        let response = serde_json::from_value::<Response<InProgress>>(json).unwrap();
        assert_eq!(
            response.complete(&graph, at(2)).err(),
            Some(SurveyError::AnswerUnknownQuestion(qid(99)))
        );
    }

    #[test]
    fn a_non_finite_number_is_refused_on_save() {
        let graph = one(QuestionKind::Number, &[], None);
        for bad in [f64::NAN, f64::INFINITY] {
            assert_eq!(
                answer_one(&graph, Answer::Number(bad)),
                Err(SurveyError::AnswerNotFinite(qid(1)))
            );
        }
    }

    #[test]
    fn a_response_cannot_complete_with_a_required_question_on_its_path_unanswered() {
        let graph = survey_of(branching(true));
        let response = saved(&graph, Answers::from([(qid(1), choice("Oui"))])).unwrap();
        assert_eq!(
            response.complete(&graph, at(2)).err(),
            Some(SurveyError::RequiredQuestionUnanswered(qid(3)))
        );
    }

    #[test]
    fn a_response_ignores_a_required_question_off_its_path() {
        let graph = survey_of(branching(true));
        let response = saved(
            &graph,
            Answers::from([(qid(1), choice("Oui")), (qid(3), text("ok"))]),
        )
        .unwrap();
        assert!(response.complete(&graph, at(2)).is_ok());
    }

    #[test]
    fn completion_discards_answers_off_the_path() {
        let graph = survey_of(branching(true));
        let response = saved(
            &graph,
            Answers::from([
                (qid(1), choice("Oui")),
                (qid(3), text("ok")),
                (qid(4), text("stale")),
            ]),
        )
        .unwrap();
        let completed: Response<Completed> = response.complete(&graph, at(2)).unwrap();
        assert_eq!(
            completed.answers().keys().copied().collect::<Vec<_>>(),
            vec![qid(1), qid(3)]
        );
        assert_eq!(completed.completed_at(), Some(at(2)));
    }

    fn identifying(kind: QuestionKind, second: bool) -> Graph {
        let mut nodes = vec![node(1, kind, true, !second)];
        let mut edges = vec![edge(1, 3, Branch::Next)];
        if second {
            nodes.push(node(2, kind, true, true));
            edges = vec![edge(1, 2, Branch::Next), edge(2, 3, Branch::Next)];
        }
        nodes.push(end(3));
        Graph::new(nodes, edges, qid(1)).unwrap()
    }

    fn identifier_of(graph: &Graph, answers: Answers) -> Option<String> {
        let survey = survey_of(graph.clone());
        let response = saved(&survey, answers).unwrap();
        let completed = response.complete(&survey, at(2)).unwrap();
        completed.identifier_value().map(str::to_string)
    }

    #[test]
    fn the_identifier_value_is_normalized_on_completion() {
        let graph = identifying(QuestionKind::Text, false);
        let value = identifier_of(&graph, Answers::from([(qid(1), text("  Jane DOE "))]));
        assert_eq!(value.as_deref(), Some("jane doe"));
    }

    fn optional_identifier() -> Graph {
        Graph::new(
            vec![node(1, QuestionKind::Text, false, true), end(3)],
            vec![edge(1, 3, Branch::Next)],
            qid(1),
        )
        .unwrap()
    }

    #[test]
    fn a_blank_identifier_counts_as_none() {
        assert_eq!(
            identifier_of(
                &optional_identifier(),
                Answers::from([(qid(1), text("   "))])
            ),
            None
        );
    }

    #[test]
    fn a_blank_required_identifier_refuses_completion() {
        let survey = survey_of(identifying(QuestionKind::Text, false));
        let response = saved(&survey, Answers::from([(qid(1), text("  "))])).unwrap();
        assert!(response.answers().is_empty());
        assert_eq!(
            response.complete(&survey, at(2)).err(),
            Some(SurveyError::RequiredQuestionUnanswered(qid(1)))
        );
    }

    #[test]
    fn a_blank_answer_is_dropped_at_save_for_every_blank_kind() {
        let cases = [
            (QuestionKind::Text, vec![], text("   ")),
            (QuestionKind::LongText, vec![], text("")),
            (
                QuestionKind::MultipleChoice,
                vec!["A"],
                Answer::Choices(vec![]),
            ),
            (QuestionKind::Photo, vec![], Answer::Photos(vec![])),
        ];
        for (kind, options, answer) in cases {
            let survey = survey_of(one(kind, &options, None));
            let response = saved(&survey, Answers::from([(qid(1), answer)])).unwrap();
            assert!(response.answers().is_empty(), "{kind:?}");
            assert_eq!(
                response.complete(&survey, at(2)).err(),
                Some(SurveyError::RequiredQuestionUnanswered(qid(1))),
                "{kind:?}"
            );
        }
    }

    #[test]
    fn a_stored_blank_answer_is_unanswered_at_completion() {
        let survey = survey_of(one(QuestionKind::Text, &[], None));
        let mut json = serde_json::to_value(response_to(&survey)).unwrap();
        json["answers"] = serde_json::json!({ qid(1).to_string(): { "Text": "  " } });
        let response = serde_json::from_value::<Response<InProgress>>(json).unwrap();
        assert_eq!(
            response.complete(&survey, at(2)).err(),
            Some(SurveyError::RequiredQuestionUnanswered(qid(1)))
        );
    }

    #[test]
    fn a_save_discards_answers_off_the_path() {
        let survey = survey_of(branching(true));
        let response = saved(
            &survey,
            Answers::from([
                (qid(1), choice("Oui")),
                (qid(3), text("ok")),
                (qid(4), text("stale")),
            ]),
        )
        .unwrap();
        assert_eq!(
            response.answers().keys().copied().collect::<Vec<_>>(),
            vec![qid(1), qid(3)]
        );
    }

    #[test]
    fn a_response_starts_on_its_survey() {
        let survey = opened();
        let response = response_to(&survey);
        assert_eq!(response.survey_id(), survey.id());
        assert_eq!(response.organization_id(), survey.organization_id());
        assert_eq!(response.resume_token(), Some(&token()));
        assert_eq!(response.member_id(), None);
    }

    #[test]
    fn a_public_survey_needs_a_token_and_refuses_a_member() {
        let survey = opened();
        let id = ResponseId(Uuid::from_u128(3));
        let member = Some(MemberId(Uuid::from_u128(4)));
        assert_eq!(
            Response::start(id, &survey, member, Some(token()), at(1)).err(),
            Some(SurveyError::ResponseMemberForbidden)
        );
        assert_eq!(
            Response::start(id, &survey, None, None, at(1)).err(),
            Some(SurveyError::ResponseResumeTokenRequired)
        );
    }

    #[test]
    fn a_members_survey_needs_a_member_and_refuses_a_token() {
        let survey = members_draft().open(at(1), None).unwrap();
        let id = ResponseId(Uuid::from_u128(3));
        let member = Some(MemberId(Uuid::from_u128(4)));
        assert_eq!(
            Response::start(id, &survey, None, None, at(1)).err(),
            Some(SurveyError::ResponseMemberRequired)
        );
        assert_eq!(
            Response::start(id, &survey, member, Some(token()), at(1)).err(),
            Some(SurveyError::ResponseResumeTokenForbidden)
        );
        let response = Response::start(id, &survey, member, None, at(1)).unwrap();
        assert_eq!(response.member_id(), member);
    }

    #[test]
    fn a_response_refuses_another_survey_on_save_and_on_complete() {
        let mut other = Survey::new(
            SurveyId(Uuid::from_u128(5)),
            org(),
            "Autre".to_string(),
            Visibility::Public,
        )
        .unwrap();
        other.edit_graph(Some(branching(true)));
        let other = other.open(at(1), Some(slug())).unwrap();
        let mut response = response();
        assert_eq!(
            response.save(&other, Answers::new()),
            Err(SurveyError::ResponseSurveyMismatch)
        );
        assert_eq!(
            response.complete(&other, at(2)).err(),
            Some(SurveyError::ResponseSurveyMismatch)
        );
    }

    #[test]
    fn a_completed_response_records_whether_its_respondent_is_unique() {
        let complete = |allow: bool| {
            let mut draft = draft();
            draft.edit_allow_multiple_responses(allow);
            draft.edit_graph(Some(optional_identifier()));
            let survey = draft.open(at(1), Some(slug())).unwrap();
            response_to(&survey).complete(&survey, at(2)).unwrap()
        };
        assert!(complete(false).unique_respondent());
        assert!(!complete(true).unique_respondent());
    }

    #[test]
    fn an_in_progress_response_cannot_be_deserialized_as_unique() {
        let mut json = serde_json::to_value(response()).unwrap();
        assert_eq!(json["unique_respondent"], serde_json::json!(false));
        json["unique_respondent"] = serde_json::json!(true);
        assert!(serde_json::from_value::<Response<InProgress>>(json).is_err());
    }

    #[test]
    fn only_the_flagged_question_gives_the_identifier() {
        let graph = identifying(QuestionKind::Text, true);
        let value = identifier_of(
            &graph,
            Answers::from([(qid(1), text("Not Me")), (qid(2), text(" Bar "))]),
        );
        assert_eq!(value.as_deref(), Some("bar"));
    }

    #[test]
    fn an_identifier_question_off_the_path_gives_no_value() {
        let graph = Graph::new(
            vec![
                node(1, QuestionKind::SingleChoice, true, false),
                Node::Condition(ConditionNode {
                    id: qid(2),
                    question: qid(1),
                    option: "Oui".to_string(),
                    checked: true,
                }),
                node(3, QuestionKind::Text, true, true),
                end(4),
            ],
            vec![
                edge(1, 2, Branch::Next),
                edge(2, 3, Branch::Then),
                edge(2, 4, Branch::Else),
                edge(3, 4, Branch::Next),
            ],
            qid(1),
        )
        .unwrap();
        let value = identifier_of(
            &graph,
            Answers::from([(qid(1), choice("Non")), (qid(3), text("x"))]),
        );
        assert_eq!(value, None);
    }

    #[test]
    fn a_phone_identifier_uses_its_normalized_form() {
        let graph = identifying(QuestionKind::Phone, false);
        let phone = PhoneNumber::try_from("+33 6 12 34 56 78").unwrap();
        let value = identifier_of(&graph, Answers::from([(qid(1), Answer::Phone(phone))]));
        assert_eq!(value.as_deref(), Some("+33612345678"));
    }

    #[test]
    fn an_email_identifier_uses_its_normalized_form() {
        let graph = identifying(QuestionKind::Email, false);
        let email = EmailAddress::try_from(" A@B.fr ").unwrap();
        let value = identifier_of(&graph, Answers::from([(qid(1), Answer::Email(email))]));
        assert_eq!(value.as_deref(), Some("a@b.fr"));
    }

    #[test]
    fn an_empty_resume_token_is_refused() {
        assert_eq!(
            ResumeToken::new(String::new()),
            Err(SurveyError::ResumeTokenEmpty)
        );
        assert!(serde_json::from_str::<ResumeToken>("\"\"").is_err());
    }

    #[test]
    fn the_graph_exposes_its_start_nodes_edges_and_questions() {
        let graph = branching(true);
        assert_eq!(graph.start(), qid(1));
        assert_eq!(graph.nodes().count(), 5);
        assert_eq!(graph.edges().count(), 5);
        assert!(graph.question(&qid(3)).is_some());
        assert!(graph.question(&qid(2)).is_none());
    }
}
