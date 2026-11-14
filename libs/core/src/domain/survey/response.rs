use std::marker::PhantomData;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::error::SurveyError;
use super::graph::{Answers, Graph};
use super::value::Answer;
use super::{ResponseId, SurveyId};
use crate::{MemberId, OrganizationId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InProgress;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Completed;

pub trait Phase {
    fn check(body: &ResponseBody) -> Result<(), SurveyError>;
}

impl Phase for InProgress {
    fn check(body: &ResponseBody) -> Result<(), SurveyError> {
        if body.completed_at.is_none() && body.identifier_value.is_none() {
            check_finite(&body.answers)
        } else {
            Err(SurveyError::ResponseInconsistentState)
        }
    }
}

impl Phase for Completed {
    fn check(body: &ResponseBody) -> Result<(), SurveyError> {
        if body.completed_at.is_some() {
            check_finite(&body.answers)
        } else {
            Err(SurveyError::ResponseInconsistentState)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct ResumeToken(String);

impl ResumeToken {
    pub fn new(value: String) -> Result<Self, SurveyError> {
        if value.is_empty() {
            Err(SurveyError::ResumeTokenEmpty)
        } else {
            Ok(Self(value))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ResumeToken {
    type Error = SurveyError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseBody {
    id: ResponseId,
    survey_id: SurveyId,
    organization_id: OrganizationId,
    member_id: Option<MemberId>,
    resume_token: Option<ResumeToken>,
    answers: Answers,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    identifier_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Response<S> {
    body: ResponseBody,
    phase: PhantomData<S>,
}

impl<S> Serialize for Response<S> {
    fn serialize<T: Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
        self.body.serialize(serializer)
    }
}

impl<'de, S: Phase> Deserialize<'de> for Response<S> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let body = ResponseBody::deserialize(deserializer)?;
        S::check(&body).map_err(serde::de::Error::custom)?;
        Ok(Self {
            body,
            phase: PhantomData,
        })
    }
}

fn check_finite(answers: &Answers) -> Result<(), SurveyError> {
    for (id, answer) in answers {
        if matches!(answer, Answer::Number(number) if !number.is_finite()) {
            return Err(SurveyError::AnswerNotFinite(*id));
        }
    }
    Ok(())
}

fn identifier_of(
    graph: &Graph,
    questions: &[super::QuestionId],
    answers: &Answers,
) -> Option<String> {
    let identifier = questions
        .iter()
        .find(|id| graph.question(id).is_some_and(|node| node.identifier))?;
    let value = match answers.get(identifier)? {
        Answer::Email(email) => email.as_str().to_string(),
        Answer::Phone(phone) => phone.as_str().to_string(),
        Answer::Text(text) => text.trim().to_lowercase(),
        _ => return None,
    };
    (!value.is_empty()).then_some(value)
}

impl<S> Response<S> {
    pub fn id(&self) -> ResponseId {
        self.body.id
    }

    pub fn survey_id(&self) -> SurveyId {
        self.body.survey_id
    }

    pub fn organization_id(&self) -> OrganizationId {
        self.body.organization_id
    }

    pub fn member_id(&self) -> Option<MemberId> {
        self.body.member_id
    }

    pub fn resume_token(&self) -> Option<&ResumeToken> {
        self.body.resume_token.as_ref()
    }

    pub fn answers(&self) -> &Answers {
        &self.body.answers
    }

    pub fn started_at(&self) -> DateTime<Utc> {
        self.body.started_at
    }

    pub fn completed_at(&self) -> Option<DateTime<Utc>> {
        self.body.completed_at
    }
}

impl Response<InProgress> {
    pub fn start(
        id: ResponseId,
        survey_id: SurveyId,
        organization_id: OrganizationId,
        member_id: Option<MemberId>,
        resume_token: Option<ResumeToken>,
        at: DateTime<Utc>,
    ) -> Self {
        Self {
            body: ResponseBody {
                id,
                survey_id,
                organization_id,
                member_id,
                resume_token,
                answers: Answers::new(),
                started_at: at,
                completed_at: None,
                identifier_value: None,
            },
            phase: PhantomData,
        }
    }

    pub fn save(&mut self, answers: Answers) -> Result<(), SurveyError> {
        check_finite(&answers)?;
        self.body.answers = answers;
        Ok(())
    }

    pub fn complete(
        mut self,
        graph: &Graph,
        at: DateTime<Utc>,
    ) -> Result<Response<Completed>, SurveyError> {
        let path = graph.path(&self.body.answers);
        for id in &path.questions {
            let required = graph.question(id).is_some_and(|node| node.required);
            if required && !self.body.answers.contains_key(id) {
                return Err(SurveyError::RequiredQuestionUnanswered(*id));
            }
        }
        self.body
            .answers
            .retain(|id, _| path.questions.contains(id));
        check_finite(&self.body.answers)?;
        self.body.identifier_value = identifier_of(graph, &path.questions, &self.body.answers);
        self.body.completed_at = Some(at);
        Ok(Response {
            body: self.body,
            phase: PhantomData,
        })
    }
}

impl Response<Completed> {
    pub fn identifier_value(&self) -> Option<&str> {
        self.body.identifier_value.as_deref()
    }
}
