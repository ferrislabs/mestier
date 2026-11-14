use chrono::{DateTime, NaiveDate, Utc};
use mestier_core::{QuestionId, QuestionKind};
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SurveyGraphSchema {
    pub start: QuestionId,
    pub nodes: Vec<SurveyNodeSchema>,
    pub edges: Vec<SurveyEdgeSchema>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SurveyNodeSchema {
    Question(SurveyQuestionNodeSchema),
    Condition(SurveyConditionNodeSchema),
    End(SurveyEndNodeSchema),
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SurveyQuestionNodeSchema {
    pub id: QuestionId,
    pub kind: QuestionKind,
    pub label: String,
    pub required: bool,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub max_photos: Option<u8>,
    #[serde(default)]
    pub identifier: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SurveyConditionNodeSchema {
    pub id: QuestionId,
    pub question: QuestionId,
    pub option: String,
    pub checked: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SurveyEndNodeSchema {
    pub id: QuestionId,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SurveyEdgeSchema {
    pub from: QuestionId,
    pub to: QuestionId,
    pub branch: SurveyBranchSchema,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SurveyBranchSchema {
    Next,
    Then,
    Else,
}

#[derive(Debug, Deserialize, ToSchema)]
pub enum SurveyAnswerSchema {
    Text(String),
    Number(f64),
    Choice(String),
    Choices(Vec<String>),
    Date(NaiveDate),
    DateTime(DateTime<Utc>),
    Phone(String),
    Email(String),
    Photos(Vec<String>),
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use mestier_core::{
        Answer, Answers, SurveyBranch, SurveyConditionNode, SurveyEdge, SurveyEndNode, SurveyGraph,
        SurveyNode, SurveyQuestionNode,
    };
    use uuid::Uuid;

    use super::*;

    fn qid(n: u128) -> QuestionId {
        QuestionId(Uuid::from_u128(n))
    }

    fn graph() -> SurveyGraph {
        SurveyGraph::new(
            vec![
                SurveyNode::Question(SurveyQuestionNode {
                    id: qid(1),
                    kind: QuestionKind::SingleChoice,
                    label: "Le stage vous a-t-il plu ?".to_owned(),
                    required: true,
                    options: vec!["Oui".to_owned(), "Non".to_owned()],
                    max_photos: None,
                    identifier: false,
                }),
                SurveyNode::Condition(SurveyConditionNode {
                    id: qid(2),
                    question: qid(1),
                    option: "Non".to_owned(),
                    checked: true,
                }),
                SurveyNode::Question(SurveyQuestionNode {
                    id: qid(3),
                    kind: QuestionKind::Photo,
                    label: "Photos".to_owned(),
                    required: false,
                    options: Vec::new(),
                    max_photos: Some(3),
                    identifier: false,
                }),
                SurveyNode::End(SurveyEndNode {
                    id: qid(4),
                    message: Some("Merci".to_owned()),
                }),
            ],
            vec![
                SurveyEdge {
                    from: qid(1),
                    to: qid(2),
                    branch: SurveyBranch::Next,
                },
                SurveyEdge {
                    from: qid(2),
                    to: qid(3),
                    branch: SurveyBranch::Then,
                },
                SurveyEdge {
                    from: qid(2),
                    to: qid(4),
                    branch: SurveyBranch::Else,
                },
                SurveyEdge {
                    from: qid(3),
                    to: qid(4),
                    branch: SurveyBranch::Next,
                },
            ],
            qid(1),
        )
        .unwrap()
    }

    #[test]
    fn a_serialised_domain_graph_reads_as_the_documented_graph() {
        let json = serde_json::to_value(graph()).unwrap();
        let read: SurveyGraphSchema = serde_json::from_value(json.clone()).unwrap();

        assert_eq!(read.start, qid(1), "{json}");
        assert_eq!(read.nodes.len(), 4, "{json}");
        assert!(matches!(read.nodes[1], SurveyNodeSchema::Condition(_)));
        assert!(matches!(read.nodes[3], SurveyNodeSchema::End(_)));
        let branches: Vec<SurveyBranchSchema> = read.edges.iter().map(|edge| edge.branch).collect();
        assert_eq!(
            branches,
            vec![
                SurveyBranchSchema::Next,
                SurveyBranchSchema::Then,
                SurveyBranchSchema::Else,
                SurveyBranchSchema::Next,
            ]
        );
    }

    #[test]
    fn serialised_domain_answers_read_as_the_documented_answers() {
        let answers: Answers = BTreeMap::from([
            (qid(1), Answer::Text("Bonjour".to_owned())),
            (qid(2), Answer::Number(4.5)),
            (qid(3), Answer::Choice("Oui".to_owned())),
            (
                qid(4),
                Answer::Choices(vec!["A".to_owned(), "B".to_owned()]),
            ),
            (
                qid(5),
                Answer::Date(NaiveDate::from_ymd_opt(2026, 3, 1).unwrap()),
            ),
            (qid(6), Answer::DateTime(Utc::now())),
            (
                qid(7),
                Answer::Phone(serde_json::from_value(serde_json::json!("+33612345678")).unwrap()),
            ),
            (
                qid(8),
                Answer::Email(
                    serde_json::from_value(serde_json::json!("jane@example.com")).unwrap(),
                ),
            ),
            (qid(9), Answer::Photos(vec!["photo-key".to_owned()])),
        ]);
        let json = serde_json::to_value(&answers).unwrap();
        let read: BTreeMap<QuestionId, SurveyAnswerSchema> =
            serde_json::from_value(json.clone()).unwrap();

        assert_eq!(read.len(), 9, "{json}");
        assert!(
            matches!(read[&qid(5)], SurveyAnswerSchema::Date(_)),
            "{json}"
        );
        assert!(
            matches!(read[&qid(6)], SurveyAnswerSchema::DateTime(_)),
            "{json}"
        );
        assert!(
            matches!(read[&qid(8)], SurveyAnswerSchema::Email(_)),
            "{json}"
        );
    }
}
