use serde_json::Value;

use super::actions;
use super::descriptor::{AuthRequirement, ConnectorDescriptor, ExecutionMode};
use super::field::Field;

pub const ACTION_FAMILY: &str = "mestier";

pub struct ActionSpec {
    pub kind: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub group: &'static str,
    pub fields: &'static [Field],
    pub read_only: bool,
    pub sample_output: fn() -> Value,
}

impl ActionSpec {
    pub fn descriptor(&self) -> ConnectorDescriptor {
        let sample = (self.sample_output)();
        let example = match &sample {
            Value::Array(items) => items.first().cloned().unwrap_or(Value::Null),
            other => other.clone(),
        };
        ConnectorDescriptor {
            kind: self.kind,
            version: 1,
            family: ACTION_FAMILY,
            label: self.label,
            auth: AuthRequirement::None,
            fields: self.fields,
            branches: &[],
            execution: ExecutionMode::PerItem,
            output_example: example,
            output_mirrors_field: None,
            group: Some(self.group),
            description: Some(self.description),
            sample_output: sample,
        }
    }
}

pub fn action_specs() -> impl Iterator<Item = &'static ActionSpec> {
    actions::TABLES.iter().flat_map(|table| table.iter())
}

pub fn find_action(kind: &str) -> Option<&'static ActionSpec> {
    action_specs().find(|spec| spec.kind == kind)
}

pub fn action_descriptors() -> Vec<ConnectorDescriptor> {
    action_specs().map(ActionSpec::descriptor).collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::domain::automation::connector::connector_catalogue;

    #[test]
    fn every_action_kind_is_unique_and_namespaced() {
        let mut seen = BTreeSet::new();
        for spec in action_specs() {
            assert!(spec.kind.starts_with("mestier."), "{}", spec.kind);
            assert_eq!(spec.kind.split('.').count(), 3, "{}", spec.kind);
            assert!(seen.insert(spec.kind), "duplicate kind {}", spec.kind);
        }
    }

    #[test]
    fn every_action_is_labelled_described_and_grouped() {
        for spec in action_specs() {
            assert!(!spec.label.is_empty(), "{}", spec.kind);
            assert!(!spec.description.is_empty(), "{}", spec.kind);
            assert!(!spec.group.is_empty(), "{}", spec.kind);
        }
    }

    #[test]
    fn no_action_offers_a_field_naming_the_organization() {
        for spec in action_specs() {
            for field in spec.fields {
                assert!(
                    !field.name.contains("organization"),
                    "{} exposes {}",
                    spec.kind,
                    field.name
                );
            }
        }
    }

    #[test]
    fn an_action_descriptor_carries_its_group_and_its_sample() {
        let spec = find_action("mestier.support.get_ticket").unwrap();

        let descriptor = spec.descriptor();

        assert_eq!(descriptor.kind, "mestier.support.get_ticket");
        assert_eq!(descriptor.family, "mestier");
        assert_eq!(descriptor.group, Some("Support"));
        assert_eq!(descriptor.sample_output, (spec.sample_output)());
        assert_eq!(descriptor.execution, ExecutionMode::PerItem);
        assert_eq!(descriptor.auth, AuthRequirement::None);
    }

    #[test]
    fn a_list_action_takes_the_first_element_of_its_sample_as_example() {
        let descriptor = find_action("mestier.support.list_tickets")
            .unwrap()
            .descriptor();

        assert!(descriptor.sample_output.is_array());
        assert!(descriptor.output_example.is_object());
    }

    #[test]
    fn every_action_is_in_the_catalogue() {
        let catalogue = connector_catalogue();

        for spec in action_specs() {
            let descriptor = catalogue.get(spec.kind, 1).expect(spec.kind);
            assert_eq!(descriptor.group, Some(spec.group));
        }
    }
}
