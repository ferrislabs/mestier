use serde_json::{Value, json};

use super::{ActionContext, failure};
use crate::domain::survey::aggregate::Stage;
use crate::domain::survey::events::answers_payload;
use crate::domain::survey::{
    CampaignId, Completed, Graph, PublicSlug, Response, Survey, SurveyId, SurveyRecord,
};
use crate::{ListSurveyResponsesCommand, OpenSurveyInCampaignCommand, SurveyCommand};

#[cfg(test)]
#[path = "tests_survey.rs"]
mod tests_survey;

const DEFAULT_LIMIT: u64 = 50;
const MAX_LIMIT: u64 = 100;

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
    let limit = match context.optional_whole_number("limit")? {
        None => DEFAULT_LIMIT,
        Some(0) => {
            return Err(format!(
                "`limit` must be between 1 and {MAX_LIMIT}: leave it empty to list {DEFAULT_LIMIT} responses"
            ));
        }
        Some(limit) => limit.min(MAX_LIMIT),
    };
    let offset = context.optional_whole_number("offset")?.unwrap_or(0);
    Ok((limit, offset))
}

fn survey_fields<S: Stage>(survey: &Survey<S>, status: &str, slug: Option<&PublicSlug>) -> Value {
    json!({
        "id": survey.id().0,
        "title": survey.title(),
        "status": status,
        "visibility": survey.visibility().as_str(),
        "campaign_ids": survey.campaign_ids().iter().map(|id| id.0).collect::<Vec<_>>(),
        "public_slug": slug.map(PublicSlug::as_str),
        "opened_at": survey.opened_at(),
        "closed_at": survey.closed_at(),
    })
}

fn completed_response(graph: &Graph, response: &Response<Completed>) -> Value {
    json!({
        "id": response.id().0,
        "member_id": response.member_id().map(|id| id.0),
        "completed_at": response.completed_at(),
        "identifier_value": response.identifier_value(),
        "graph_version": response.graph_version(),
        "answers": answers_payload(graph, response.answers()),
    })
}

async fn open(context: &ActionContext<'_>) -> Result<Value, String> {
    let command = survey_command(context)?;
    let campaign_id = context.optional_uuid("campaign_id")?.map(CampaignId);

    let opened = match campaign_id {
        Some(campaign_id) => context
            .usecase
            .open_survey_in_campaign(OpenSurveyInCampaignCommand {
                actor: command.actor,
                organization_id: command.organization_id,
                id: command.id,
                campaign_id,
            })
            .await
            .map_err(failure)?,
        None => context
            .usecase
            .open_survey(command)
            .await
            .map_err(failure)?,
    };

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
        "path": slug.path(),
    }))
}

async fn list_responses(context: &ActionContext<'_>) -> Result<Value, String> {
    let command = survey_command(context)?;
    let (limit, offset) = page(context)?;

    let listed = context
        .usecase
        .list_completed_survey_responses(ListSurveyResponsesCommand {
            actor: command.actor,
            organization_id: command.organization_id,
            survey_id: command.id,
            limit,
            offset,
        })
        .await
        .map_err(failure)?;
    let responses = listed
        .responses
        .iter()
        .map(|response| {
            listed
                .graph_of(response)
                .map(|graph| completed_response(graph, response))
                .map_err(|error| format!("response {}: {error}", response.id().0))
        })
        .collect::<Result<Vec<Value>, String>>()?;
    Ok(json!({
        "total": listed.total,
        "responses": responses,
    }))
}
