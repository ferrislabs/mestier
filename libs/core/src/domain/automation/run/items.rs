use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use serde_json::{Map, Value, json};

use crate::domain::automation::workflow::{Branch, Graph};

use super::graph::descendants_via;

pub const SCALAR_KEY: &str = "valeur";

pub fn as_item(value: Value) -> Value {
    match value {
        Value::Object(_) => value,
        other => json!({ SCALAR_KEY: other }),
    }
}

pub fn push_result(items: &mut Vec<Value>, produced: Value) {
    match produced {
        Value::Array(list) => items.extend(list.into_iter().map(as_item)),
        other => items.push(as_item(other)),
    }
}

pub fn trigger_item(payload: Option<&Value>) -> Value {
    match payload {
        None | Some(Value::Null) => Value::Object(Map::new()),
        Some(other) => as_item(other.clone()),
    }
}

fn branch_key(branch: Branch) -> &'static str {
    match branch {
        Branch::Then => "Then",
        Branch::Else => "Else",
        Branch::Each => "Each",
        Branch::After => "After",
    }
}

fn branch_of(key: &str) -> Option<Branch> {
    match key {
        "Then" => Some(Branch::Then),
        "Else" => Some(Branch::Else),
        "Each" => Some(Branch::Each),
        "After" => Some(Branch::After),
        _ => None,
    }
}

const BRANCH_ORDER: [Branch; 4] = [Branch::Then, Branch::Else, Branch::Each, Branch::After];

#[derive(Debug, Clone, PartialEq)]
pub enum StepItems {
    Main(Vec<Value>),
    Branched(Vec<(Branch, Vec<Value>)>),
}

impl StepItems {
    pub fn passthrough(branches: &[Branch], input: Vec<Value>) -> Self {
        if branches.contains(&Branch::Then) {
            Self::Branched(vec![(Branch::Then, input), (Branch::Else, Vec::new())])
        } else if branches.contains(&Branch::Each) {
            Self::Branched(vec![
                (Branch::Each, Vec::new()),
                (Branch::After, Vec::new()),
            ])
        } else {
            Self::Main(input)
        }
    }

    pub fn from_json(value: &Value) -> Self {
        match value {
            Value::Array(items) => Self::Main(items.clone()),
            Value::Object(map) if is_branched(map) => Self::Branched(
                BRANCH_ORDER
                    .iter()
                    .filter_map(|branch| {
                        map.get(branch_key(*branch))
                            .and_then(Value::as_array)
                            .map(|items| (*branch, items.clone()))
                    })
                    .collect(),
            ),
            Value::Null => Self::Main(Vec::new()),
            other => Self::Main(vec![other.clone()]),
        }
    }

    pub fn to_json(&self) -> Value {
        match self {
            Self::Main(items) => Value::Array(items.clone()),
            Self::Branched(lists) => Value::Object(
                lists
                    .iter()
                    .map(|(branch, items)| {
                        (branch_key(*branch).to_owned(), Value::Array(items.clone()))
                    })
                    .collect(),
            ),
        }
    }

    pub fn list(&self, branch: Option<Branch>) -> &[Value] {
        match (self, branch) {
            (Self::Main(items), None) => items,
            (Self::Branched(lists), Some(wanted)) => lists
                .iter()
                .find(|(branch, _)| *branch == wanted)
                .map_or(&[], |(_, items)| items.as_slice()),
            _ => &[],
        }
    }

    pub fn sent(&self, branch: Option<Branch>) -> Vec<Value> {
        self.list(branch).iter().cloned().map(as_item).collect()
    }

    pub fn view(&self, feeding: &[Branch]) -> Vec<Value> {
        match self {
            Self::Main(items) => items.iter().cloned().map(as_item).collect(),
            Self::Branched(lists) => {
                let selected = lists
                    .iter()
                    .filter(|(branch, _)| feeding.is_empty() || feeding.contains(branch));
                selected
                    .flat_map(|(_, items)| items.iter().cloned().map(as_item))
                    .collect()
            }
        }
    }
}

