//! Skeleton from W0. W2 implements it, see docs/chantiers/doe.md.

use common::CoreError;
use mestier_macros::repository;

use crate::{
    OrganizationId, Supplier, SupplierId, domain::supplier_invoice::ports::SupplierRepository,
    infrastructure::postgres::SharedTx,
};

#[repository(domain = Supplier, backend = Postgres)]
pub struct PgSupplierRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgSupplierRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> SupplierRepository for PgSupplierRepository<'tx> {
    async fn insert(&mut self, _supplier: &Supplier) -> Result<Supplier, CoreError> {
        let _ = &self.tx;
        todo!("W2")
    }

    async fn find_by_id(&mut self, _id: SupplierId) -> Result<Option<Supplier>, CoreError> {
        todo!("W2")
    }

    async fn list_by_organization(
        &mut self,
        _organization_id: OrganizationId,
        _limit: u64,
        _offset: u64,
    ) -> Result<(Vec<Supplier>, u64), CoreError> {
        todo!("W2")
    }

    async fn update(&mut self, _supplier: &Supplier) -> Result<Supplier, CoreError> {
        todo!("W2")
    }
}
