use common::CoreError;

use crate::{ChannelId, Webhook, WebhookId};

#[cfg_attr(test, mockall::automock)]
pub trait WebhookRepository: Send {
    fn insert(&mut self, w: &Webhook) -> impl Future<Output = Result<Webhook, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: WebhookId,
    ) -> impl Future<Output = Result<Option<Webhook>, CoreError>> + Send;

    fn list_by_channel(
        &mut self,
        channel: ChannelId,
    ) -> impl Future<Output = Result<Vec<Webhook>, CoreError>> + Send;

    fn update(&mut self, w: &Webhook) -> impl Future<Output = Result<Webhook, CoreError>> + Send;

    fn delete(&mut self, id: WebhookId) -> impl Future<Output = Result<(), CoreError>> + Send;
}
