use serde::Serialize;

use super::SurveyId;
use super::aggregate::{Opened, Survey};
use super::graph::Graph;
use super::value::{PublicSlug, Visibility};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RespondentSurvey {
    id: SurveyId,
    title: String,
    description: Option<String>,
    visibility: Visibility,
    graph: Graph,
    #[serde(skip_serializing_if = "Option::is_none")]
    public_slug: Option<PublicSlug>,
}

impl RespondentSurvey {
    pub fn id(&self) -> SurveyId {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn visibility(&self) -> Visibility {
        self.visibility
    }

    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    pub fn public_slug(&self) -> Option<&PublicSlug> {
        self.public_slug.as_ref()
    }
}

impl Survey<Opened> {
    pub fn respondent_view(&self) -> RespondentSurvey {
        RespondentSurvey {
            id: self.id(),
            title: self.title().to_owned(),
            description: self.description().map(str::to_owned),
            visibility: self.visibility(),
            graph: self.definition().clone(),
            public_slug: self.public_slug().cloned(),
        }
    }
}
