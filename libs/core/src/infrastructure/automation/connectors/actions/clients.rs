use std::str::FromStr;

use chrono::{DateTime, Utc};
use common::CoreError;
use rust_decimal::Decimal;
use serde_json::{Map, Value, json};
use uuid::Uuid;

use super::{ActionContext, failure};
use crate::domain::customer::commands::UpdateCustomerCommand;
use crate::domain::customer_contact::commands::{
    CreateCustomerContactCommand, UpdateCustomerContactCommand,
};
use crate::domain::customer_context::commands::{
    CreateCustomerContextCommand, UpdateCustomerContextCommand,
};
use crate::{
    Customer, CustomerContact, CustomerContactId, CustomerContext, CustomerContextId, CustomerId,
    CustomerPipelineStage, CustomerStatus,
};

#[cfg(test)]
#[path = "tests_clients.rs"]
pub(super) mod tests_clients;

const DEFAULT_LIMIT: u64 = 50;
const MAX_LIMIT: u64 = 100;

pub(super) async fn run(kind: &str, context: &ActionContext<'_>) -> Option<Result<Value, String>> {
    Some(match kind {
        "mestier.customer.get" => get_customer(context).await,
        "mestier.customer.list" => list_customers(context).await,
        "mestier.customer.update" => update_customer(context).await,
        "mestier.customer.delete" => delete_customer(context).await,
        "mestier.customer_contact.list" => list_contacts(context).await,
        "mestier.customer_contact.get" => get_contact(context).await,
        "mestier.customer_contact.create" => create_contact(context).await,
        "mestier.customer_contact.update" => update_contact(context).await,
        "mestier.customer_contact.delete" => delete_contact(context).await,
        "mestier.customer_context.list" => list_contexts(context).await,
        "mestier.customer_context.get" => get_context(context).await,
        "mestier.customer_context.create" => create_context(context).await,
        "mestier.customer_context.update" => update_context(context).await,
        "mestier.customer_context.delete" => delete_context(context).await,
        _ => return None,
    })
}

pub(super) fn not_found() -> String {
    failure(CoreError::NotFound)
}

pub(super) fn page(context: &ActionContext<'_>) -> Result<(u64, u64), String> {
    let limit = context
        .optional_whole_number("limit")?
        .unwrap_or(DEFAULT_LIMIT)
        .clamp(1, MAX_LIMIT);
    let offset = context.optional_whole_number("offset")?.unwrap_or(0);
    Ok((limit, offset))
}

pub(super) fn select<T: FromStr<Err = String>>(
    context: &ActionContext<'_>,
    name: &str,
) -> Result<T, String> {
    T::from_str(context.text(name)?.trim())
}

pub(super) fn optional_select<T: FromStr<Err = String>>(
    context: &ActionContext<'_>,
    name: &str,
) -> Result<Option<T>, String> {
    context.optional_text(name)?.map(T::from_str).transpose()
}

pub(super) fn instant(raw: &str, name: &str) -> Result<DateTime<Utc>, String> {
    DateTime::parse_from_rfc3339(raw.trim())
        .map(|parsed| parsed.with_timezone(&Utc))
        .map_err(|_| format!("`{name}` must be an RFC 3339 date and time"))
}

pub(super) fn optional_instant(
    context: &ActionContext<'_>,
    name: &str,
) -> Result<Option<DateTime<Utc>>, String> {
    context
        .optional_text(name)?
        .map(|raw| instant(raw, name))
        .transpose()
}

fn whole_i32(value: &Value, name: &str) -> Result<i32, String> {
    let invalid = || format!("`{name}` must be a whole number");
    match value {
        Value::Number(number) => number
            .as_i64()
            .and_then(|parsed| i32::try_from(parsed).ok())
            .ok_or_else(invalid),
        Value::String(text) => text.trim().parse().map_err(|_| invalid()),
        _ => Err(invalid()),
    }
}

pub(super) fn integer(context: &ActionContext<'_>, name: &str) -> Result<i32, String> {
    match context.config.get(name) {
        None | Some(Value::Null) => Err(format!("`{name}` must be a whole number")),
        Some(value) => whole_i32(value, name),
    }
}

pub(super) fn array<'a>(context: &'a ActionContext<'_>, name: &str) -> Result<&'a [Value], String> {
    match context.config.get(name) {
        Some(Value::Array(items)) => Ok(items),
        _ => Err(format!("`{name}` must resolve to an array")),
    }
}

