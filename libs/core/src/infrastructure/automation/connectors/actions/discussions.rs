use common::{CoreError, RoleId, UserId};
use discord::{
    AddReactionCommand, Category, CategoryId, Channel, ChannelId, ChannelPermissionOverwrite,
    ChannelType, CreateCategoryCommand, CreateChannelCommand, CreateThreadCommand,
    CreateWebhookCommand, DeleteChannelOverwrite, Message, MessageId, OverwriteTarget,
    RemoveReactionCommand, UpdateCategoryCommand, UpdateChannelCommand, UpdateThreadCommand,
    UpdateWebhookCommand, UpsertChannelOverwrite, Webhook, WebhookId,
};
use serde::Serialize;
use serde_json::{Value, json};

use super::{ActionContext, failure};

#[cfg(test)]
mod tests_discussions;

const TOKEN_NOTE: &str = "The webhook token is not exposed in automation output.";
const MAX_NAME_CHARS: usize = 100;
const MAX_TOPIC_CHARS: usize = 1_000;
const MAX_CONTENT_CHARS: usize = 4_000;
const MAX_EMOJI_CHARS: usize = 64;
const MAX_URL_CHARS: usize = 2_000;
const DEFAULT_MESSAGES: u64 = 50;
const MAX_MESSAGES: u64 = 100;

pub(super) async fn run(kind: &str, context: &ActionContext<'_>) -> Option<Result<Value, String>> {
    Some(match kind {
        "mestier.discussions.list_categories" => list_categories(context).await,
        "mestier.discussions.create_category" => create_category(context).await,
        "mestier.discussions.update_category" => update_category(context).await,
        "mestier.discussions.delete_category" => delete_category(context).await,
        "mestier.discussions.list_channels" => list_channels(context).await,
        "mestier.discussions.get_channel" => get_channel(context).await,
        "mestier.discussions.create_channel" => create_channel(context).await,
        "mestier.discussions.update_channel" => update_channel(context).await,
        "mestier.discussions.delete_channel" => delete_channel(context).await,
        "mestier.discussions.list_threads" => list_threads(context).await,
        "mestier.discussions.create_thread" => create_thread(context).await,
        "mestier.discussions.update_thread" => update_thread(context).await,
        "mestier.discussions.delete_thread" => delete_thread(context).await,
        "mestier.discussions.list_messages" => list_messages(context).await,
        "mestier.discussions.edit_message" => edit_message(context).await,
        "mestier.discussions.delete_message" => delete_message(context).await,
        "mestier.discussions.list_reactors" => list_reactors(context).await,
        "mestier.discussions.list_webhooks" => list_webhooks(context).await,
        "mestier.discussions.update_webhook" => update_webhook(context).await,
        "mestier.discussions.delete_webhook" => delete_webhook(context).await,
        "mestier.discussions.list_channel_permissions" => list_permissions(context).await,
        "mestier.discussions.set_everyone_permissions" => set_everyone(context).await,
        "mestier.discussions.remove_everyone_permissions" => remove_everyone(context).await,
        "mestier.discussions.set_target_permissions" => set_target(context).await,
        "mestier.discussions.remove_target_permissions" => remove_target(context).await,
        "mestier.discussions.create_webhook" => create_webhook(context).await,
        "mestier.discussions.add_reaction" => add_reaction(context).await,
        "mestier.discussions.remove_reaction" => remove_reaction(context).await,
        _ => return None,
    })
}

fn to_json(value: &impl Serialize) -> Result<Value, String> {
    serde_json::to_value(value).map_err(|error| error.to_string())
}

fn many(values: &[impl Serialize]) -> Result<Value, String> {
    values
        .iter()
        .map(to_json)
        .collect::<Result<Vec<_>, _>>()
        .map(Value::Array)
}

fn webhook_json(webhook: &Webhook) -> Value {
    json!({
        "id": webhook.id,
        "organization_id": webhook.organization_id,
        "channel_id": webhook.channel_id,
        "name": webhook.name,
        "avatar_url": webhook.avatar_url,
        "created_by": webhook.created_by,
        "created_at": webhook.created_at,
        "updated_at": webhook.updated_at,
    })
}

fn created_webhook_json(webhook: &Webhook) -> Value {
    let mut value = webhook_json(webhook);
    value["note"] = json!(TOKEN_NOTE);
    value
}

