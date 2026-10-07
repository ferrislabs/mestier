//! Skeleton from W0. W3a implements it, see docs/chantiers/doe.md.

use common::CoreError;
use mestier_macros::repository;

use crate::{
    DoeSettings, OrganizationId, domain::doe_model::ports::DoeSettingsRepository,
    infrastructure::postgres::SharedTx,
};

#[repository(domain = DoeSettings, backend = Postgres)]
pub struct PgDoeSettingsRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgDoeSettingsRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> DoeSettingsRepository for PgDoeSettingsRepository<'tx> {
    async fn find(
        &mut self,
        _organization_id: OrganizationId,
    ) -> Result<Option<DoeSettings>, CoreError> {
        let _ = &self.tx;
        todo!("W3a")
    }

    async fn upsert(&mut self, _settings: &DoeSettings) -> Result<DoeSettings, CoreError> {
        todo!("W3a")
    }
}