pub(super) fn optional_array<'a>(
    context: &'a ActionContext<'_>,
    name: &str,
) -> Result<&'a [Value], String> {
    match context.config.get(name) {
        None | Some(Value::Null) => Ok(&[]),
        Some(Value::Array(items)) => Ok(items),
        Some(_) => Err(format!("`{name}` must resolve to an array")),
    }
}

pub(super) struct Row<'a> {
    object: &'a Map<String, Value>,
    array: &'a str,
    position: usize,
}

impl<'a> Row<'a> {
    pub(super) fn new(item: &'a Value, array: &'a str, position: usize) -> Result<Self, String> {
        item.as_object()
            .map(|object| Self {
                object,
                array,
                position,
            })
            .ok_or_else(|| format!("`{array}` element {position} must be an object"))
    }

    fn problem(&self, name: &str, expected: &str) -> String {
        format!(
            "`{}` element {}: `{name}` must be {expected}",
            self.array, self.position
        )
    }

    pub(super) fn text(&self, name: &str) -> Result<&'a str, String> {
        self.optional_text(name)?
            .ok_or_else(|| self.problem(name, "a string"))
    }

    pub(super) fn optional_text(&self, name: &str) -> Result<Option<&'a str>, String> {
        match self.object.get(name) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::String(value)) if value.trim().is_empty() => Ok(None),
            Some(Value::String(value)) => Ok(Some(value.trim())),
            Some(_) => Err(self.problem(name, "a string")),
        }
    }

    pub(super) fn optional_uuid(&self, name: &str) -> Result<Option<Uuid>, String> {
        self.optional_text(name)?
            .map(|raw| Uuid::parse_str(raw).map_err(|_| self.problem(name, "a UUID")))
            .transpose()
    }

    pub(super) fn integer(&self, name: &str) -> Result<i32, String> {
        self.optional_integer(name)?
            .ok_or_else(|| self.problem(name, "a whole number"))
    }

    pub(super) fn optional_integer(&self, name: &str) -> Result<Option<i32>, String> {
        match self.object.get(name) {
            None | Some(Value::Null) => Ok(None),
            Some(value) => whole_i32(value, name)
                .map(Some)
                .map_err(|_| self.problem(name, "a whole number")),
        }
    }

    pub(super) fn decimal(&self, name: &str) -> Result<Decimal, String> {
        let raw = match self.object.get(name) {
            Some(Value::Number(number)) => number.to_string(),
            Some(Value::String(text)) => text.trim().to_owned(),
            _ => return Err(self.problem(name, "a decimal number")),
        };
        Decimal::from_str(&raw).map_err(|_| self.problem(name, "a decimal number"))
    }

    pub(super) fn bool_or(&self, name: &str, default: bool) -> Result<bool, String> {
        match self.object.get(name) {
            None | Some(Value::Null) => Ok(default),
            Some(Value::Bool(value)) => Ok(*value),
            Some(_) => Err(self.problem(name, "true or false")),
        }
    }

    pub(super) fn strings(&self, name: &str) -> Result<Vec<String>, String> {
        match self.object.get(name) {
            None | Some(Value::Null) => Ok(Vec::new()),
            Some(Value::Array(items)) => items
                .iter()
                .map(|item| {
                    item.as_str()
                        .map(str::to_owned)
                        .ok_or_else(|| self.problem(name, "an array of strings"))
                })
                .collect(),
            Some(_) => Err(self.problem(name, "an array of strings")),
        }
    }

    pub(super) fn uuids(&self, name: &str) -> Result<Vec<Uuid>, String> {
        self.strings(name)?
            .iter()
            .map(|raw| Uuid::parse_str(raw.trim()).map_err(|_| self.problem(name, "UUIDs")))
            .collect()
    }

    pub(super) fn optional_instant(&self, name: &str) -> Result<Option<DateTime<Utc>>, String> {
        self.optional_text(name)?
            .map(|raw| instant(raw, name).map_err(|_| self.problem(name, "an RFC 3339 date")))
            .transpose()
    }
}

pub(super) async fn customer_of_run(
    context: &ActionContext<'_>,
    id: CustomerId,
) -> Result<Customer, String> {
    let customer = context.usecase.get_customer(id).await.map_err(failure)?;
    if customer.organization_id != context.organization_id {
        return Err(not_found());
    }
    Ok(customer)
}