fn overwrite_json(overwrite: &ChannelPermissionOverwrite) -> Value {
    let (target_type, target_id) = target_parts(overwrite.target);
    json!({
        "target_type": target_type,
        "target_id": target_id,
        "allow": overwrite.allow,
        "deny": overwrite.deny,
        "created_at": overwrite.created_at,
        "updated_at": overwrite.updated_at,
    })
}

fn target_parts(target: OverwriteTarget) -> (&'static str, Option<uuid::Uuid>) {
    match target {
        OverwriteTarget::Everyone => ("everyone", None),
        OverwriteTarget::Role(role) => ("role", Some(role.0)),
        OverwriteTarget::Member(user) => ("member", Some(user.0)),
    }
}

fn window<T>(context: &ActionContext<'_>, items: Vec<T>) -> Result<Vec<T>, String> {
    let limit = context.optional_whole_number("limit")?;
    let offset = context.optional_whole_number("offset")?.unwrap_or(0);
    let skip = usize::try_from(offset).unwrap_or(usize::MAX);
    let take = limit.map_or(usize::MAX, |limit| {
        usize::try_from(limit).unwrap_or(usize::MAX)
    });
    Ok(items.into_iter().skip(skip).take(take).collect())
}

fn position(context: &ActionContext<'_>) -> Result<Option<i32>, String> {
    context
        .optional_whole_number("position")?
        .map(|position| i32::try_from(position).map_err(|_| "`position` is too large".to_owned()))
        .transpose()
}

fn bits(context: &ActionContext<'_>, name: &str) -> Result<i64, String> {
    let raw = context
        .optional_whole_number(name)?
        .ok_or_else(|| format!("`{name}` must resolve to a whole number"))?;
    i64::try_from(raw).map_err(|_| format!("`{name}` is too large"))
}

fn not_found() -> String {
    failure(CoreError::NotFound)
}

async fn category_of(context: &ActionContext<'_>, id: CategoryId) -> Result<Category, String> {
    let category = context.usecase.get_category(id).await.map_err(failure)?;
    if category.organization_id != context.organization_id {
        return Err(not_found());
    }
    Ok(category)
}

async fn channel_of(context: &ActionContext<'_>, id: ChannelId) -> Result<Channel, String> {
    let channel = context.usecase.get_channel(id).await.map_err(failure)?;
    if channel.organization_id != context.organization_id {
        return Err(not_found());
    }
    Ok(channel)
}

async fn thread_of(context: &ActionContext<'_>, id: ChannelId) -> Result<Channel, String> {
    let thread = channel_of(context, id).await?;
    if thread.channel_type != ChannelType::Thread {
        return Err(not_found());
    }
    Ok(thread)
}

async fn message_of(context: &ActionContext<'_>, id: MessageId) -> Result<Message, String> {
    let message = context.usecase.get_message(id).await.map_err(failure)?;
    if message.organization_id != context.organization_id {
        return Err(not_found());
    }
    Ok(message)
}

async fn webhook_of(context: &ActionContext<'_>, id: WebhookId) -> Result<Webhook, String> {
    let webhook = context.usecase.get_webhook(id).await.map_err(failure)?;
    if webhook.organization_id != context.organization_id {
        return Err(not_found());
    }
    Ok(webhook)
}

fn category_id(context: &ActionContext<'_>) -> Result<CategoryId, String> {
    context.uuid("category_id").map(CategoryId)
}

fn channel_id(context: &ActionContext<'_>) -> Result<ChannelId, String> {
    context.uuid("channel_id").map(ChannelId)
}

fn thread_id(context: &ActionContext<'_>) -> Result<ChannelId, String> {
    context.uuid("thread_id").map(ChannelId)
}

fn message_id(context: &ActionContext<'_>) -> Result<MessageId, String> {
    context.uuid("message_id").map(MessageId)
}

fn webhook_id(context: &ActionContext<'_>) -> Result<WebhookId, String> {
    context.uuid("webhook_id").map(WebhookId)
}

async fn optional_category(
    context: &ActionContext<'_>,
) -> Result<Option<Option<CategoryId>>, String> {
    match context.optional_uuid("category_id")? {
        Some(raw) => {
            let id = CategoryId(raw);
            category_of(context, id).await?;
            Ok(Some(Some(id)))
        }
        None => Ok(None),
    }
}

