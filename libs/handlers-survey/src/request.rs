use std::collections::BTreeMap;

use common::CoreError;
use handlers::ApiError;
use mestier_core::{
    Answers, Metadata, QuestionId, SurveyEdge, SurveyError, SurveyGraph, SurveyNode,
};
use serde::{Deserialize, Deserializer};

pub(crate) fn deserialize_present<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Deserialize::deserialize(deserializer).map(Some)
}

pub(crate) fn refused(error: SurveyError) -> ApiError {
    CoreError::from(error).into()
}

pub(crate) fn metadata(entries: Option<BTreeMap<String, String>>) -> Result<Metadata, ApiError> {
    Metadata::new(entries.unwrap_or_default()).map_err(refused)
}

#[derive(Deserialize)]
struct GraphBody {
    nodes: Vec<SurveyNode>,
    edges: Vec<SurveyEdge>,
    start: QuestionId,
}

pub(crate) fn graph(value: serde_json::Value) -> Result<SurveyGraph, ApiError> {
    let body: GraphBody = serde_json::from_value(value)
        .map_err(|error| ApiError::UnprocessableEntity(error.to_string()))?;
    SurveyGraph::new(body.nodes, body.edges, body.start).map_err(refused)
}

pub(crate) fn answers(value: serde_json::Value) -> Result<Answers, ApiError> {
    serde_json::from_value(value).map_err(|error| ApiError::UnprocessableEntity(error.to_string()))
}

#[cfg(test)]
mod tests {
    use axum::response::IntoResponse;

    use super::*;

    fn status(error: ApiError) -> u16 {
        error.into_response().status().as_u16()
    }

    #[test]
    fn a_graph_that_breaks_the_domain_rules_is_unprocessable() {
        let refused = graph(
            serde_json::json!({"nodes": [], "edges": [], "start": "00000000-0000-0000-0000-000000000000"}),
        );
        let Err(error) = refused else {
            panic!("an empty graph must be refused");
        };
        assert!(error.to_string().contains("graph: has no node"), "{error}");
        assert_eq!(status(error), 422);
    }

    #[tokio::test]
    async fn a_graph_refusal_names_its_reason_and_question() {
        let lone = "00000000-0000-0000-0000-000000000001";
        let refused = graph(serde_json::json!({
            "nodes": [
                {"type": "QUESTION", "id": lone, "kind": "TEXT", "label": "A", "required": true, "block": "00000000-0000-0000-0000-000000000064"},
                {"type": "END", "id": "00000000-0000-0000-0000-000000000002"}
            ],
            "edges": [{"from": lone, "to": "00000000-0000-0000-0000-000000000002", "branch": "NEXT"}],
            "start": lone
        }));
        let Err(error) = refused else {
            panic!("a block of one question must be refused");
        };
        let response = error.into_response();
        assert_eq!(response.status().as_u16(), 422);
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap_or_default();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_default();
        assert_eq!(
            body["details"],
            serde_json::json!({"reason": "graph_block_too_small", "question_id": lone}),
            "{body}"
        );
    }

    #[test]
    fn an_answer_with_an_invalid_email_is_unprocessable() {
        let refused = answers(serde_json::json!({
            "00000000-0000-0000-0000-000000000001": {"Email": "not an email"}
        }));
        let Err(error) = refused else {
            panic!("an invalid email must be refused");
        };
        assert_eq!(status(error), 422);
    }

    #[test]
    fn answers_keyed_by_question_are_read_in_the_domain_form() {
        let read = answers(serde_json::json!({
            "00000000-0000-0000-0000-000000000001": {"Text": "Bonjour"}
        }));
        assert_eq!(read.map(|answers| answers.len()).ok(), Some(1));
    }

    #[test]
    fn blank_metadata_values_are_unprocessable() {
        let refused = metadata(Some(BTreeMap::from([(
            "classe".to_owned(),
            " ".to_owned(),
        )])));
        let Err(error) = refused else {
            panic!("a blank metadata value must be refused");
        };
        assert_eq!(status(error), 422);
    }

    #[test]
    fn absent_metadata_is_empty() {
        assert_eq!(metadata(None).ok(), Some(Metadata::default()));
    }
}
