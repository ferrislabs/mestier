use std::sync::Arc;

use common::CoreError;
use sqlx::{PgPool, Postgres, Transaction};
use tokio::sync::{Mutex, MutexGuard};

use crate::infrastructure::postgres::error::map_sqlx_error;

/// Cloneable handle over a live transaction so several repositories can share
/// it without juggling exclusive `&mut` borrows. Each repository takes its own
/// clone and locks the inner transaction only for the duration of one query.
/// Contention is effectively zero (a use-case runs sequentially on one task)
/// — the mutex exists to satisfy the borrow checker while keeping the
/// hexagonal split: the service stays unaware of `tx`.
pub struct SharedTx<'tx> {
    inner: Arc<Mutex<&'tx mut Transaction<'static, Postgres>>>,
}

impl<'tx> SharedTx<'tx> {
    fn new(tx: &'tx mut Transaction<'static, Postgres>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(tx)),
        }
    }

    pub async fn lock(&self) -> MutexGuard<'_, &'tx mut Transaction<'static, Postgres>> {
        self.inner.lock().await
    }
}

impl<'tx> Clone for SharedTx<'tx> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

pub async fn with_tx<F, T>(pool: &PgPool, work: F) -> Result<T, CoreError>
where
    F: AsyncFnOnce(SharedTx<'_>) -> Result<T, CoreError>,
{
    let mut tx = pool.begin().await.map_err(map_sqlx_error)?;

    let result = {
        let shared = SharedTx::new(&mut tx);
        work(shared).await
        // all `SharedTx` clones drop here, releasing the &mut borrow on `tx`
    };

    match result {
        Ok(value) => {
            tx.commit().await.map_err(map_sqlx_error)?;
            Ok(value)
        }
        Err(err) => {
            let _ = tx.rollback().await;
            Err(err)
        }
    }
}
