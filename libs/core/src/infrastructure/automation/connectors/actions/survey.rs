use serde_json::{Value, json};

use super::{ActionContext, failure};
use crate::domain::survey::aggregate::Stage;
use crate::domain::survey::events::answer_value;
use crate::domain::survey::{
    Answers, CampaignId, Completed, Graph, PublicSlug, Response, ResponseRecord, Survey, SurveyId,
    SurveyRecord,
};
use crate::{ListSurveyResponsesCommand, SurveyCommand, SurveyPatch, UpdateSurveyCommand};

#[cfg(test)]
#[path = "tests_survey.rs"]
mod tests_survey;

const DEFAULT_LIMIT: u64 = 50;
const MAX_LIMIT: u64 = 100;
const SCAN_BATCH: u64 = 100;

pub(super) async fn run(kind: &str, context: &ActionContext<'_>) -> Option<Result<Value, String>> {
    Some(match kind {
        "mestier.survey.open" => open(context).await,
        "mestier.survey.close" => close(context).await,
        "mestier.survey.get_link" => get_link(context).await,
        "mestier.survey.list_responses" => list_responses(context).await,
        _ => return None,
    })
}

fn survey_command(context: &ActionContext<'_>) -> Result<SurveyCommand, String> {
    Ok(SurveyCommand {
        actor: context.actor(),
        organization_id: context.organization_id,
        id: SurveyId(context.uuid("survey_id")?),
    })
}

fn page(context: &ActionContext<'_>) -> Result<(u64, u64), String> {
    let limit = context
        .optional_whole_number("limit")?
        .unwrap_or(DEFAULT_LIMIT)
        .clamp(1, MAX_LIMIT);
    let offset = context.optional_whole_number("offset")?.unwrap_or(0);
    Ok((limit, offset))
}

pub(super) fn public_path(slug: &PublicSlug) -> String {
    format!("/s/{}", slug.as_str())
}

fn survey_fields<S: Stage>(survey: &Survey<S>, status: &str, slug: Option<&PublicSlug>) -> Value {
    json!({
        "id": survey.id().0,
        "title": survey.title(),
        "status": status,
        "visibility": survey.visibility().as_str(),
        "campaign_id": survey.campaign_id().map(|id| id.0),
        "public_slug": slug.map(PublicSlug::as_str),
        "opened_at": survey.opened_at(),
        "closed_at": survey.closed_at(),
    })
}

pub(super) fn answers_payload(graph: &Graph, answers: &Answers) -> Vec<Value> {
    graph
        .path(answers)
        .questions
        .iter()
        .filter_map(|id| graph.question(id))
        .map(|question| {
            json!({
                "question_id": question.id.0,
                "label": question.label,
                "kind": question.kind.as_str(),
                "value": answers.get(&question.id).map_or(Value::Null, answer_value),
            })
        })
        .collect()
}

fn completed_response(graph: Option<&Graph>, response: &Response<Completed>) -> Value {
    json!({
        "id": response.id().0,
        "member_id": response.member_id().map(|id| id.0),
        "completed_at": response.completed_at(),
        "identifier_value": response.identifier_value(),
        "answers": graph.map_or_else(Vec::new, |graph| answers_payload(graph, response.answers())),
    })
}

async fn open(context: &ActionContext<'_>) -> Result<Value, String> {
    let command = survey_command(context)?;
    let campaign_id = context.optional_uuid("campaign_id")?.map(CampaignId);

    if let Some(campaign_id) = campaign_id {
        context
            .usecase
            .update_survey(UpdateSurveyCommand {
                actor: context.actor(),
                organization_id: context.organization_id,
                id: command.id,
                patch: SurveyPatch::default(),
                campaign_id: Some(Some(campaign_id)),
            })
            .await
            .map_err(failure)?;
    }
    let opened = context
        .usecase
        .open_survey(command)
        .await
        .map_err(failure)?;

    Ok(survey_fields(&opened, "OPENED", opened.public_slug()))
}

async fn close(context: &ActionContext<'_>) -> Result<Value, String> {
    let command = survey_command(context)?;

    let closed = context
        .usecase
        .close_survey(command)
        .await
        .map_err(failure)?;

    Ok(survey_fields(&closed, "CLOSED", closed.public_slug()))
}

async fn get_link(context: &ActionContext<'_>) -> Result<Value, String> {
    let command = survey_command(context)?;

    let record = context.usecase.get_survey(command).await.map_err(failure)?;
    let (status, slug) = match &record {
        SurveyRecord::Draft(_) => {
            return Err("the survey is still a draft: open it before reading its link".to_owned());
        }
        SurveyRecord::Opened(survey) => ("OPENED", survey.public_slug()),
        SurveyRecord::Closed(survey) => ("CLOSED", survey.public_slug()),
    };
    let slug = slug.ok_or_else(|| {
        "a MEMBERS survey has no public link: its members answer it from the application".to_owned()
    })?;

    Ok(json!({
        "survey_id": record.id().0,
        "status": status,
        "public_slug": slug.as_str(),
        "path": public_path(slug),
    }))
}

async fn list_responses(context: &ActionContext<'_>) -> Result<Value, String> {
    let command = survey_command(context)?;
    let (limit, offset) = page(context)?;
    let survey_id = command.id;

    let record = context.usecase.get_survey(command).await.map_err(failure)?;
    let graph = match &record {
        SurveyRecord::Draft(survey) => survey.graph(),
        SurveyRecord::Opened(survey) => survey.graph(),
        SurveyRecord::Closed(survey) => survey.graph(),
    };
    let mut skipped = 0;
    let mut listed = Vec::new();
    let mut cursor = 0;
    loop {
        let (batch, total) = context
            .usecase
            .list_survey_responses(ListSurveyResponsesCommand {
                actor: context.actor(),
                organization_id: context.organization_id,
                survey_id,
                limit: SCAN_BATCH,
                offset: cursor,
            })
            .await
            .map_err(failure)?;
        let fetched = batch.len() as u64;
        for response in batch {
            let ResponseRecord::Completed(response) = response else {
                continue;
            };
            if skipped < offset {
                skipped += 1;
                continue;
            }
            listed.push(completed_response(graph, &response));
            if listed.len() as u64 == limit {
                return Ok(Value::Array(listed));
            }
        }
        cursor += fetched;
        if fetched < SCAN_BATCH || cursor >= total {
            return Ok(Value::Array(listed));
        }
    }
}
