use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize, Serializer};

use super::error::SurveyError;
use super::value::{Answer, QuestionId, QuestionKind};

pub type Answers = BTreeMap<QuestionId, Answer>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestionNode {
    pub id: QuestionId,
    pub kind: QuestionKind,
    pub label: String,
    pub required: bool,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub max_photos: Option<u8>,
    #[serde(default)]
    pub identifier: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConditionNode {
    pub id: QuestionId,
    pub question: QuestionId,
    pub option: String,
    pub checked: bool,
}

impl ConditionNode {
    fn holds(&self, answers: &Answers) -> bool {
        let checked = match answers.get(&self.question) {
            Some(Answer::Choice(option)) => *option == self.option,
            Some(Answer::Choices(options)) => options.contains(&self.option),
            _ => false,
        };
        checked == self.checked
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EndNode {
    pub id: QuestionId,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Node {
    Question(QuestionNode),
    Condition(ConditionNode),
    End(EndNode),
}

impl Node {
    pub fn id(&self) -> QuestionId {
        match self {
            Self::Question(node) => node.id,
            Self::Condition(node) => node.id,
            Self::End(node) => node.id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Branch {
    Next,
    Then,
    Else,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Edge {
    pub from: QuestionId,
    pub to: QuestionId,
    pub branch: Branch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphPath {
    pub questions: Vec<QuestionId>,
    pub end: QuestionId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Vertex {
    Question {
        node: QuestionNode,
        next: usize,
    },
    Condition {
        node: ConditionNode,
        then: usize,
        otherwise: usize,
    },
    End(EndNode),
}

impl Vertex {
    fn id(&self) -> QuestionId {
        match self {
            Self::Question { node, .. } => node.id,
            Self::Condition { node, .. } => node.id,
            Self::End(node) => node.id,
        }
    }

    fn successors(&self) -> impl Iterator<Item = usize> {
        let pair = match self {
            Self::Question { next, .. } => [Some(*next), None],
            Self::Condition {
                then, otherwise, ..
            } => [Some(*then), Some(*otherwise)],
            Self::End(_) => [None, None],
        };
        pair.into_iter().flatten()
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct Exits {
    next: Option<usize>,
    then: Option<usize>,
    otherwise: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mark {
    Unvisited,
    Open,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "GraphDraft")]
pub struct Graph {
    vertices: Vec<Vertex>,
    start: usize,
}

#[derive(Deserialize)]
struct GraphDraft {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    start: QuestionId,
}

impl TryFrom<GraphDraft> for Graph {
    type Error = SurveyError;

    fn try_from(draft: GraphDraft) -> Result<Self, Self::Error> {
        Self::new(draft.nodes, draft.edges, draft.start)
    }
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
enum NodeView<'a> {
    Question(&'a QuestionNode),
    Condition(&'a ConditionNode),
    End(&'a EndNode),
}

#[derive(Serialize)]
struct GraphView<'a> {
    nodes: Vec<NodeView<'a>>,
    edges: Vec<Edge>,
    start: QuestionId,
}

impl Serialize for Graph {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        GraphView {
            nodes: self.vertices.iter().map(NodeView::from).collect(),
            edges: self.edges(),
            start: self.vertices[self.start].id(),
        }
        .serialize(serializer)
    }
}

impl<'a> From<&'a Vertex> for NodeView<'a> {
    fn from(vertex: &'a Vertex) -> Self {
        match vertex {
            Vertex::Question { node, .. } => Self::Question(node),
            Vertex::Condition { node, .. } => Self::Condition(node),
            Vertex::End(node) => Self::End(node),
        }
    }
}

impl Graph {
    pub fn new(nodes: Vec<Node>, edges: Vec<Edge>, start: QuestionId) -> Result<Self, SurveyError> {
        if nodes.is_empty() {
            return Err(SurveyError::GraphEmpty);
        }
        let mut index = BTreeMap::new();
        for (position, node) in nodes.iter().enumerate() {
            if index.insert(node.id(), position).is_some() {
                return Err(SurveyError::GraphDuplicateNode(node.id()));
            }
        }
        if !nodes.iter().any(|node| matches!(node, Node::End(_))) {
            return Err(SurveyError::GraphWithoutEnd);
        }
        let start = *index.get(&start).ok_or(SurveyError::GraphUnknownStart)?;
        if !matches!(nodes[start], Node::Question(_)) {
            return Err(SurveyError::GraphStartNotQuestion);
        }
        let exits = Self::exits(&nodes, &edges, &index)?;
        let vertices = nodes
            .into_iter()
            .zip(exits)
            .map(Self::vertex)
            .collect::<Result<Vec<_>, _>>()?;
        let order = Self::topological_order(&vertices, start)?;
        Self::check_conditions(&vertices, &index, &order, start)?;
        Ok(Self { vertices, start })
    }

    pub fn path(&self, answers: &Answers) -> GraphPath {
        let mut questions = Vec::new();
        let mut current = self.start;
        loop {
            match &self.vertices[current] {
                Vertex::Question { node, next } => {
                    questions.push(node.id);
                    current = *next;
                }
                Vertex::Condition {
                    node,
                    then,
                    otherwise,
                } => {
                    current = if node.holds(answers) {
                        *then
                    } else {
                        *otherwise
                    };
                }
                Vertex::End(node) => {
                    return GraphPath {
                        questions,
                        end: node.id,
                    };
                }
            }
        }
    }

    fn edges(&self) -> Vec<Edge> {
        let mut edges = Vec::with_capacity(self.vertices.len() * 2);
        for vertex in &self.vertices {
            let from = vertex.id();
            let id = |position: usize| self.vertices[position].id();
            match vertex {
                Vertex::Question { next, .. } => edges.push(Edge {
                    from,
                    to: id(*next),
                    branch: Branch::Next,
                }),
                Vertex::Condition {
                    then, otherwise, ..
                } => {
                    edges.push(Edge {
                        from,
                        to: id(*then),
                        branch: Branch::Then,
                    });
                    edges.push(Edge {
                        from,
                        to: id(*otherwise),
                        branch: Branch::Else,
                    });
                }
                Vertex::End(_) => {}
            }
        }
        edges
    }

    fn exits(
        nodes: &[Node],
        edges: &[Edge],
        index: &BTreeMap<QuestionId, usize>,
    ) -> Result<Vec<Exits>, SurveyError> {
        let mut exits = vec![Exits::default(); nodes.len()];
        for edge in edges {
            let from = *index
                .get(&edge.from)
                .ok_or(SurveyError::GraphUnknownEdgeNode(edge.from))?;
            let to = *index
                .get(&edge.to)
                .ok_or(SurveyError::GraphUnknownEdgeNode(edge.to))?;
            let exit = &mut exits[from];
            let slot = match (&nodes[from], edge.branch) {
                (Node::Question(_), Branch::Next) => &mut exit.next,
                (Node::Condition(_), Branch::Then) => &mut exit.then,
                (Node::Condition(_), Branch::Else) => &mut exit.otherwise,
                _ => return Err(SurveyError::GraphInvalidBranch(edge.from)),
            };
            if slot.replace(to).is_some() {
                return Err(SurveyError::GraphDuplicateExit(edge.from));
            }
        }
        Ok(exits)
    }

    fn vertex((node, exits): (Node, Exits)) -> Result<Vertex, SurveyError> {
        let dead_end = SurveyError::GraphDeadEnd(node.id());
        match node {
            Node::Question(node) => exits
                .next
                .map(|next| Vertex::Question { node, next })
                .ok_or(dead_end),
            Node::Condition(node) => match (exits.then, exits.otherwise) {
                (Some(then), Some(otherwise)) => Ok(Vertex::Condition {
                    node,
                    then,
                    otherwise,
                }),
                _ => Err(dead_end),
            },
            Node::End(node) => Ok(Vertex::End(node)),
        }
    }

    fn topological_order(vertices: &[Vertex], start: usize) -> Result<Vec<usize>, SurveyError> {
        let mut marks = vec![Mark::Unvisited; vertices.len()];
        let mut finished = Vec::with_capacity(vertices.len());
        let mut stack = vec![(start, false)];
        while let Some((position, expanded)) = stack.pop() {
            if expanded {
                marks[position] = Mark::Closed;
                finished.push(position);
                continue;
            }
            if marks[position] != Mark::Unvisited {
                continue;
            }
            marks[position] = Mark::Open;
            stack.push((position, true));
            for successor in vertices[position].successors() {
                match marks[successor] {
                    Mark::Open => return Err(SurveyError::GraphCycle(vertices[successor].id())),
                    Mark::Unvisited => stack.push((successor, false)),
                    Mark::Closed => {}
                }
            }
        }
        if let Some(position) = marks.iter().position(|mark| *mark == Mark::Unvisited) {
            return Err(SurveyError::GraphUnreachableNode(vertices[position].id()));
        }
        finished.reverse();
        Ok(finished)
    }

    fn check_conditions(
        vertices: &[Vertex],
        index: &BTreeMap<QuestionId, usize>,
        order: &[usize],
        start: usize,
    ) -> Result<(), SurveyError> {
        let mut asked: Vec<Option<BTreeSet<QuestionId>>> = vec![None; vertices.len()];
        asked[start] = Some(BTreeSet::new());
        for &position in order {
            let mut carried = asked[position].take().unwrap_or_default();
            match &vertices[position] {
                Vertex::Question { node, .. } => {
                    carried.insert(node.id);
                }
                Vertex::Condition { node, .. } => {
                    Self::check_condition(node, vertices, index, &carried)?;
                }
                Vertex::End(_) => {}
            }
            for successor in vertices[position].successors() {
                asked[successor] = Some(match asked[successor].take() {
                    None => carried.clone(),
                    Some(previous) => previous.intersection(&carried).copied().collect(),
                });
            }
        }
        Ok(())
    }

    fn check_condition(
        condition: &ConditionNode,
        vertices: &[Vertex],
        index: &BTreeMap<QuestionId, usize>,
        asked: &BTreeSet<QuestionId>,
    ) -> Result<(), SurveyError> {
        let target = *index
            .get(&condition.question)
            .ok_or(SurveyError::ConditionUnknownQuestion(condition.id))?;
        let Vertex::Question { node: question, .. } = &vertices[target] else {
            return Err(SurveyError::ConditionOnNonChoice(condition.id));
        };
        if !matches!(
            question.kind,
            QuestionKind::SingleChoice | QuestionKind::MultipleChoice
        ) {
            return Err(SurveyError::ConditionOnNonChoice(condition.id));
        }
        if !question.options.contains(&condition.option) {
            return Err(SurveyError::ConditionUnknownOption(condition.id));
        }
        if !asked.contains(&condition.question) {
            return Err(SurveyError::ConditionQuestionNotAsked(condition.id));
        }
        Ok(())
    }
}
