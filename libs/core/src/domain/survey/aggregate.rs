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

    use super::{Body, SurveyError, VersionedGraph};

    pub trait Sealed {
        type Definition: Debug + Clone + PartialEq;
        type Input;

        fn check(body: &Body) -> Result<(), SurveyError>;

        fn definition(graph: Option<VersionedGraph>) -> Result<Self::Definition, SurveyError>;

        fn current(definition: &Self::Definition) -> Option<&VersionedGraph>;

        fn slot(
            definition: &mut Self::Definition,
            graph: Self::Input,
        ) -> Option<(&mut VersionedGraph, super::Graph)>;
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

fn frozen(graph: Option<VersionedGraph>) -> Result<VersionedGraph, SurveyError> {
    graph.ok_or(SurveyError::SurveyWithoutGraph)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GraphVersion(NonZeroU32);

impl GraphVersion {
    pub const FIRST: Self = Self(NonZeroU32::MIN);

    pub fn new(version: u32) -> Result<Self, SurveyError> {
        NonZeroU32::new(version)
            .map(Self)
            .ok_or(SurveyError::GraphVersionInvalid)
    }

    pub fn get(self) -> u32 {
        self.0.get()
    }

    pub fn next(self) -> Result<Self, SurveyError> {
        self.0
            .checked_add(1)
            .map(Self)
            .ok_or(SurveyError::GraphVersionExhausted)
    }
}

impl std::fmt::Display for GraphVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VersionedGraph {
    version: GraphVersion,
    graph: Graph,
}

impl VersionedGraph {
    pub fn new(version: GraphVersion, graph: Graph) -> Self {
        Self { version, graph }
    }

    pub fn version(&self) -> GraphVersion {
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
    type Definition = Option<VersionedGraph>;
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

    fn definition(graph: Option<VersionedGraph>) -> Result<Self::Definition, SurveyError> {
        Ok(graph)
    }

    fn current(definition: &Self::Definition) -> Option<&VersionedGraph> {
        definition.as_ref()
    }

    fn slot(
        definition: &mut Self::Definition,
        graph: Self::Input,
    ) -> Option<(&mut VersionedGraph, Graph)> {
        match (definition, graph) {
            (definition, None) => {
                *definition = None;
                None
            }
            (Some(current), Some(graph)) => Some((current, graph)),
            (definition, Some(graph)) => {
                *definition = Some(VersionedGraph::new(GraphVersion::FIRST, graph));
                None
            }
        }
    }
}

impl sealed::Sealed for Opened {
    type Definition = VersionedGraph;
    type Input = Graph;

    fn check(body: &Body) -> Result<(), SurveyError> {
        check_slug(body.visibility, body.public_slug.as_ref())?;
        match (body.opened_at, body.closed_at) {
            (Some(_), None) => Ok(()),
            _ => Err(SurveyError::SurveyInconsistentState),
        }
    }

    fn definition(graph: Option<VersionedGraph>) -> Result<Self::Definition, SurveyError> {
        frozen(graph)
    }

    fn current(definition: &Self::Definition) -> Option<&VersionedGraph> {
        Some(definition)
    }

    fn slot(
        definition: &mut Self::Definition,
        graph: Self::Input,
    ) -> Option<(&mut VersionedGraph, Graph)> {
        Some((definition, graph))
    }
}

impl sealed::Sealed for Closed {
    type Definition = VersionedGraph;
    type Input = Graph;

    fn check(body: &Body) -> Result<(), SurveyError> {
        check_slug(body.visibility, body.public_slug.as_ref())?;
        match (body.opened_at, body.closed_at) {
            (Some(opened), Some(closed)) if opened <= closed => Ok(()),
            _ => Err(SurveyError::SurveyInconsistentState),
        }
    }

    fn definition(graph: Option<VersionedGraph>) -> Result<Self::Definition, SurveyError> {
        frozen(graph)
    }

    fn current(definition: &Self::Definition) -> Option<&VersionedGraph> {
        Some(definition)
    }

    fn slot(
        definition: &mut Self::Definition,
        graph: Self::Input,
    ) -> Option<(&mut VersionedGraph, Graph)> {
        Some((definition, graph))
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
    graph_version: Option<GraphVersion>,
}

#[derive(Deserialize)]
struct SurveyRow {
    #[serde(flatten)]
    body: Body,
    graph: Option<Graph>,
    graph_version: Option<GraphVersion>,
}

fn versioned(
    graph: Option<Graph>,
    version: Option<GraphVersion>,
) -> Result<Option<VersionedGraph>, SurveyError> {
    match (graph, version) {
        (None, None) => Ok(None),
        (Some(graph), Some(version)) => Ok(Some(VersionedGraph::new(version, graph))),
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
        S::current(&self.definition).map(VersionedGraph::graph)
    }

    pub fn graph_version(&self) -> Option<GraphVersion> {
        S::current(&self.definition).map(VersionedGraph::version)
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
        if let Some((current, graph)) = S::slot(&mut self.definition, graph) {
            current.graph = graph;
        }
    }

    pub fn revise_graph(&mut self, graph: S::Input) -> Result<Option<VersionedGraph>, SurveyError> {
        match S::slot(&mut self.definition, graph) {
            Some((current, graph)) if current.graph != graph => {
                let next = VersionedGraph::new(current.version.next()?, graph);
                Ok(Some(std::mem::replace(current, next)))
            }
            _ => Ok(None),
        }
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

    pub fn version(&self) -> GraphVersion {
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
