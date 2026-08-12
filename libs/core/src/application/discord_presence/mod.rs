use common::CoreError;
use discord::{OrganizationId, Presence, PresenceService, SetPresenceCommand, StartTypingCommand};
use mestier_macros::transactional;

use crate::application::MestierUseCase;

impl MestierUseCase {
    #[transactional(presence, events)]
    pub async fn set_presence(&self, cmd: SetPresenceCommand) -> Result<Presence, CoreError> {
        let mut service = PresenceService::new(presence_repository, &events);
        let result = service.set_presence(cmd).await?;
        Ok(result)
    }

    #[transactional(presence, events)]
    pub async fn list_presence(&self, org: OrganizationId) -> Result<Vec<Presence>, CoreError> {
        let mut service = PresenceService::new(presence_repository, &events);
        service.list_presence(org).await
    }

    /// `start_typing` does NOT write to the DB — it only publishes a TypingStarted event.
    /// We still open a (no-op) transaction to satisfy the macro, but the service
    /// issues no repo calls.
    #[transactional(presence, events)]
    pub async fn start_typing(&self, cmd: StartTypingCommand) -> Result<(), CoreError> {
        let mut service = PresenceService::new(presence_repository, &events);
        service.start_typing(cmd).await?;
        Ok(())
    }
}
