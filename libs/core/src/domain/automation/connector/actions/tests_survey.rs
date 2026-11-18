use super::*;
use crate::domain::automation::connector::find_action;

fn names(spec: &ActionSpec) -> Vec<&'static str> {
    spec.fields.iter().map(|field| field.name).collect()
}

#[test]
fn the_survey_family_offers_exactly_the_four_actions_of_the_spec() {
    let kinds: Vec<&str> = SPECS.iter().map(|spec| spec.kind).collect();

    assert_eq!(
        kinds,
        [
            "mestier.survey.open",
            "mestier.survey.close",
            "mestier.survey.get_link",
            "mestier.survey.list_responses",
        ]
    );
    for kind in kinds {
        assert!(find_action(kind).is_some(), "{kind}");
    }
}

#[test]
fn every_survey_action_targets_a_survey_and_is_grouped_under_questionnaires() {
    for spec in SPECS {
        assert_eq!(spec.group, "Questionnaires", "{}", spec.kind);
        assert_eq!(spec.fields[0].name, "survey_id", "{}", spec.kind);
        assert!(spec.fields[0].required, "{}", spec.kind);
        assert!(!spec.label.is_empty() && !spec.description.is_empty());
        assert!(spec.fields.iter().all(|field| !field.label.is_empty()));
    }
}

#[test]
fn reads_are_flagged_read_only_and_transitions_are_not() {
    for spec in SPECS {
        let reads = spec.kind.ends_with(".get_link") || spec.kind.ends_with(".list_responses");
        assert_eq!(spec.read_only, reads, "{}", spec.kind);
    }
}

#[test]
fn each_action_asks_for_its_own_fields() {
    let fields = |kind: &str| names(SPECS.iter().find(|spec| spec.kind == kind).unwrap());

    assert_eq!(fields("mestier.survey.open"), ["survey_id", "campaign_id"]);
    assert_eq!(fields("mestier.survey.close"), ["survey_id"]);
    assert_eq!(fields("mestier.survey.get_link"), ["survey_id"]);
    assert_eq!(
        fields("mestier.survey.list_responses"),
        ["survey_id", "limit", "offset"]
    );
}

#[test]
fn the_listed_answers_sample_the_entries_of_the_completed_event() {
    let sample = responses_sample();
    let answer = &sample["responses"][0]["answers"][0];

    let mut keys: Vec<&str> = answer
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();

    assert_eq!(keys, ["kind", "label", "question_id", "value"]);
    assert!(sample["responses"][0].get("completed_at").is_some());
    assert!(sample["responses"][0].get("identifier_value").is_some());
    assert_eq!(sample["total"], 1);
}

#[test]
fn the_responses_field_labels_tell_the_order_the_default_and_the_cap() {
    let spec = SPECS
        .iter()
        .find(|spec| spec.kind == "mestier.survey.list_responses")
        .unwrap();
    let limit = spec
        .fields
        .iter()
        .find(|field| field.name == "limit")
        .unwrap();

    assert!(limit.label.contains("les plus récentes d’abord"));
    assert!(limit.label.contains("50") && limit.label.contains("100"));
    assert!(spec.description.contains("les plus récentes d’abord"));
}

#[test]
fn the_link_sample_names_the_public_path() {
    let sample = link_sample();
    let slug =
        crate::domain::survey::PublicSlug::new(sample["public_slug"].as_str().unwrap().to_owned())
            .unwrap();

    assert_eq!(sample["path"], slug.path());
}