fn is_branched(map: &Map<String, Value>) -> bool {
    !map.is_empty()
        && map
            .iter()
            .all(|(key, value)| branch_of(key).is_some() && value.is_array())
}

pub fn gather_lists(
    graph: &Graph,
    node_id: &str,
    sent: impl Fn(&str, Option<Branch>) -> Option<Vec<Value>>,
    trigger_item: &Value,
) -> Vec<Vec<Value>> {
    let mut seen: HashSet<(&str, Option<Branch>)> = HashSet::new();
    let mut trigger_taken = false;
    let mut lists = Vec::new();
    let mut fed = false;

    for edge in graph.edges.iter().filter(|edge| edge.to == node_id) {
        if edge.from == node_id {
            continue;
        }
        fed = true;
        if !seen.insert((edge.from.as_str(), edge.branch)) {
            continue;
        }
        if let Some(list) = sent(&edge.from, edge.branch) {
            lists.push(list);
            continue;
        }
        let from_trigger = graph.triggers.iter().any(|trigger| trigger.id == edge.from);
        if from_trigger && !trigger_taken {
            trigger_taken = true;
            lists.push(vec![trigger_item.clone()]);
        }
    }

    if !fed {
        lists.push(vec![trigger_item.clone()]);
    }
    lists
}

pub fn flatten(lists: &[Vec<Value>]) -> Vec<Value> {
    lists.iter().flatten().cloned().collect()
}

fn node_ids_by_name(graph: &Graph) -> HashMap<&str, &str> {
    let mut ids: HashMap<&str, &str> = HashMap::new();
    let named = graph
        .connectors
        .iter()
        .map(|connector| (connector.name.as_deref(), connector.id.as_str()))
        .chain(
            graph
                .triggers
                .iter()
                .map(|trigger| (trigger.name.as_deref(), trigger.id.as_str())),
        );
    for (name, id) in named {
        if let Some(name) = name.map(str::trim).filter(|name| !name.is_empty()) {
            ids.entry(name).or_insert(id);
        }
    }
    ids
}

struct Reach<'g> {
    by_branch: HashMap<&'g str, Vec<(Branch, HashSet<&'g str>)>>,
}

impl<'g> Reach<'g> {
    fn new(graph: &'g Graph) -> Self {
        let mut grouped: HashMap<&str, Vec<(Branch, Vec<&str>)>> = HashMap::new();
        for edge in &graph.edges {
            let Some(branch) = edge.branch else { continue };
            let groups = grouped.entry(edge.from.as_str()).or_default();
            match groups.iter_mut().find(|(known, _)| *known == branch) {
                Some((_, targets)) => targets.push(edge.to.as_str()),
                None => groups.push((branch, vec![edge.to.as_str()])),
            }
        }
        let by_branch = grouped
            .into_iter()
            .map(|(source, groups)| {
                let reached = groups
                    .into_iter()
                    .map(|(branch, targets)| (branch, descendants_via(graph, &targets)))
                    .collect();
                (source, reached)
            })
            .collect();
        Self { by_branch }
    }

    fn feeding(&self, source: &str, node: &str) -> Vec<Branch> {
        self.by_branch
            .get(source)
            .into_iter()
            .flatten()
            .filter(|(_, reached)| reached.contains(node))
            .map(|(branch, _)| *branch)
            .collect()
    }
}

pub fn referenced_names(config: &Map<String, Value>, nested_fields: &[&str]) -> BTreeSet<String> {
    fn collect(raw: &Value, names: &mut BTreeSet<String>, deep: bool) {
        match raw {
            Value::Object(map) if deep => {
                map.values().for_each(|value| collect(value, names, deep))
            }
            Value::Array(items) if deep => {
                items.iter().for_each(|value| collect(value, names, deep))
            }
            other => {
                if let Ok(template) = crate::domain::automation::expression::parse_template(other) {
                    names.extend(template.referenced_nodes());
                }
            }
        }
    }

    let mut names = BTreeSet::new();
    for (key, raw) in config {
        collect(raw, &mut names, nested_fields.contains(&key.as_str()));
    }
    names
}

