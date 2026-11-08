use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use super::{
    Surcharge, SurchargeId, SurchargeInput, SurchargeOutcome, commands::SurchargeDraft,
    default_surcharges, ports::SurchargeRepository,
};
use crate::{OrganizationId, domain::absence_type::staggered};

impl SurchargeDraft {
    pub fn as_input(&self) -> SurchargeInput<'_> {
        SurchargeInput {
            name: &self.name,
            rate_bp: self.rate_bp,
            kind: self.kind,
        }
    }
}

pub struct SurchargeService<S>
where
    S: SurchargeRepository,
{
    surcharges: S,
}

impl<S> SurchargeService<S>
where
    S: SurchargeRepository,
{
    pub fn new(surcharges: S) -> Self {
        Self { surcharges }
    }

    pub async fn list(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<Surcharge>, CoreError> {
        self.surcharges.lock_organization(organization_id).await?;
        let stored = self
            .surcharges
            .list_by_organization(organization_id)
            .await?;
        if !stored.is_empty() {
            return Ok(stored);
        }
        let base = Utc::now();
        for (index, default) in default_surcharges().iter().enumerate() {
            let surcharge = Surcharge::new(
                SurchargeId(generate_uuid_v7()),
                organization_id,
                SurchargeInput::from(default),
                staggered(base, index),
            )
            .map_err(|error| {
                CoreError::Internal(format!("default surcharge is invalid: {error}"))
            })?;
            self.surcharges.insert(&surcharge).await?;
        }
        self.surcharges.list_by_organization(organization_id).await
    }

    pub async fn find(
        &mut self,
        organization_id: OrganizationId,
        id: SurchargeId,
    ) -> Result<Surcharge, CoreError> {
        self.surcharges
            .find(organization_id, id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    pub async fn create(
        &mut self,
        organization_id: OrganizationId,
        draft: &SurchargeDraft,
    ) -> Result<SurchargeOutcome<Surcharge>, CoreError> {
        self.list(organization_id).await?;
        let surcharge = match Surcharge::new(
            SurchargeId(generate_uuid_v7()),
            organization_id,
            draft.as_input(),
            Utc::now(),
        ) {
            Ok(surcharge) => surcharge,
            Err(refusal) => return Ok(Err(refusal)),
        };
        self.surcharges.insert(&surcharge).await.map(Ok)
    }

    pub async fn update(
        &mut self,
        organization_id: OrganizationId,
        id: SurchargeId,
        draft: &SurchargeDraft,
    ) -> Result<SurchargeOutcome<Surcharge>, CoreError> {
        let mut surcharge = self.find(organization_id, id).await?;
        if let Err(refusal) = surcharge.revise(draft.as_input()) {
            return Ok(Err(refusal));
        }
        self.surcharges.update(&surcharge).await.map(Ok)
    }

    pub async fn set_archived(
        &mut self,
        organization_id: OrganizationId,
        id: SurchargeId,
        archived: bool,
    ) -> Result<Surcharge, CoreError> {
        let mut surcharge = self.find(organization_id, id).await?;
        if archived {
            surcharge.archive(Utc::now());
        } else {
            surcharge.restore();
        }
        self.surcharges.update(&surcharge).await
    }
}
