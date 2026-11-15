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

    use super::{Body, GraphVersion, SurveyError};

    pub trait Sealed {
        type Definition: Debug + Clone + PartialEq;
        type Input;

        fn check(body: &Body) -> Result<(), SurveyError>;

        fn definition(graph: Option<GraphVersion>) -> Result<Self::Definition, SurveyError>;

        fn current(definition: &Self::Definition) -> Option<&GraphVersion>;

        fn replace(
            definition: &mut Self::Definition,
            graph: Self::Input,
            keep_previous: bool,
        ) -> Option<GraphVersion>;
    }
}

fn check_title(body: &Body) -> Result<(), SurveyError> {
    if body.title.trim().is_empty() {
        Err(SurveyError::SurveyTitleEmpty)
    } else {
        Ok(())
    }
}

fn check_campaigns(body: &Body) -> Result<(), SurveyError> {
    let ids = &body.campaign_ids;
    if ids
        .iter()
        .enumerate()
        .any(|(position, id)| ids[..position].contains(id))
    {
        Err(SurveyError::SurveyInconsistentState)
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

fn frozen(graph: Option<GraphVersion>) -> Result<GraphVersion, SurveyError> {
    graph.ok_or(SurveyError::SurveyWithoutGraph)
}

fn swap(current: &mut GraphVersion, graph: Graph, keep_previous: bool) -> Option<GraphVersion> {
    if current.graph == graph {
        return None;
    }
    if keep_previous {
        let next = GraphVersion {
            version: current.version.saturating_add(1),
            graph,
        };
        Some(std::mem::replace(current, next))
    } else {
        current.graph = graph;
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphVersion {
    version: u32,
    graph: Graph,
}

impl GraphVersion {
    pub fn new(version: u32, graph: Graph) -> Result<Self, SurveyError> {
        if version == 0 {
            Err(SurveyError::GraphVersionInvalid)
        } else {
            Ok(Self { version, graph })
        }
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    pub fn into_graph(self) -> Graph {
        self.graph
    }
}

pub trait Stage: sealed::Sealed {}

impl Stage for Draft {}
impl Stage for Opened {}
impl Stage for Closed {}

impl sealed::Sealed for Draft {
    type Definition = Option<GraphVersion>;
    type Input = Option<Graph>;

    fn check(body: &Body) -> Result<(), SurveyError> {
        if body.public_slug.is_some() {
            return Err(SurveyError::SurveyPublicSlugForbidden);
        }
        match (body.opened_at, body.closed_at) {
            (None, None) => Ok(()),
            _ => Err(SurveyError::SurveyInconsistentState),
        }
    }

    fn definition(graph: Option<GraphVersion>) -> Result<Self::Definition, SurveyError> {
        Ok(graph)
    }

    fn current(definition: &Self::Definition) -> Option<&GraphVersion> {
        definition.as_ref()
    }

    fn replace(
        definition: &mut Self::Definition,
        graph: Self::Input,
        keep_previous: bool,
    ) -> Option<GraphVersion> {
        match (definition.as_mut(), graph) {
            (_, None) => {
                *definition = None;
                None
            }
            (None, Some(graph)) => {
                *definition = Some(GraphVersion { version: 1, graph });
                None
            }
            (Some(current), Some(graph)) => swap(current, graph, keep_previous),
        }
    }
}

impl sealed::Sealed for Opened {
    type Definition = GraphVersion;
    type Input = Graph;

    fn check(body: &Body) -> Result<(), SurveyError> {
        check_slug(body.visibility, body.public_slug.as_ref())?;
        match (body.opened_at, body.closed_at) {
            (Some(_), None) => Ok(()),
            _ => Err(SurveyError::SurveyInconsistentState),
        }
    }

    fn definition(graph: Option<GraphVersion>) -> Result<Self::Definition, SurveyError> {
        frozen(graph)
    }

    fn current(definition: &Self::Definition) -> Option<&GraphVersion> {
        Some(definition)
    }

    fn replace(
        definition: &mut Self::Definition,
        graph: Self::Input,
        keep_previous: bool,
    ) -> Option<GraphVersion> {
        swap(definition, graph, keep_previous)
    }
}

impl sealed::Sealed for Closed {
    type Definition = GraphVersion;
    type Input = Graph;

    fn check(body: &Body) -> Result<(), SurveyError> {
        check_slug(body.visibility, body.public_slug.as_ref())?;
        match (body.opened_at, body.closed_at) {
            (Some(opened), Some(closed)) if opened <= closed => Ok(()),
            _ => Err(SurveyError::SurveyInconsistentState),
        }
    }

    fn definition(graph: Option<GraphVersion>) -> Result<Self::Definition, SurveyError> {
        frozen(graph)
    }

    fn current(definition: &Self::Definition) -> Option<&GraphVersion> {
        Some(definition)
    }

    fn replace(
        definition: &mut Self::Definition,
        graph: Self::Input,
        keep_previous: bool,
    ) -> Option<GraphVersion> {
        swap(definition, graph, keep_previous)
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
    campaign_ids: Vec<CampaignId>,
    public_slug: Option<PublicSlug>,
    opened_at: Option<DateTime<Utc>>,
    closed_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
struct SurveyView<'a> {
    #[serde(flatten)]
    body: &'a Body,
    graph: Option<&'a Graph>,
    graph_version: Option<u32>,
}

#[derive(Deserialize)]
struct SurveyRow {
    #[serde(flatten)]
    body: Body,
    graph: Option<Graph>,
    graph_version: Option<u32>,
}

fn versioned(
    graph: Option<Graph>,
    version: Option<u32>,
) -> Result<Option<GraphVersion>, SurveyError> {
    match (graph, version) {
        (None, None) => Ok(None),
        (Some(graph), Some(version)) => GraphVersion::new(version, graph).map(Some),
        _ => Err(SurveyError::SurveyInconsistentState),
    }
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
            graph: self.graph(),
            graph_version: self.graph_version(),
        }
        .serialize(serializer)
    }
}

impl<'de, S: Stage> Deserialize<'de> for Survey<S> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let SurveyRow {
            body,
            graph,
            graph_version,
        } = SurveyRow::deserialize(deserializer)?;
        check_title(&body)
            .and_then(|()| check_campaigns(&body))
            .and_then(|()| S::check(&body))
            .and_then(|()| versioned(graph, graph_version))
            .and_then(S::definition)
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

    pub fn campaign_ids(&self) -> &[CampaignId] {
        &self.body.campaign_ids
    }

    pub fn graph(&self) -> Option<&Graph> {
        S::current(&self.definition).map(GraphVersion::graph)
    }

    pub fn graph_version(&self) -> Option<u32> {
        S::current(&self.definition).map(GraphVersion::version)
    }

    pub fn opened_at(&self) -> Option<DateTime<Utc>> {
        self.body.opened_at
    }

    pub fn closed_at(&self) -> Option<DateTime<Utc>> {
        self.body.closed_at
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

    pub fn edit_allow_multiple_responses(&mut self, allowed: bool) {
        self.body.allow_multiple_responses = allowed;
    }

    pub fn edit_response_cap(&mut self, cap: Option<NonZeroU32>) {
        self.body.response_cap = cap;
    }

    pub fn edit_metadata(&mut self, metadata: Metadata) {
        self.body.metadata = metadata;
    }

    pub fn edit_campaigns(&mut self, campaign_ids: Vec<CampaignId>) {
        let mut kept: Vec<CampaignId> = Vec::with_capacity(campaign_ids.len());
        for id in campaign_ids {
            if !kept.contains(&id) {
                kept.push(id);
            }
        }
        self.body.campaign_ids = kept;
    }

    pub fn edit_graph(&mut self, graph: S::Input) {
        S::replace(&mut self.definition, graph, false);
    }

    pub fn revise_graph(&mut self, graph: S::Input) -> Option<GraphVersion> {
        S::replace(&mut self.definition, graph, true)
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
                campaign_ids: Vec::new(),
                public_slug: None,
                opened_at: None,
                closed_at: None,
            },
            definition: None,
        };
        survey.edit_title(title)?;
        Ok(survey)
    }

    pub fn edit_visibility(&mut self, visibility: Visibility) {
        self.body.visibility = visibility;
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
        &self.definition.graph
    }

    pub fn version(&self) -> u32 {
        self.definition.version
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

    pub(super) fn ordered_campaigns<'c>(
        &self,
        campaigns: &'c [Campaign],
    ) -> Result<Vec<&'c Campaign>, SurveyError> {
        if campaigns
            .iter()
            .any(|campaign| campaign.organization_id() != self.organization_id())
        {
            return Err(SurveyError::SurveyCampaignOrganizationMismatch);
        }
        let ids = self.campaign_ids();
        let ordered: Vec<&Campaign> = ids
            .iter()
            .filter_map(|id| campaigns.iter().find(|campaign| campaign.id() == *id))
            .collect();
        if ordered.len() != ids.len() || campaigns.len() != ids.len() {
            return Err(SurveyError::SurveyCampaignMismatch);
        }
        Ok(ordered)
    }

    pub fn accepts(&self, campaigns: &[Campaign], now: DateTime<Utc>) -> Result<(), SurveyError> {
        let campaigns = self.ordered_campaigns(campaigns)?;
        if campaigns.is_empty()
            || campaigns
                .iter()
                .any(|campaign| campaign.window().contains(now))
        {
            return Ok(());
        }
        if campaigns.iter().any(|campaign| {
            campaign
                .window()
                .starts_at()
                .is_some_and(|start| now < start)
        }) {
            Err(SurveyError::CampaignNotStarted)
        } else {
            Err(SurveyError::CampaignEnded)
        }
    }
}

impl Survey<Closed> {
    pub fn public_slug(&self) -> Option<&PublicSlug> {
        self.body.public_slug.as_ref()
    }

    pub fn accepts(&self, _campaigns: &[Campaign], _now: DateTime<Utc>) -> Result<(), SurveyError> {
        Err(SurveyError::SurveyClosed)
    }
}
