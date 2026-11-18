use events::{DomainEvent, EventSubject};
use serde_json::{Value, json};

use super::{
    Answer, Campaign, CampaignId, Completed, InProgress, Metadata, Opened, Response, Survey,
    SurveyError,
};

pub use super::descriptors::descriptors;
#[cfg(test)]
pub use super::descriptors::emitted_events;

fn merged_metadata(
    survey: &Survey<Opened>,
    campaign: Option<&Campaign>,
) -> Result<Metadata, SurveyError> {
    survey.check_campaign(campaign)?;
    Ok(campaign.map_or_else(
        || survey.metadata().clone(),
        |campaign| Metadata::merged(campaign.metadata(), survey.metadata()),
    ))
}

fn ensure_response<S>(survey: &Survey<Opened>, response: &Response<S>) -> Result<(), SurveyError> {
    if response.survey_id() == survey.id() {
        Ok(())
    } else {
        Err(SurveyError::ResponseSurveyMismatch)
    }
}

pub struct SurveyOpened<'a> {
    survey: &'a Survey<Opened>,
    metadata: Metadata,
}

impl<'a> SurveyOpened<'a> {
    pub fn new(
        survey: &'a Survey<Opened>,
        campaign: Option<&Campaign>,
    ) -> Result<Self, SurveyError> {
        Ok(Self {
            metadata: merged_metadata(survey, campaign)?,
            survey,
        })
    }
}

impl DomainEvent for SurveyOpened<'_> {
    fn name(&self) -> &'static str {
        "survey.opened"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("survey", self.survey.id().0)
    }

    fn payload(&self) -> Value {
        let survey = self.survey;
        json!({
            "survey_id": survey.id().0,
            "campaign_id": campaign_id(survey.campaign_id()),
            "metadata": metadata_payload(&self.metadata),
            "survey": {
                "id": survey.id().0,
                "organization_id": survey.organization_id().0,
                "title": survey.title(),
                "description": survey.description(),
                "visibility": survey.visibility().as_str(),
                "allow_multiple_responses": survey.allow_multiple_responses(),
                "response_cap": survey.response_cap().map(|cap| cap.get()),
                "opened_at": survey.opened_at(),
            },
        })
    }
}

pub struct SurveyResponseSubmitted<'a> {
    survey: &'a Survey<Opened>,
    response: &'a Response<InProgress>,
    metadata: Metadata,
}

impl<'a> SurveyResponseSubmitted<'a> {
    pub fn new(
        survey: &'a Survey<Opened>,
        campaign: Option<&Campaign>,
        response: &'a Response<InProgress>,
    ) -> Result<Self, SurveyError> {
        ensure_response(survey, response)?;
        Ok(Self {
            metadata: merged_metadata(survey, campaign)?,
            survey,
            response,
        })
    }
}

impl DomainEvent for SurveyResponseSubmitted<'_> {
    fn name(&self) -> &'static str {
        "survey.response_submitted"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("survey_response", self.response.id().0)
    }

    fn payload(&self) -> Value {
        let response = self.response;
        json!({
            "survey_id": self.survey.id().0,
            "campaign_id": campaign_id(self.survey.campaign_id()),
            "metadata": metadata_payload(&self.metadata),
            "response": {
                "id": response.id().0,
                "member_id": response.member_id().map(|id| id.0),
                "status": "IN_PROGRESS",
            },
            "answered_questions": response.answers().len(),
        })
    }
}

pub struct SurveyResponseCompleted<'a> {
    survey: &'a Survey<Opened>,
    response: &'a Response<Completed>,
    metadata: Metadata,
}

impl<'a> SurveyResponseCompleted<'a> {
    pub fn new(
        survey: &'a Survey<Opened>,
        campaign: Option<&Campaign>,
        response: &'a Response<Completed>,
    ) -> Result<Self, SurveyError> {
        ensure_response(survey, response)?;
        Ok(Self {
            metadata: merged_metadata(survey, campaign)?,
            survey,
            response,
        })
    }

    fn answers_payload(&self) -> Vec<Value> {
        let graph = self.survey.definition();
        let answers = self.response.answers();
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
}

impl DomainEvent for SurveyResponseCompleted<'_> {
    fn name(&self) -> &'static str {
        "survey.response_completed"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("survey_response", self.response.id().0)
    }

    fn payload(&self) -> Value {
        let response = self.response;
        json!({
            "survey_id": self.survey.id().0,
            "campaign_id": campaign_id(self.survey.campaign_id()),
            "survey": { "id": self.survey.id().0, "title": self.survey.title() },
            "response": {
                "id": response.id().0,
                "member_id": response.member_id().map(|id| id.0),
                "completed_at": response.completed_at(),
                "identifier_value": response.identifier_value(),
            },
            "metadata": metadata_payload(&self.metadata),
            "answers": self.answers_payload(),
        })
    }
}

pub struct SurveyCampaignStarted {
    pub campaign: Campaign,
}

impl DomainEvent for SurveyCampaignStarted {
    fn name(&self) -> &'static str {
        "survey_campaign.started"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("survey_campaign", self.campaign.id().0)
    }

    fn payload(&self) -> Value {
        campaign_payload(&self.campaign)
    }
}

pub struct SurveyCampaignEnded {
    pub campaign: Campaign,
}

impl DomainEvent for SurveyCampaignEnded {
    fn name(&self) -> &'static str {
        "survey_campaign.ended"
    }

    fn version(&self) -> u16 {
        1
    }

    fn subject(&self) -> EventSubject {
        EventSubject::new("survey_campaign", self.campaign.id().0)
    }

    fn payload(&self) -> Value {
        campaign_payload(&self.campaign)
    }
}

fn campaign_id(id: Option<CampaignId>) -> Value {
    id.map_or(Value::Null, |id| json!(id.0))
}

fn metadata_payload(metadata: &Metadata) -> Value {
    Value::Object(
        metadata
            .iter()
            .map(|(key, value)| (key.to_string(), json!(value)))
            .collect(),
    )
}

fn campaign_payload(campaign: &Campaign) -> Value {
    json!({
        "campaign_id": campaign.id().0,
        "campaign": {
            "id": campaign.id().0,
            "organization_id": campaign.organization_id().0,
            "name": campaign.name(),
            "starts_at": campaign.window().starts_at(),
            "ends_at": campaign.window().ends_at(),
            "metadata": metadata_payload(campaign.metadata()),
        },
    })
}

pub(crate) fn answer_value(answer: &Answer) -> Value {
    match answer {
        Answer::Text(text) | Answer::Choice(text) => json!(text),
        Answer::Number(number) => json!(number),
        Answer::Choices(values) | Answer::Photos(values) => json!(values),
        Answer::Date(date) => json!(date.format("%Y-%m-%d").to_string()),
        Answer::DateTime(at) => json!(at),
        Answer::Phone(phone) => json!(phone.as_str()),
        Answer::Email(email) => json!(email.as_str()),
    }
}