async fn list_categories(context: &ActionContext<'_>) -> Result<Value, String> {
    context.optional_whole_number("limit")?;
    context.optional_whole_number("offset")?;

    let categories = context
        .usecase
        .list_categories(context.organization_id)
        .await
        .map_err(failure)?;

    many(&window(context, categories)?)
}

async fn create_category(context: &ActionContext<'_>) -> Result<Value, String> {
    let name = context.filled("name", MAX_NAME_CHARS)?.to_owned();
    let position = position(context)?.unwrap_or(0);

    let category = context
        .usecase
        .create_category(CreateCategoryCommand {
            organization_id: context.organization_id,
            name,
            position,
        })
        .await
        .map_err(failure)?;

    to_json(&category)
}

async fn update_category(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = category_id(context)?;
    let name = context.optional_text("name")?.map(str::to_owned);
    let position = position(context)?;
    let existing = category_of(context, id).await?;

    let category = context
        .usecase
        .update_category(UpdateCategoryCommand {
            id,
            name: name.unwrap_or(existing.name),
            position: position.unwrap_or(existing.position),
        })
        .await
        .map_err(failure)?;

    to_json(&category)
}

async fn delete_category(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = category_id(context)?;
    category_of(context, id).await?;

    context.usecase.delete_category(id).await.map_err(failure)?;

    Ok(json!({ "category_id": id.0, "deleted": true }))
}

async fn list_channels(context: &ActionContext<'_>) -> Result<Value, String> {
    context.optional_whole_number("limit")?;
    context.optional_whole_number("offset")?;

    let channels = context
        .usecase
        .list_channels(context.organization_id)
        .await
        .map_err(failure)?;

    many(&window(context, channels)?)
}

async fn get_channel(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = channel_id(context)?;

    to_json(&channel_of(context, id).await?)
}

async fn create_channel(context: &ActionContext<'_>) -> Result<Value, String> {
    let name = context.filled("name", MAX_NAME_CHARS)?.to_owned();
    let topic = context.optional_text("topic")?.map(str::to_owned);
    if topic
        .as_deref()
        .is_some_and(|topic| topic.chars().count() > MAX_TOPIC_CHARS)
    {
        return Err(format!(
            "`topic` holds at most {MAX_TOPIC_CHARS} characters"
        ));
    }
    let position = position(context)?.unwrap_or(0);
    let category_id = optional_category(context).await?.flatten();

    let channel = context
        .usecase
        .create_channel(CreateChannelCommand {
            organization_id: context.organization_id,
            channel_type: ChannelType::Text,
            category_id,
            name,
            topic,
            position,
        })
        .await
        .map_err(failure)?;

    to_json(&channel)
}

async fn update_channel(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = channel_id(context)?;
    let name = context.optional_text("name")?.map(str::to_owned);
    let topic = context.optional_text("topic")?.map(str::to_owned);
    if topic
        .as_deref()
        .is_some_and(|topic| topic.chars().count() > MAX_TOPIC_CHARS)
    {
        return Err(format!(
            "`topic` holds at most {MAX_TOPIC_CHARS} characters"
        ));
    }
    let position = position(context)?;
    let existing = channel_of(context, id).await?;
    let category_id = match optional_category(context).await? {
        Some(category) => category,
        None => existing.category_id,
    };

    let channel = context
        .usecase
        .update_channel(UpdateChannelCommand {
            id,
            category_id,
            name: name.unwrap_or(existing.name),
            topic: topic.or(existing.topic),
            position: position.unwrap_or(existing.position),
        })
        .await
        .map_err(failure)?;

    to_json(&channel)
}

async fn delete_channel(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = channel_id(context)?;
    channel_of(context, id).await?;

    context.usecase.delete_channel(id).await.map_err(failure)?;

    Ok(json!({ "channel_id": id.0, "deleted": true }))
}

async fn list_threads(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = channel_id(context)?;
    channel_of(context, id).await?;

    let threads = context.usecase.list_threads(id).await.map_err(failure)?;

    many(&threads)
}

async fn create_thread(context: &ActionContext<'_>) -> Result<Value, String> {
    let parent_id = channel_id(context)?;
    let name = context.filled("name", MAX_NAME_CHARS)?.to_owned();
    let origin_message_id = context.optional_uuid("origin_message_id")?.map(MessageId);
    channel_of(context, parent_id).await?;
    if let Some(origin) = origin_message_id {
        message_of(context, origin).await?;
    }

    let thread = context
        .usecase
        .create_thread(CreateThreadCommand {
            organization_id: context.organization_id,
            parent_id,
            origin_message_id,
            name,
        })
        .await
        .map_err(failure)?;

    to_json(&thread)
}

