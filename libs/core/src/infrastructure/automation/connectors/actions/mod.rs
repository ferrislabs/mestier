use common::{CoreError, OrganizationId};
use serde_json::{Map, Value};
use uuid::Uuid;

use super::fields;
use crate::application::MestierUseCase;
use crate::domain::automation::connector::{ConnectorDescriptor, find_action};
use crate::domain::automation::run::{ConnectorInput, ConnectorOutcome};

mod achats;
mod clients;
mod devis;
mod discussions;
mod factures;
mod membres;
mod planning_a;
mod planning_b;
mod referentiel;
mod reporting;
mod support;
mod tables;
mod terrain;

pub struct ActionContext<'a> {
    pub usecase: &'a MestierUseCase,
    pub organization_id: OrganizationId,
    pub config: &'a Map<String, Value>,
}

impl ActionContext<'_> {
    pub(super) fn actor(&self) -> authz::Subject {
        authz::Subject::system()
    }

    pub(super) async fn system_user_id(&self) -> Result<common::UserId, String> {
        self.usecase.system_user_id().await.map_err(failure)
    }

    pub(super) fn uuid(&self, name: &str) -> Result<Uuid, String> {
        fields::uuid(self.config, name)
    }

    pub(super) fn optional_uuid(&self, name: &str) -> Result<Option<Uuid>, String> {
        fields::optional_text(self.config, name)?
            .map(|raw| Uuid::parse_str(raw).map_err(|_| format!("`{name}` must be a UUID")))
            .transpose()
    }

    pub(super) fn text(&self, name: &str) -> Result<&str, String> {
        fields::text(self.config, name)
    }

    pub(super) fn filled(&self, name: &str, max_chars: usize) -> Result<&str, String> {
        fields::filled(self.config, name, max_chars)
    }

    pub(super) fn optional_text(&self, name: &str) -> Result<Option<&str>, String> {
        fields::optional_text(self.config, name)
    }

    pub(super) fn optional_whole_number(&self, name: &str) -> Result<Option<u64>, String> {
        fields::optional_whole_number(self.config, name)
    }

    pub(super) fn optional_bool(&self, name: &str) -> Result<Option<bool>, String> {
        match self.config.get(name) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::Bool(value)) => Ok(Some(*value)),
            Some(Value::String(value)) if value.trim().is_empty() => Ok(None),
            Some(Value::String(value)) if value.trim() == "true" => Ok(Some(true)),
            Some(Value::String(value)) if value.trim() == "false" => Ok(Some(false)),
            Some(_) => Err(format!("`{name}` must be true or false")),
        }
    }

    pub(super) fn optional_object(&self, name: &str) -> Result<Option<Value>, String> {
        match self.config.get(name) {
            None | Some(Value::Null) => Ok(None),
            Some(value @ Value::Object(_)) => Ok(Some(value.clone())),
            Some(_) => Err(format!("`{name}` must resolve to an object")),
        }
    }

    pub(super) fn optional_strings(&self, name: &str) -> Result<Vec<&str>, String> {
        match self.config.get(name) {
            None | Some(Value::Null) => Ok(Vec::new()),
            Some(Value::Array(items)) => items
                .iter()
                .enumerate()
                .map(|(position, item)| {
                    item.as_str()
                        .ok_or_else(|| format!("`{name}` element {position} must be a string"))
                })
                .collect(),
            Some(_) => Err(format!("`{name}` must resolve to an array")),
        }
    }
}

pub(super) fn failure(error: CoreError) -> String {
    match error {
        CoreError::NotFound => {
            "not found: the targeted object does not exist in this organization".to_owned()
        }
        other => other.to_string(),
    }
}

pub(super) async fn execute(
    usecase: &MestierUseCase,
    kind: &str,
    input: ConnectorInput<'_>,
) -> Option<ConnectorOutcome> {
    let acting = usecase.acting_as_automation(input.run_id);
    let context = ActionContext {
        usecase: &acting,
        organization_id: input.org_id,
        config: input.config,
    };

    Some(match run(kind, context).await? {
        Ok(output) => ConnectorOutcome::Produced(output),
        Err(error) => ConnectorOutcome::Failed { error },
    })
}

pub(super) fn dry_run(descriptor: &ConnectorDescriptor) -> Option<ConnectorOutcome> {
    find_action(descriptor.kind)?;
    Some(ConnectorOutcome::Produced(descriptor.sample_output.clone()))
}

async fn run(kind: &str, context: ActionContext<'_>) -> Option<Result<Value, String>> {
    find_action(kind)?;
    if let Some(result) = support::run(kind, &context).await {
        return Some(result);
    }
    if let Some(result) = devis::run(kind, &context).await {
        return Some(result);
    }
    if let Some(result) = clients::run(kind, &context).await {
        return Some(result);
    }
    if let Some(result) = factures::run(kind, &context).await {
        return Some(result);
    }
    if let Some(result) = reporting::run(kind, &context).await {
        return Some(result);
    }
    if let Some(result) = discussions::run(kind, &context).await {
        return Some(result);
    }
    if let Some(result) = achats::run(kind, &context).await {
        return Some(result);
    }
    if let Some(result) = terrain::run(kind, &context).await {
        return Some(result);
    }
    if let Some(result) = planning_a::run(kind, &context).await {
        return Some(result);
    }
    if let Some(result) = planning_b::run(kind, &context).await {
        return Some(result);
    }
    if let Some(result) = referentiel::run(kind, &context).await {
        return Some(result);
    }
    if let Some(result) = membres::run(kind, &context).await {
        return Some(result);
    }
    if let Some(result) = tables::run(kind, &context).await {
        return Some(result);
    }
    None
}

#[cfg(test)]
mod tests;
