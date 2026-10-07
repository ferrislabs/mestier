//! Skeleton from W0. W3b implements it, see docs/chantiers/doe.md.

use common::CoreError;
use mestier_macros::repository;

use crate::{
    Doe, DoeId, ProjectId, domain::doe::ports::DoeRepository, infrastructure::postgres::SharedTx,
};

#[repository(domain = Doe, backend = Postgres)]
pub struct PgDoeRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgDoeRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> DoeRepository for PgDoeRepository<'tx> {
    async fn insert(&mut self, _doe: &Doe) -> Result<Doe, CoreError> {
        let _ = &self.tx;
        todo!("W3b")
    }

    async fn find_by_id(&mut self, _id: DoeId) -> Result<Option<Doe>, CoreError> {
        todo!("W3b")
    }

    async fn find_by_project(&mut self, _project_id: ProjectId) -> Result<Option<Doe>, CoreError> {
        todo!("W3b")
    }

    async fn update(&mut self, _doe: &Doe) -> Result<Doe, CoreError> {
        todo!("W3b")
    }
}