async fn update_thread(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = thread_id(context)?;
    let name = context.optional_text("name")?.map(str::to_owned);
    let archived = context.optional_bool("archived")?;
    let existing = thread_of(context, id).await?;

    let thread = context
        .usecase
        .update_thread(UpdateThreadCommand {
            id,
            name: name.unwrap_or(existing.name),
            archived: archived.unwrap_or(existing.archived),
        })
        .await
        .map_err(failure)?;

    to_json(&thread)
}

async fn delete_thread(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = thread_id(context)?;
    thread_of(context, id).await?;

    context.usecase.delete_thread(id).await.map_err(failure)?;

    Ok(json!({ "thread_id": id.0, "deleted": true }))
}

async fn list_messages(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = channel_id(context)?;
    let before = context.optional_uuid("before")?.map(MessageId);
    let after = context.optional_uuid("after")?.map(MessageId);
    let limit = context
        .optional_whole_number("limit")?
        .unwrap_or(DEFAULT_MESSAGES)
        .clamp(1, MAX_MESSAGES);
    channel_of(context, id).await?;

    let messages = context
        .usecase
        .list_messages(id, before, after, limit)
        .await
        .map_err(failure)?;

    many(&messages)
}

async fn edit_message(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = message_id(context)?;
    let content = context.filled("content", MAX_CONTENT_CHARS)?.to_owned();
    message_of(context, id).await?;

    let message = context
        .usecase
        .edit_system_message(id, content)
        .await
        .map_err(failure)?;

    to_json(&message)
}

async fn delete_message(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = message_id(context)?;
    message_of(context, id).await?;

    context.usecase.delete_message(id).await.map_err(failure)?;

    Ok(json!({ "message_id": id.0, "deleted": true }))
}

async fn list_reactors(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = message_id(context)?;
    let emoji = context.filled("emoji", MAX_EMOJI_CHARS)?;
    let message = message_of(context, id).await?;

    let reactors: Vec<UserId> = message
        .reactions
        .into_iter()
        .find(|reaction| reaction.emoji == emoji)
        .map(|reaction| reaction.user_ids)
        .unwrap_or_default();

    Ok(Value::Array(
        reactors
            .iter()
            .map(|user_id| json!({ "user_id": user_id }))
            .collect(),
    ))
}

async fn list_webhooks(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = channel_id(context)?;
    channel_of(context, id).await?;

    let webhooks = context.usecase.list_webhooks(id).await.map_err(failure)?;

    Ok(Value::Array(webhooks.iter().map(webhook_json).collect()))
}

async fn update_webhook(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = webhook_id(context)?;
    let name = context.optional_text("name")?.map(str::to_owned);
    let avatar_url = context.optional_text("avatar_url")?.map(str::to_owned);
    if avatar_url
        .as_deref()
        .is_some_and(|url| url.chars().count() > MAX_URL_CHARS)
    {
        return Err(format!(
            "`avatar_url` holds at most {MAX_URL_CHARS} characters"
        ));
    }
    let existing = webhook_of(context, id).await?;

    let webhook = context
        .usecase
        .update_webhook(UpdateWebhookCommand {
            id,
            name: name.unwrap_or(existing.name),
            avatar_url: avatar_url.or(existing.avatar_url),
        })
        .await
        .map_err(failure)?;

    Ok(webhook_json(&webhook))
}

async fn delete_webhook(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = webhook_id(context)?;
    webhook_of(context, id).await?;

    context.usecase.delete_webhook(id).await.map_err(failure)?;

    Ok(json!({ "webhook_id": id.0, "deleted": true }))
}

async fn list_permissions(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = channel_id(context)?;
    channel_of(context, id).await?;

    let overwrites = context
        .usecase
        .list_channel_overwrites(id)
        .await
        .map_err(failure)?;

    Ok(Value::Array(
        overwrites.iter().map(overwrite_json).collect(),
    ))
}

fn parse_target(context: &ActionContext<'_>) -> Result<OverwriteTarget, String> {
    let kind = context.text("target_type")?;
    let raw = context.uuid("target_id")?;
    match kind {
        "role" => Ok(OverwriteTarget::Role(RoleId(raw))),
        "member" => Ok(OverwriteTarget::Member(UserId(raw))),
        _ => Err("`target_type` must be `role` or `member`".to_owned()),
    }
}

