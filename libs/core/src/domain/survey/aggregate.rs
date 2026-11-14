use std::num::NonZeroU32;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::campaign::Campaign;
use super::error::SurveyError;
use super::graph::Graph;
use super::metadata::Metadata;
use super::value::{PublicSlug, Visibility};
use super::{CampaignId, SurveyId};
use crate::OrganizationId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Draft;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Opened;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Closed;

mod sealed {
    use std::fmt::Debug;

    use super::{Body, Graph, SurveyError};

    pub trait Sealed {
        type Definition: Debug + Clone + PartialEq;

        fn check(body: &Body) -> Result<(), SurveyError>;

        fn definition(graph: Option<Graph>) -> Result<Self::Definition, SurveyError>;

        fn graph(definition: &Self::Definition) -> Option<&Graph>;
    }
}

fn check_title(body: &Body) -> Result<(), SurveyError> {
    if body.title.trim().is_empty() {
        Err(SurveyError::SurveyTitleEmpty)
    } else {
        Ok(())
    }
}

fn check_slug(visibility: Visibility, slug: Option<&PublicSlug>) -> Result<(), SurveyError> {
    match (visibility, slug) {
        (Visibility::Public, None) => Err(SurveyError::SurveyPublicSlugRequired),
        (Visibility::Members, Some(_)) => Err(SurveyError::SurveyPublicSlugForbidden),
        _ => Ok(()),
    }
}

fn frozen(graph: Option<Graph>) -> Result<Graph, SurveyError> {
    graph.ok_or(SurveyError::SurveyWithoutGraph)
}

pub trait Stage: sealed::Sealed {}

impl Stage for Draft {}
impl Stage for Opened {}
impl Stage for Closed {}

impl sealed::Sealed for Draft {
    type Definition = Option<Graph>;

    fn check(body: &Body) -> Result<(), SurveyError> {
        if body.public_slug.is_some() {
            return Err(SurveyError::SurveyPublicSlugForbidden);
        }
        match (body.opened_at, body.closed_at) {
            (None, None) => Ok(()),
            _ => Err(SurveyError::SurveyInconsistentState),
        }
    }

    fn definition(graph: Option<Graph>) -> Result<Self::Definition, SurveyError> {
        Ok(graph)
    }

    fn graph(definition: &Self::Definition) -> Option<&Graph> {
        definition.as_ref()
    }
}

impl sealed::Sealed for Opened {
    type Definition = Graph;

    fn check(body: &Body) -> Result<(), SurveyError> {
        check_slug(body.visibility, body.public_slug.as_ref())?;
        match (body.opened_at, body.closed_at) {
            (Some(_), None) => Ok(()),
            _ => Err(SurveyError::SurveyInconsistentState),
        }
    }

    fn definition(graph: Option<Graph>) -> Result<Self::Definition, SurveyError> {
        frozen(graph)
    }

    fn graph(definition: &Self::Definition) -> Option<&Graph> {
        Some(definition)
    }
}

impl sealed::Sealed for Closed {
    type Definition = Graph;

    fn check(body: &Body) -> Result<(), SurveyError> {
        check_slug(body.visibility, body.public_slug.as_ref())?;
        match (body.opened_at, body.closed_at) {
            (Some(opened), Some(closed)) if opened <= closed => Ok(()),
            _ => Err(SurveyError::SurveyInconsistentState),
        }
    }

    fn definition(graph: Option<Graph>) -> Result<Self::Definition, SurveyError> {
        frozen(graph)
    }

    fn graph(definition: &Self::Definition) -> Option<&Graph> {
        Some(definition)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[doc(hidden)]
pub struct Body {
    id: SurveyId,
    organization_id: OrganizationId,
    title: String,
    description: Option<String>,
    visibility: Visibility,
    allow_multiple_responses: bool,
    response_cap: Option<NonZeroU32>,
    metadata: Metadata,
    campaign_id: Option<CampaignId>,
    public_slug: Option<PublicSlug>,
    opened_at: Option<DateTime<Utc>>,
    closed_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
struct SurveyView<'a> {
    #[serde(flatten)]
    body: &'a Body,
    graph: Option<&'a Graph>,
}

#[derive(Deserialize)]
struct SurveyRow {
    #[serde(flatten)]
    body: Body,
    graph: Option<Graph>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Survey<S: Stage> {
    body: Body,
    definition: S::Definition,
}

impl<S: Stage> Serialize for Survey<S> {
    fn serialize<T: Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
        SurveyView {
            body: &self.body,
            graph: S::graph(&self.definition),
        }
        .serialize(serializer)
    }
}

impl<'de, S: Stage> Deserialize<'de> for Survey<S> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let SurveyRow { body, graph } = SurveyRow::deserialize(deserializer)?;
        check_title(&body)
            .and_then(|()| S::check(&body))
            .and_then(|()| S::definition(graph))
            .map(|definition| Self { body, definition })
            .map_err(serde::de::Error::custom)
    }
}

impl<S: Stage> Survey<S> {
    pub fn id(&self) -> SurveyId {
        self.body.id
    }

