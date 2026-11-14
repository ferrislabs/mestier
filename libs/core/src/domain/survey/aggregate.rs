use std::{marker::PhantomData, num::NonZeroU32};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::campaign::Campaign;
use super::error::SurveyError;
use super::graph::Graph;
use super::metadata::Metadata;
use super::value::Visibility;
use super::{CampaignId, SurveyId};
use crate::OrganizationId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Draft;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Opened;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Closed;

mod sealed {
    pub trait Sealed {}
}

impl sealed::Sealed for Draft {}
impl sealed::Sealed for Opened {}
impl sealed::Sealed for Closed {}

fn check_title(body: &Body) -> Result<(), SurveyError> {
    if body.title.trim().is_empty() {
        Err(SurveyError::SurveyTitleEmpty)
    } else {
        Ok(())
    }
}

pub trait Stage: sealed::Sealed {
    fn check(body: &Body) -> Result<(), SurveyError>;
}

impl Stage for Draft {
    fn check(body: &Body) -> Result<(), SurveyError> {
        match (body.opened_at, body.closed_at) {
            (None, None) => Ok(()),
            _ => Err(SurveyError::SurveyInconsistentState),
        }
    }
}

impl Stage for Opened {
    fn check(body: &Body) -> Result<(), SurveyError> {
        if body.graph.is_none() {
            return Err(SurveyError::SurveyWithoutGraph);
        }
        match (body.opened_at, body.closed_at) {
            (Some(_), None) => Ok(()),
            _ => Err(SurveyError::SurveyInconsistentState),
        }
    }
}

impl Stage for Closed {
    fn check(body: &Body) -> Result<(), SurveyError> {
        if body.graph.is_none() {
            return Err(SurveyError::SurveyWithoutGraph);
        }
        match (body.opened_at, body.closed_at) {
            (Some(opened), Some(closed)) if opened <= closed => Ok(()),
            _ => Err(SurveyError::SurveyInconsistentState),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Body {
    id: SurveyId,
    organization_id: OrganizationId,
    title: String,
    description: Option<String>,
    visibility: Visibility,
    allow_multiple_responses: bool,
    response_cap: Option<NonZeroU32>,
    metadata: Metadata,
    campaign_id: Option<CampaignId>,
    graph: Option<Graph>,
    opened_at: Option<DateTime<Utc>>,
    closed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Survey<S> {
    body: Body,
    stage: PhantomData<S>,
}

impl<S> Serialize for Survey<S> {
    fn serialize<T: Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
        self.body.serialize(serializer)
    }
}

impl<'de, S: Stage> Deserialize<'de> for Survey<S> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let body = Body::deserialize(deserializer)?;
        check_title(&body)
            .and_then(|()| S::check(&body))
            .map_err(serde::de::Error::custom)?;
        Ok(Self {
            body,
            stage: PhantomData,
        })
    }
}

impl<S> Survey<S> {
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
        self.body.graph.as_ref()
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
                graph: None,
                opened_at: None,
                closed_at: None,
            },
            stage: PhantomData,
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
        self.body.graph = graph;
    }

    pub fn open(mut self, at: DateTime<Utc>) -> Result<Survey<Opened>, SurveyError> {
        if self.body.graph.is_none() {
            return Err(SurveyError::SurveyWithoutGraph);
        }
        self.body.opened_at = Some(at);
        Ok(Survey {
            body: self.body,
            stage: PhantomData,
        })
    }
}

impl Survey<Opened> {
    pub fn close(mut self, at: DateTime<Utc>) -> Result<Survey<Closed>, SurveyError> {
        if self.body.opened_at.is_some_and(|opened| at < opened) {
            return Err(SurveyError::SurveyClosedBeforeOpened);
        }
        self.body.closed_at = Some(at);
        Ok(Survey {
            body: self.body,
            stage: PhantomData,
        })
    }

    pub fn accepts(
        &self,
        campaign: Option<&Campaign>,
        now: DateTime<Utc>,
    ) -> Result<(), SurveyError> {
        let Some(campaign) = campaign else {
            return Ok(());
        };
        let window = campaign.window;
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
    pub fn accepts(
        &self,
        _campaign: Option<&Campaign>,
        _now: DateTime<Utc>,
    ) -> Result<(), SurveyError> {
        Err(SurveyError::SurveyClosed)
    }
}
