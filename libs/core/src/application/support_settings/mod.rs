use common::CoreError;
use mestier_macros::transactional;

use crate::{
    DisplayFields, GetDisplayFieldsCommand, ReplaceDisplayFieldsCommand, SupportSettingsRepository,
    application::{MestierUseCase, support_ticket::authorize},
};

#[cfg(test)]
pub(crate) mod tests;

impl MestierUseCase {
    #[transactional(support_settings)]
    pub async fn get_support_display_fields(
        &self,
        command: GetDisplayFieldsCommand,
    ) -> Result<DisplayFields, CoreError> {
        let mut support_settings_repository = support_settings_repository;

        support_settings_repository
            .display_fields(command.organization_id)
            .await
    }

    #[transactional(support_settings, member, role, authz)]
    pub async fn replace_support_display_fields(
        &self,
        command: ReplaceDisplayFieldsCommand,
    ) -> Result<DisplayFields, CoreError> {
        let mut support_settings_repository = support_settings_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;

        authorize(
            authz,
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        support_settings_repository
            .replace_display_fields(command.organization_id, &command.display_fields)
            .await
    }
}