    pub fn organization_id(&self) -> OrganizationId {
        self.body.organization_id
    }

    pub fn title(&self) -> &str {
        &self.body.title
    }

    pub fn description(&self) -> Option<&str> {
        self.body.description.as_deref()
    }

    pub fn visibility(&self) -> Visibility {
        self.body.visibility
    }

    pub fn allow_multiple_responses(&self) -> bool {
        self.body.allow_multiple_responses
    }

    pub fn response_cap(&self) -> Option<NonZeroU32> {
        self.body.response_cap
    }

    pub fn metadata(&self) -> &Metadata {
        &self.body.metadata
    }

    pub fn campaign_id(&self) -> Option<CampaignId> {
        self.body.campaign_id
    }

    pub fn graph(&self) -> Option<&Graph> {
        S::graph(&self.definition)
    }

    pub fn opened_at(&self) -> Option<DateTime<Utc>> {
        self.body.opened_at
    }

    pub fn closed_at(&self) -> Option<DateTime<Utc>> {
        self.body.closed_at
    }
}

impl Survey<Draft> {
    pub fn new(
        id: SurveyId,
        organization_id: OrganizationId,
        title: String,
        visibility: Visibility,
    ) -> Result<Self, SurveyError> {
        let mut survey = Self {
            body: Body {
                id,
                organization_id,
                title: String::new(),
                description: None,
                visibility,
                allow_multiple_responses: false,
                response_cap: None,
                metadata: Metadata::default(),
                campaign_id: None,
                public_slug: None,
                opened_at: None,
                closed_at: None,
            },
            definition: None,
        };
        survey.edit_title(title)?;
        Ok(survey)
    }

    pub fn edit_title(&mut self, title: String) -> Result<(), SurveyError> {
        if title.trim().is_empty() {
            return Err(SurveyError::SurveyTitleEmpty);
        }
        self.body.title = title;
        Ok(())
    }

    pub fn edit_description(&mut self, description: Option<String>) {
        self.body.description = description;
    }

    pub fn edit_visibility(&mut self, visibility: Visibility) {
        self.body.visibility = visibility;
    }

    pub fn edit_allow_multiple_responses(&mut self, allowed: bool) {
        self.body.allow_multiple_responses = allowed;
    }

    pub fn edit_response_cap(&mut self, cap: Option<NonZeroU32>) {
        self.body.response_cap = cap;
    }

    pub fn edit_metadata(&mut self, metadata: Metadata) {
        self.body.metadata = metadata;
    }

    pub fn edit_campaign(&mut self, campaign_id: Option<CampaignId>) {
        self.body.campaign_id = campaign_id;
    }

    pub fn edit_graph(&mut self, graph: Option<Graph>) {
        self.definition = graph;
    }

    pub fn open(
        mut self,
        at: DateTime<Utc>,
        public_slug: Option<PublicSlug>,
    ) -> Result<Survey<Opened>, SurveyError> {
        let definition = frozen(self.definition)?;
        check_slug(self.body.visibility, public_slug.as_ref())?;
        self.body.public_slug = public_slug;
        self.body.opened_at = Some(at);
        Ok(Survey {
            body: self.body,
            definition,
        })
    }
}

impl Survey<Opened> {
    pub fn definition(&self) -> &Graph {
        &self.definition
    }

    pub fn public_slug(&self) -> Option<&PublicSlug> {
        self.body.public_slug.as_ref()
    }

    pub fn close(mut self, at: DateTime<Utc>) -> Result<Survey<Closed>, SurveyError> {
        if self.body.opened_at.is_some_and(|opened| at < opened) {
            return Err(SurveyError::SurveyClosedBeforeOpened);
        }
        self.body.closed_at = Some(at);
        Ok(Survey {
            body: self.body,
            definition: self.definition,
        })
    }

    pub(super) fn check_campaign(&self, campaign: Option<&Campaign>) -> Result<(), SurveyError> {
        if campaign.is_some_and(|campaign| campaign.organization_id() != self.organization_id()) {
            return Err(SurveyError::SurveyCampaignOrganizationMismatch);
        }
        if self.campaign_id() != campaign.map(Campaign::id) {
            return Err(SurveyError::SurveyCampaignMismatch);
        }
        Ok(())
    }

    pub fn accepts(
        &self,
        campaign: Option<&Campaign>,
        now: DateTime<Utc>,
    ) -> Result<(), SurveyError> {
        self.check_campaign(campaign)?;
        let Some(campaign) = campaign else {
            return Ok(());
        };
        let window = campaign.window();
        if window.starts_at().is_some_and(|start| now < start) {
            return Err(SurveyError::CampaignNotStarted);
        }
        if !window.contains(now) {
            return Err(SurveyError::CampaignEnded);
        }
        Ok(())
    }
}

impl Survey<Closed> {
    pub fn public_slug(&self) -> Option<&PublicSlug> {
        self.body.public_slug.as_ref()
    }

    pub fn accepts(
        &self,
        _campaign: Option<&Campaign>,
        _now: DateTime<Utc>,
    ) -> Result<(), SurveyError> {
        Err(SurveyError::SurveyClosed)
    }
}
