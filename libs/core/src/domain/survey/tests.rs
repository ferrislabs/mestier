use proptest::prelude::*;

use super::{EmailAddress, PhoneNumber, SurveyError};

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
        let json = r#"{"nodes":[],"edges":[],"start":"00000000-0000-0000-0000-000000000001"}"#;
        assert!(serde_json::from_str::<Graph>(json).is_err());
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