pub(super) async fn context_of_run(
    context: &ActionContext<'_>,
    id: CustomerContextId,
) -> Result<CustomerContext, String> {
    let found = context
        .usecase
        .get_customer_context(id)
        .await
        .map_err(failure)?;
    customer_of_run(context, found.customer_id).await?;
    Ok(found)
}

pub(super) async fn contact_of_run(
    context: &ActionContext<'_>,
    id: CustomerContactId,
) -> Result<CustomerContact, String> {
    let found = context
        .usecase
        .get_customer_contact(id)
        .await
        .map_err(failure)?;
    customer_of_run(context, found.customer_id).await?;
    Ok(found)
}

pub(super) async fn targets_of_run(
    context: &ActionContext<'_>,
    customer_id: CustomerId,
    customer_context_id: CustomerContextId,
) -> Result<(), String> {
    customer_of_run(context, customer_id).await?;
    let found = context_of_run(context, customer_context_id).await?;
    if found.customer_id != customer_id {
        return Err(not_found());
    }
    Ok(())
}

fn customer_json(customer: &Customer) -> Value {
    json!({
        "id": customer.id.0,
        "organization_id": customer.organization_id.0,
        "status": customer.status.as_str(),
        "pipeline_stage": customer.pipeline_stage.as_str(),
        "name": customer.name,
        "registration_number": customer.registration_number,
        "phone": customer.phone,
        "email": customer.email,
        "created_at": customer.created_at,
        "updated_at": customer.updated_at,
    })
}

fn contact_json(contact: &CustomerContact) -> Value {
    json!({
        "id": contact.id.0,
        "customer_id": contact.customer_id.0,
        "first_name": contact.first_name,
        "last_name": contact.last_name,
        "role": contact.role,
        "phone": contact.phone,
        "email": contact.email,
        "is_primary": contact.is_primary,
        "created_at": contact.created_at,
        "updated_at": contact.updated_at,
    })
}

fn context_json(found: &CustomerContext) -> Value {
    json!({
        "id": found.id.0,
        "customer_id": found.customer_id.0,
        "label": found.label,
        "address_line": found.address_line,
        "postal_code": found.postal_code,
        "city": found.city,
        "photo_key": found.photo_key,
        "created_at": found.created_at,
        "updated_at": found.updated_at,
    })
}

fn customer_id(context: &ActionContext<'_>) -> Result<CustomerId, String> {
    context.uuid("customer_id").map(CustomerId)
}

fn owned(value: Option<&str>) -> Option<String> {
    value.map(str::to_owned)
}

async fn get_customer(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = customer_id(context)?;

    customer_of_run(context, id)
        .await
        .map(|customer| customer_json(&customer))
}

async fn list_customers(context: &ActionContext<'_>) -> Result<Value, String> {
    let (limit, offset) = page(context)?;

    context
        .usecase
        .list_customers(context.organization_id, limit, offset)
        .await
        .map(|(customers, _)| Value::Array(customers.iter().map(customer_json).collect()))
        .map_err(failure)
}

async fn update_customer(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = customer_id(context)?;
    let status = select::<CustomerStatus>(context, "status")?;
    let pipeline_stage = select::<CustomerPipelineStage>(context, "pipeline_stage")?;
    let name = context.text("name")?.to_owned();
    let registration_number = owned(context.optional_text("registration_number")?);
    let phone = owned(context.optional_text("phone")?);
    let email = owned(context.optional_text("email")?);
    customer_of_run(context, id).await?;

    context
        .usecase
        .update_customer(UpdateCustomerCommand {
            actor: context.actor(),
            id,
            status,
            pipeline_stage,
            name,
            registration_number,
            phone,
            email,
        })
        .await
        .map(|customer| customer_json(&customer))
        .map_err(failure)
}

async fn delete_customer(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = customer_id(context)?;
    customer_of_run(context, id).await?;

    context
        .usecase
        .soft_delete_customer(id, context.actor())
        .await
        .map_err(failure)?;

    Ok(json!({ "customer_id": id.0, "deleted": true }))
}

async fn list_contacts(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = customer_id(context)?;
    let (limit, offset) = page(context)?;
    customer_of_run(context, id).await?;

    context
        .usecase
        .list_customer_contacts(id, limit, offset)
        .await
        .map(|(contacts, _)| Value::Array(contacts.iter().map(contact_json).collect()))
        .map_err(failure)
}

