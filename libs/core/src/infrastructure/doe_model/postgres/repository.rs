//! Skeleton from W0. W3a implements it, see docs/chantiers/doe.md.

use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
    DoeModel, DoeModelId, OrganizationId, domain::doe_model::ports::DoeModelRepository,
    infrastructure::postgres::SharedTx,
};

#[repository(domain = DoeModel, backend = Postgres)]
pub struct PgDoeModelRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgDoeModelRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> DoeModelRepository for PgDoeModelRepository<'tx> {
    async fn insert(&mut self, _model: &DoeModel) -> Result<DoeModel, CoreError> {
        let _ = &self.tx;
        todo!("W3a")
    }

    async fn find_by_id(&mut self, _id: DoeModelId) -> Result<Option<DoeModel>, CoreError> {
        todo!("W3a")
    }

    async fn list_by_organization(
        &mut self,
        _organization_id: OrganizationId,
        _include_archived: bool,
        _limit: u64,
        _offset: u64,
    ) -> Result<(Vec<DoeModel>, u64), CoreError> {
        todo!("W3a")
    }

    async fn update(&mut self, _model: &DoeModel) -> Result<DoeModel, CoreError> {
        todo!("W3a")
    }

    async fn set_archived_at(
        &mut self,
        _id: DoeModelId,
        _archived_at: Option<DateTime<Utc>>,
    ) -> Result<(), CoreError> {
        todo!("W3a")
    }
}