async fn require_target_in_organization(
    context: &ActionContext<'_>,
    target: OverwriteTarget,
) -> Result<(), String> {
    let known = match target {
        OverwriteTarget::Everyone => true,
        OverwriteTarget::Role(role) => context
            .usecase
            .list_roles(context.organization_id, context.actor())
            .await
            .map_err(failure)?
            .iter()
            .any(|candidate| candidate.id == role),
        OverwriteTarget::Member(user) => context
            .usecase
            .find_membership(context.organization_id, user)
            .await
            .map_err(failure)?
            .is_some(),
    };
    if known { Ok(()) } else { Err(not_found()) }
}

async fn upsert(
    context: &ActionContext<'_>,
    channel_id: ChannelId,
    target: OverwriteTarget,
) -> Result<Value, String> {
    let allow = bits(context, "allow")?;
    let deny = bits(context, "deny")?;
    channel_of(context, channel_id).await?;

    let overwrite = context
        .usecase
        .upsert_channel_overwrite(UpsertChannelOverwrite {
            channel_id,
            organization_id: context.organization_id,
            target,
            allow,
            deny,
        })
        .await
        .map_err(failure)?;

    Ok(overwrite_json(&overwrite))
}

async fn remove(
    context: &ActionContext<'_>,
    channel_id: ChannelId,
    target: OverwriteTarget,
) -> Result<Value, String> {
    context
        .usecase
        .delete_channel_overwrite(DeleteChannelOverwrite { channel_id, target })
        .await
        .map_err(failure)?;

    let (target_type, target_id) = target_parts(target);
    Ok(json!({
        "channel_id": channel_id.0,
        "target_type": target_type,
        "target_id": target_id,
        "removed": true,
    }))
}

async fn set_everyone(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = channel_id(context)?;
    upsert(context, id, OverwriteTarget::Everyone).await
}

async fn remove_everyone(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = channel_id(context)?;
    channel_of(context, id).await?;
    remove(context, id, OverwriteTarget::Everyone).await
}

async fn set_target(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = channel_id(context)?;
    let target = parse_target(context)?;
    bits(context, "allow")?;
    bits(context, "deny")?;
    channel_of(context, id).await?;
    require_target_in_organization(context, target).await?;
    upsert(context, id, target).await
}

async fn remove_target(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = channel_id(context)?;
    let target = parse_target(context)?;
    channel_of(context, id).await?;
    remove(context, id, target).await
}

async fn create_webhook(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = channel_id(context)?;
    let name = context.filled("name", MAX_NAME_CHARS)?.to_owned();
    let avatar_url = context.optional_text("avatar_url")?.map(str::to_owned);
    if avatar_url
        .as_deref()
        .is_some_and(|url| url.chars().count() > MAX_URL_CHARS)
    {
        return Err(format!(
            "`avatar_url` holds at most {MAX_URL_CHARS} characters"
        ));
    }
    let channel = channel_of(context, id).await?;
    let created_by = context.system_user_id().await?;

    let webhook = context
        .usecase
        .create_webhook(CreateWebhookCommand {
            organization_id: channel.organization_id,
            channel_id: id,
            name,
            avatar_url,
            created_by,
        })
        .await
        .map_err(failure)?;

    Ok(created_webhook_json(&webhook))
}

async fn add_reaction(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = message_id(context)?;
    let emoji = context.filled("emoji", MAX_EMOJI_CHARS)?.to_owned();
    message_of(context, id).await?;
    let user_id = context.system_user_id().await?;

    context
        .usecase
        .add_reaction(AddReactionCommand {
            message_id: id,
            emoji: emoji.clone(),
            user_id,
        })
        .await
        .map_err(failure)?;

    Ok(json!({ "message_id": id.0, "emoji": emoji, "reacted": true }))
}

async fn remove_reaction(context: &ActionContext<'_>) -> Result<Value, String> {
    let id = message_id(context)?;
    let emoji = context.filled("emoji", MAX_EMOJI_CHARS)?.to_owned();
    message_of(context, id).await?;
    let user_id = context.system_user_id().await?;

    context
        .usecase
        .remove_reaction(RemoveReactionCommand {
            message_id: id,
            emoji: emoji.clone(),
            user_id,
        })
        .await
        .map_err(failure)?;

    Ok(json!({ "message_id": id.0, "emoji": emoji, "reacted": false }))
}
