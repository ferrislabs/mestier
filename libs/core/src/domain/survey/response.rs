use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::aggregate::{GraphVersion, Opened, Survey};
use super::error::SurveyError;
use super::graph::{Answers, Graph};
use super::validation::{check_finite, retain_valid, validate};
use super::value::{Answer, Visibility};
use super::{ResponseId, SurveyId};
use crate::{MemberId, OrganizationId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InProgress;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Completed;

mod sealed {
    use std::fmt::Debug;

    use super::{GraphVersion, ResponseBody, SurveyError};

    pub trait Sealed {
        type Version: Debug + Clone + Copy + PartialEq;

        fn check(
            body: &ResponseBody,
            version: Option<GraphVersion>,
        ) -> Result<Self::Version, SurveyError>;

        fn stored(version: Self::Version) -> Option<GraphVersion>;
    }
}

pub trait Phase: sealed::Sealed {}

impl Phase for InProgress {}
impl Phase for Completed {}

impl sealed::Sealed for InProgress {
    type Version = ();

    fn check(body: &ResponseBody, version: Option<GraphVersion>) -> Result<(), SurveyError> {
        if body.completed_at.is_none()
            && body.identifier_value.is_none()
            && !body.unique_respondent
            && version.is_none()
        {
            check_finite(&body.answers)
        } else {
            Err(SurveyError::ResponseInconsistentState)
        }
    }

    fn stored((): ()) -> Option<GraphVersion> {
        None
    }
}

impl sealed::Sealed for Completed {
    type Version = GraphVersion;

    fn check(
        body: &ResponseBody,
        version: Option<GraphVersion>,
    ) -> Result<GraphVersion, SurveyError> {
        match version {
            Some(version) if body.completed_at.is_some() => {
                check_finite(&body.answers)?;
                Ok(version)
            }
            _ => Err(SurveyError::ResponseInconsistentState),
        }
    }

    fn stored(version: GraphVersion) -> Option<GraphVersion> {
        Some(version)
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
#[doc(hidden)]
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
    unique_respondent: bool,
}

#[derive(Serialize)]
struct ResponseView<'a> {
    #[serde(flatten)]
    body: &'a ResponseBody,
    graph_version: Option<GraphVersion>,
}

#[derive(Deserialize)]
struct ResponseRow {
    #[serde(flatten)]
    body: ResponseBody,
    graph_version: Option<GraphVersion>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Response<S: Phase> {
    body: ResponseBody,
    version: S::Version,
}

impl<S: Phase> Serialize for Response<S> {
    fn serialize<T: Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
        ResponseView {
            body: &self.body,
            graph_version: S::stored(self.version),
        }
        .serialize(serializer)
    }
}

impl<'de, S: Phase> Deserialize<'de> for Response<S> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let ResponseRow {
            body,
            graph_version,
        } = ResponseRow::deserialize(deserializer)?;
        let version = S::check(&body, graph_version).map_err(serde::de::Error::custom)?;
        Ok(Self { body, version })
    }
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

impl<S: Phase> Response<S> {
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

    pub fn photo_directory(&self) -> String {
        photo_directory(self.body.organization_id, self.body.survey_id, self.body.id)
    }
}

pub fn photo_directory(
    organization_id: OrganizationId,
    survey_id: SurveyId,
    response_id: ResponseId,
) -> String {
    format!(
        "survey-{}/{}/{}",
        organization_id.0, survey_id.0, response_id.0
    )
}

fn check_respondent(
    visibility: Visibility,
    member_id: Option<MemberId>,
    resume_token: Option<&ResumeToken>,
) -> Result<(), SurveyError> {
    match (visibility, member_id, resume_token) {
        (Visibility::Members, None, _) => Err(SurveyError::ResponseMemberRequired),
        (Visibility::Members, Some(_), Some(_)) => Err(SurveyError::ResponseResumeTokenForbidden),
        (Visibility::Public, Some(_), _) => Err(SurveyError::ResponseMemberForbidden),
        (Visibility::Public, None, None) => Err(SurveyError::ResponseResumeTokenRequired),
        _ => Ok(()),
    }
}

fn drop_blank(answers: &mut Answers) {
    answers.retain(|_, answer| !answer.is_blank());
}

impl Response<InProgress> {
    pub fn start(
        id: ResponseId,
        survey: &Survey<Opened>,
        member_id: Option<MemberId>,
        resume_token: Option<ResumeToken>,
        at: DateTime<Utc>,
    ) -> Result<Self, SurveyError> {
        check_respondent(survey.visibility(), member_id, resume_token.as_ref())?;
        Ok(Self {
            body: ResponseBody {
                id,
                survey_id: survey.id(),
                organization_id: survey.organization_id(),
                member_id,
                resume_token,
                answers: Answers::new(),
                started_at: at,
                completed_at: None,
                identifier_value: None,
                unique_respondent: false,
            },
            version: (),
        })
    }

    fn ensure_survey(&self, survey: &Survey<Opened>) -> Result<(), SurveyError> {
        if survey.id() == self.body.survey_id {
            Ok(())
        } else {
            Err(SurveyError::ResponseSurveyMismatch)
        }
    }

    pub fn save(
        &mut self,
        survey: &Survey<Opened>,
        mut answers: Answers,
    ) -> Result<(), SurveyError> {
        self.ensure_survey(survey)?;
        let graph = survey.definition();
        validate(graph, &answers, &self.photo_directory())?;
        drop_blank(&mut answers);
        let path = graph.path(&answers);
        answers.retain(|id, _| path.questions.contains(id));
        self.body.answers = answers;
        Ok(())
    }

    pub fn complete(
        mut self,
        survey: &Survey<Opened>,
        at: DateTime<Utc>,
    ) -> Result<Response<Completed>, SurveyError> {
        self.ensure_survey(survey)?;
        let graph = survey.definition();
        let directory = self.photo_directory();
        retain_valid(graph, &mut self.body.answers, &directory);
        drop_blank(&mut self.body.answers);
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
        validate(graph, &self.body.answers, &directory)?;
        self.body.identifier_value = identifier_of(graph, &path.questions, &self.body.answers);
        self.body.unique_respondent = !survey.allow_multiple_responses();
        self.body.completed_at = Some(at);
        Ok(Response {
            body: self.body,
            version: survey.version(),
        })
    }
}

impl Response<Completed> {
    pub fn identifier_value(&self) -> Option<&str> {
        self.body.identifier_value.as_deref()
    }

    pub fn unique_respondent(&self) -> bool {
        self.body.unique_respondent
    }

    pub fn graph_version(&self) -> GraphVersion {
        self.version
    }
}
