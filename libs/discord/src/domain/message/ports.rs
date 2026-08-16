use common::CoreError;

use crate::{ChannelId, Message, MessageId, Reaction, UserId};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait MessageRepository: Send {
    fn insert(&mut self, m: &Message) -> impl Future<Output = Result<Message, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: MessageId,
    ) -> impl Future<Output = Result<Option<Message>, CoreError>> + Send;

    fn list_by_channel(
        &mut self,
        channel: ChannelId,
        before: Option<MessageId>,
        after: Option<MessageId>,
        limit: u64,
    ) -> impl Future<Output = Result<Vec<Message>, CoreError>> + Send;

    fn update(&mut self, m: &Message) -> impl Future<Output = Result<Message, CoreError>> + Send;

    fn delete(&mut self, id: MessageId) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn add_reaction(&mut self, r: &Reaction) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn remove_reaction(
        &mut self,
        message_id: MessageId,
        emoji: &str,
        user_id: UserId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn list_reactions(
        &mut self,
        message_id: MessageId,
    ) -> impl Future<Output = Result<Vec<Reaction>, CoreError>> + Send;
}
