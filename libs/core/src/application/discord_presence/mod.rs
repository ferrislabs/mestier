use common::CoreError;
use discord::{OrganizationId, Presence, PresenceService, SetPresenceCommand, StartTypingCommand};
use mestier_macros::transactional;

use crate::application::MestierUseCase;

impl MestierUseCase {
    #[transactional(presence)]
    pub async fn set_presence(&self, cmd: SetPresenceCommand) -> Result<Presence, CoreError> {
        let org_id = cmd.organization_id;
        let mut service = PresenceService::new(presence_repository, self.events.as_ref());
        let result = service.set_presence(cmd).await?;
        // best-effort flush at end of tx closure; events are reconciled via REST (spec §2)
        self.events.flush(org_id);
        Ok(result)
    }

    #[transactional(presence)]
    pub async fn list_presence(&self, org: OrganizationId) -> Result<Vec<Presence>, CoreError> {
        let mut service = PresenceService::new(presence_repository, self.events.as_ref());
        service.list_presence(org).await
    }

    /// `start_typing` does NOT write to the DB — it only publishes a TypingStarted event.
    /// We still open a (no-op) transaction to satisfy the macro, but the service
    /// issues no repo calls.
    #[transactional(presence)]
    pub async fn start_typing(&self, cmd: StartTypingCommand) -> Result<(), CoreError> {
        let org_id = cmd.organization_id;
        let mut service = PresenceService::new(presence_repository, self.events.as_ref());
        service.start_typing(cmd).await?;
        self.events.flush(org_id);
        Ok(())
    }
}