async fn get_contact(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = context.uuid("contact_id").map(CustomerContactId)?;

    contact_of_run(context, id)
        .await
        .map(|contact| contact_json(&contact))
}

async fn create_contact(context: &ActionContext<'_>) -> Result<Value, String> {
    let customer_id = customer_id(context)?;
    let first_name = context.text("first_name")?.to_owned();
    let last_name = context.text("last_name")?.to_owned();
    let role = owned(context.optional_text("role")?);
    let phone = owned(context.optional_text("phone")?);
    let email = owned(context.optional_text("email")?);
    let is_primary = context.optional_bool("is_primary")?.unwrap_or(false);
    customer_of_run(context, customer_id).await?;

    context
        .usecase
        .create_customer_contact(CreateCustomerContactCommand {
            actor: context.actor(),
            customer_id,
            first_name,
            last_name,
            role,
            phone,
            email,
            is_primary,
        })
        .await
        .map(|contact| contact_json(&contact))
        .map_err(failure)
}

async fn update_contact(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = context.uuid("contact_id").map(CustomerContactId)?;
    let first_name = context.text("first_name")?.to_owned();
    let last_name = context.text("last_name")?.to_owned();
    let role = owned(context.optional_text("role")?);
    let phone = owned(context.optional_text("phone")?);
    let email = owned(context.optional_text("email")?);
    let is_primary = context.optional_bool("is_primary")?.unwrap_or(false);
    contact_of_run(context, id).await?;

    context
        .usecase
        .update_customer_contact(UpdateCustomerContactCommand {
            actor: context.actor(),
            id,
            first_name,
            last_name,
            role,
            phone,
            email,
            is_primary,
        })
        .await
        .map(|contact| contact_json(&contact))
        .map_err(failure)
}

async fn delete_contact(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = context.uuid("contact_id").map(CustomerContactId)?;
    contact_of_run(context, id).await?;

    context
        .usecase
        .soft_delete_customer_contact(id, context.actor())
        .await
        .map_err(failure)?;

    Ok(json!({ "contact_id": id.0, "deleted": true }))
}

async fn list_contexts(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = customer_id(context)?;
    let (limit, offset) = page(context)?;
    customer_of_run(context, id).await?;

    context
        .usecase
        .list_customer_contexts(id, limit, offset)
        .await
        .map(|(found, _)| Value::Array(found.iter().map(context_json).collect()))
        .map_err(failure)
}

async fn get_context(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = context.uuid("context_id").map(CustomerContextId)?;

    context_of_run(context, id)
        .await
        .map(|found| context_json(&found))
}

async fn create_context(context: &ActionContext<'_>) -> Result<Value, String> {
    let customer_id = customer_id(context)?;
    let label = context.text("label")?.to_owned();
    let address_line = owned(context.optional_text("address_line")?);
    let postal_code = owned(context.optional_text("postal_code")?);
    let city = owned(context.optional_text("city")?);
    let photo_key = owned(context.optional_text("photo_key")?);
    customer_of_run(context, customer_id).await?;

    context
        .usecase
        .create_customer_context(CreateCustomerContextCommand {
            actor: context.actor(),
            customer_id,
            label,
            address_line,
            postal_code,
            city,
            photo_key,
        })
        .await
        .map(|found| context_json(&found))
        .map_err(failure)
}

async fn update_context(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = context.uuid("context_id").map(CustomerContextId)?;
    let label = context.text("label")?.to_owned();
    let address_line = owned(context.optional_text("address_line")?);
    let postal_code = owned(context.optional_text("postal_code")?);
    let city = owned(context.optional_text("city")?);
    let photo_key = owned(context.optional_text("photo_key")?);
    context_of_run(context, id).await?;

    context
        .usecase
        .update_customer_context(UpdateCustomerContextCommand {
            actor: context.actor(),
            id,
            label,
            address_line,
            postal_code,
            city,
            photo_key,
        })
        .await
        .map(|found| context_json(&found))
        .map_err(failure)
}

async fn delete_context(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = context.uuid("context_id").map(CustomerContextId)?;
    context_of_run(context, id).await?;

    context
        .usecase
        .soft_delete_customer_context(id, context.actor())
        .await
        .map_err(failure)?;

    Ok(json!({ "context_id": id.0, "deleted": true }))
}
