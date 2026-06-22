use common::CoreError;

use crate::domain::message::ports::MessageRepository;
use crate::events::EventPublisher;

use super::commands::MarkChannelReadCommand;
use super::ports::ReadStateRepository;

pub struct ReadStateService<R, M, E> {
    read_state: R,
    messages: M,
    events: E,
}

impl<R, M, E> ReadStateService<R, M, E>
where
    R: ReadStateRepository,
    M: MessageRepository,
    E: EventPublisher,
{
    pub fn new(read_state: R, messages: M, events: E) -> Self {
        Self {
            read_state,
            messages,
            events,
        }
    }

    pub async fn mark_channel_read(
        &self,
        command: MarkChannelReadCommand,
    ) -> Result<(), CoreError> {
        let _ = command;
        unimplemented!("implemented in Task 4")
    }
}