pub struct NodeIndex<'g> {
    graph: &'g Graph,
    ids_by_name: HashMap<&'g str, &'g str>,
    reach: Reach<'g>,
}

impl<'g> NodeIndex<'g> {
    pub fn new(graph: &'g Graph) -> Self {
        Self {
            graph,
            ids_by_name: node_ids_by_name(graph),
            reach: Reach::new(graph),
        }
    }

    pub fn view(
        &self,
        names: &BTreeSet<String>,
        evaluated: &str,
        trigger: &Value,
        loop_item: impl Fn(&str) -> Option<Value>,
        output_of: impl Fn(&str) -> Option<StepItems>,
    ) -> BTreeMap<String, Vec<Value>> {
        let mut view = BTreeMap::new();
        for name in names {
            let Some(id) = self.ids_by_name.get(name.as_str()).copied() else {
                continue;
            };
            if self
                .graph
                .triggers
                .iter()
                .any(|candidate| candidate.id == id)
            {
                view.insert(name.clone(), vec![trigger.clone()]);
            } else if let Some(item) = loop_item(id) {
                view.insert(name.clone(), vec![as_item(item)]);
            } else if let Some(output) = output_of(id) {
                view.insert(
                    name.clone(),
                    output.view(&self.reach.feeding(id, evaluated)),
                );
            }
        }
        view
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::automation::workflow::{Edge, PlacedConnector, PlacedTrigger, TriggerKind};

    fn connector(id: &str, name: Option<&str>) -> PlacedConnector {
        PlacedConnector {
            id: id.to_string(),
            kind: "flow.config".to_string(),
            version: 1,
            credential_id: None,
            config: Map::new(),
            name: name.map(str::to_owned),
            description: None,
            disabled: false,
            operation: None,
        }
    }

    fn edge(from: &str, to: &str, branch: Option<Branch>) -> Edge {
        Edge {
            from: from.to_string(),
            to: to.to_string(),
            branch,
        }
    }

    fn trigger(id: &str, name: Option<&str>) -> PlacedTrigger {
        PlacedTrigger {
            id: id.to_string(),
            kind: TriggerKind::Manual,
            name: name.map(str::to_owned),
            description: None,
        }
    }

    #[test]
    fn a_scalar_is_wrapped_as_an_item_and_an_object_is_kept() {
        assert_eq!(as_item(json!(3)), json!({ "valeur": 3 }));
        assert_eq!(as_item(json!("a")), json!({ "valeur": "a" }));
        assert_eq!(as_item(Value::Null), json!({ "valeur": null }));
        assert_eq!(as_item(json!({ "a": 1 })), json!({ "a": 1 }));
    }

    #[test]
    fn a_produced_list_flattens_into_items_and_a_single_value_is_one_item() {
        let mut items = Vec::new();

        push_result(&mut items, json!([{ "a": 1 }, 2]));
        push_result(&mut items, json!({ "b": 3 }));
        push_result(&mut items, json!([]));

        assert_eq!(
            items,
            vec![json!({ "a": 1 }), json!({ "valeur": 2 }), json!({ "b": 3 })]
        );
    }

    #[test]
    fn a_trigger_always_produces_one_object_item() {
        assert_eq!(trigger_item(None), json!({}));
        assert_eq!(trigger_item(Some(&Value::Null)), json!({}));
        assert_eq!(trigger_item(Some(&json!({ "a": 1 }))), json!({ "a": 1 }));
        assert_eq!(trigger_item(Some(&json!(7))), json!({ "valeur": 7 }));
    }

    #[test]
    fn a_plain_step_stores_a_list_and_a_branching_one_stores_a_list_per_branch() {
        let plain = StepItems::Main(vec![json!({ "a": 1 }), json!({ "a": 2 })]);
        let branched = StepItems::Branched(vec![
            (Branch::Then, vec![json!({ "a": 1 })]),
            (Branch::Else, vec![]),
        ]);

        assert_eq!(plain.to_json(), json!([{ "a": 1 }, { "a": 2 }]));
        assert_eq!(
            branched.to_json(),
            json!({ "Then": [{ "a": 1 }], "Else": [] })
        );
        assert_eq!(StepItems::from_json(&plain.to_json()), plain);
        assert_eq!(StepItems::from_json(&branched.to_json()), branched);
    }

    #[test]
    fn a_loop_stores_its_each_and_after_lists() {
        let looped = StepItems::Branched(vec![
            (Branch::Each, vec![json!(1), json!(2)]),
            (Branch::After, vec![json!(1), json!(2)]),
        ]);

        assert_eq!(looped.to_json(), json!({ "Each": [1, 2], "After": [1, 2] }));
        assert_eq!(StepItems::from_json(&looped.to_json()), looped);
    }

    #[test]
    fn a_stored_value_that_is_not_a_list_reads_as_one_item() {
        assert_eq!(
            StepItems::from_json(&json!({ "id": 42 })),
            StepItems::Main(vec![json!({ "id": 42 })])
        );
        assert_eq!(
            StepItems::from_json(&json!({ "items": [1], "matched": true })),
            StepItems::Main(vec![json!({ "items": [1], "matched": true })])
        );
        assert_eq!(
            StepItems::from_json(&json!("text")),
            StepItems::Main(vec![json!("text")])
        );
        assert_eq!(
            StepItems::from_json(&Value::Null),
            StepItems::Main(Vec::new())
        );
        assert_eq!(
            StepItems::from_json(&json!([])),
            StepItems::Main(Vec::new())
        );
    }

    #[test]
    fn the_list_an_edge_carries_is_the_one_of_its_branch() {
        let branched = StepItems::Branched(vec![
            (Branch::Then, vec![json!({ "n": 1 })]),
            (Branch::Else, vec![json!({ "n": 2 }), json!({ "n": 3 })]),
        ]);
        let plain = StepItems::Main(vec![json!({ "n": 4 })]);

        assert_eq!(branched.list(Some(Branch::Else)).len(), 2);
        assert_eq!(branched.list(Some(Branch::Then)), [json!({ "n": 1 })]);
        assert!(branched.list(None).is_empty());
        assert!(branched.list(Some(Branch::Each)).is_empty());
        assert_eq!(plain.list(None), [json!({ "n": 4 })]);
        assert!(plain.list(Some(Branch::Then)).is_empty());
    }

    #[test]
    fn what_a_branch_sends_downstream_is_always_made_of_objects() {
        let looped = StepItems::Branched(vec![(Branch::Each, vec![json!(1), json!({ "a": 1 })])]);

        assert_eq!(
            looped.sent(Some(Branch::Each)),
            vec![json!({ "valeur": 1 }), json!({ "a": 1 })]
        );
    }

    #[test]
    fn a_disabled_node_sends_everything_on_its_main_output() {
        let input = vec![json!({ "a": 1 })];

        assert_eq!(
            StepItems::passthrough(&[], input.clone()),
            StepItems::Main(input.clone())
        );
        assert_eq!(
            StepItems::passthrough(&[Branch::Then, Branch::Else], input.clone()),
            StepItems::Branched(vec![(Branch::Then, input.clone()), (Branch::Else, vec![])])
        );
        assert_eq!(
            StepItems::passthrough(&[Branch::Each, Branch::After], input),
            StepItems::Branched(vec![(Branch::Each, vec![]), (Branch::After, vec![])])
        );
    }

    #[test]
    fn the_view_of_a_branching_node_keeps_the_branches_that_feed_the_reader() {
        let branched = StepItems::Branched(vec![
            (Branch::Then, vec![json!({ "n": 1 })]),
            (Branch::Else, vec![json!({ "n": 2 })]),
        ]);

        assert_eq!(branched.view(&[Branch::Else]), vec![json!({ "n": 2 })]);
        assert_eq!(
            branched.view(&[Branch::Then, Branch::Else]),
            vec![json!({ "n": 1 }), json!({ "n": 2 })]
        );
        assert_eq!(branched.view(&[]).len(), 2);
    }

    #[test]
    fn a_node_receives_the_lists_of_its_predecessors_in_edge_order() {
        let graph = Graph {
            connectors: vec![
                connector("a", None),
                connector("b", None),
                connector("m", None),
            ],
            edges: vec![
                edge("t1", "a", None),
                edge("t1", "b", None),
                edge("b", "m", None),
                edge("a", "m", None),
            ],
            triggers: vec![trigger("t1", None)],
        };
        let sent = |from: &str, _: Option<Branch>| match from {
            "a" => Some(vec![json!({ "from": "a" })]),
            "b" => Some(vec![json!({ "from": "b1" }), json!({ "from": "b2" })]),
            _ => None,
        };

        let lists = gather_lists(&graph, "m", sent, &json!({ "trigger": true }));

        assert_eq!(
            flatten(&lists),
            vec![
                json!({ "from": "b1" }),
                json!({ "from": "b2" }),
                json!({ "from": "a" })
            ]
        );
        assert_eq!(lists.len(), 2);
    }

    #[test]
    fn a_predecessor_that_sent_nothing_is_left_out_and_the_trigger_counts_once() {
        let graph = Graph {
            connectors: vec![connector("a", None), connector("m", None)],
            edges: vec![
                edge("t1", "m", None),
                edge("t2", "m", None),
                edge("a", "m", None),
            ],
            triggers: vec![trigger("t1", None), trigger("t2", None)],
        };
        let item = json!({ "payload": 1 });

        let lists = gather_lists(&graph, "m", |_, _| None, &item);

        assert_eq!(lists, vec![vec![item]]);
    }

    #[test]
    fn a_node_nothing_points_at_starts_from_the_trigger_item() {
        let graph = Graph {
            connectors: vec![connector("a", None)],
            edges: vec![],
            triggers: vec![],
        };

        let lists = gather_lists(&graph, "a", |_, _| None, &json!({ "x": 1 }));

        assert_eq!(lists, vec![vec![json!({ "x": 1 })]]);
    }

    #[test]
    fn an_edge_is_read_once_per_source_and_branch() {
        let graph = Graph {
            connectors: vec![connector("c", None), connector("m", None)],
            edges: vec![
                edge("c", "m", Some(Branch::Then)),
                edge("c", "m", Some(Branch::Then)),
                edge("c", "m", Some(Branch::Else)),
            ],
            triggers: vec![],
        };
        let sent = |_: &str, branch: Option<Branch>| match branch {
            Some(Branch::Then) => Some(vec![json!({ "side": "then" })]),
            Some(Branch::Else) => Some(vec![json!({ "side": "else" })]),
            _ => None,
        };

        let lists = gather_lists(&graph, "m", sent, &json!({}));

        assert_eq!(
            lists,
            vec![
                vec![json!({ "side": "then" })],
                vec![json!({ "side": "else" })]
            ]
        );
    }

    #[test]
    fn nodes_are_reached_by_their_trimmed_names_triggers_included() {
        let graph = Graph {
            connectors: vec![
                connector("c1", Some("  Rechercher ")),
                connector("c2", None),
                connector("c3", Some("  ")),
            ],
            edges: vec![],
            triggers: vec![trigger("t1", Some("Départ"))],
        };

        let ids = node_ids_by_name(&graph);

        assert_eq!(ids.get("Rechercher"), Some(&"c1"));
        assert_eq!(ids.get("Départ"), Some(&"t1"));
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn the_branches_that_lead_to_a_node_are_the_ones_whose_targets_reach_it() {
        let graph = Graph {
            connectors: vec![
                connector("cond", None),
                connector("yes", None),
                connector("no", None),
                connector("join", None),
            ],
            edges: vec![
                edge("cond", "yes", Some(Branch::Then)),
                edge("cond", "no", Some(Branch::Else)),
                edge("yes", "join", None),
                edge("no", "join", None),
            ],
            triggers: vec![],
        };
        let reach = Reach::new(&graph);

        assert_eq!(reach.feeding("cond", "yes"), vec![Branch::Then]);
        assert_eq!(reach.feeding("cond", "no"), vec![Branch::Else]);
        let mut both = reach.feeding("cond", "join");
        both.sort_by_key(|branch| branch_key(*branch));
        assert_eq!(both, vec![Branch::Else, Branch::Then]);
        assert!(reach.feeding("yes", "join").is_empty());
    }

    #[test]
    fn the_names_an_expression_reads_are_found_in_every_field_and_nested_value() {
        let config = json!({
            "to": "{{ $('Destinataire').item.json.email }}",
            "subject": "Bonjour {{ $('Devis').first().json.nom }}",
            "plain": "text",
            "count": 3,
            "body": { "list": [{ "name": "{{ $('Profond').item.json.x }}" }] }
        })
        .as_object()
        .cloned()
        .unwrap();

        let shallow = referenced_names(&config, &[]);
        let nested = referenced_names(&config, &["body"]);

        assert_eq!(
            shallow.into_iter().collect::<Vec<_>>(),
            vec!["Destinataire".to_string(), "Devis".to_string()]
        );
        assert_eq!(
            nested.into_iter().collect::<Vec<_>>(),
            vec![
                "Destinataire".to_string(),
                "Devis".to_string(),
                "Profond".to_string()
            ]
        );
    }

    #[test]
    fn the_view_holds_triggers_loop_items_and_the_outputs_that_are_available() {
        let graph = Graph {
            connectors: vec![
                connector("c1", Some("Un")),
                connector("c2", Some("Boucle")),
                connector("c3", Some("Absent")),
            ],
            edges: vec![],
            triggers: vec![trigger("t1", Some("Départ"))],
        };
        let names: BTreeSet<String> = ["Un", "Boucle", "Absent", "Départ", "Inconnu"]
            .into_iter()
            .map(str::to_owned)
            .collect();

        let view = NodeIndex::new(&graph).view(
            &names,
            "reader",
            &json!({ "t": 1 }),
            |id| (id == "c2").then(|| json!(5)),
            |id| (id == "c1").then(|| StepItems::Main(vec![json!({ "a": 1 })])),
        );

        assert_eq!(view["Un"], vec![json!({ "a": 1 })]);
        assert_eq!(view["Boucle"], vec![json!({ "valeur": 5 })]);
        assert_eq!(view["Départ"], vec![json!({ "t": 1 })]);
        assert!(!view.contains_key("Absent"));
        assert!(!view.contains_key("Inconnu"));
    }

    #[test]
    fn the_view_of_a_branching_node_is_the_branch_that_leads_to_the_reader() {
        let graph = Graph {
            connectors: vec![
                connector("cond", Some("Seuil")),
                connector("yes", None),
                connector("no", None),
            ],
            edges: vec![
                edge("cond", "yes", Some(Branch::Then)),
                edge("cond", "no", Some(Branch::Else)),
            ],
            triggers: vec![],
        };
        let names: BTreeSet<String> = ["Seuil"].into_iter().map(str::to_owned).collect();
        let index = NodeIndex::new(&graph);
        let output = || {
            Some(StepItems::Branched(vec![
                (Branch::Then, vec![json!({ "n": 5 })]),
                (Branch::Else, vec![json!({ "n": 1 })]),
            ]))
        };

        let from_yes = index.view(&names, "yes", &json!({}), |_| None, |_| output());
        let from_no = index.view(&names, "no", &json!({}), |_| None, |_| output());

        assert_eq!(from_yes["Seuil"], vec![json!({ "n": 5 })]);
        assert_eq!(from_no["Seuil"], vec![json!({ "n": 1 })]);
    }
}
