# 0019. Organizations turn self-service features on and off themselves

Status: proposed
Date: 2026-11-15

Partially amends 0017: only the Mestier staff could enable a flag there.

## Context

The product owner wants an organization to open the questionnaires module from its own settings, without asking the Mestier staff. ADR 0017 declares the flags in code, as the closed enum `FeatureFlag`, and stores the activations in `organization_feature_flags`. Some flags must stay staff-only: `drive_ai` costs money per call and is opened by hand. The organization settings are guarded by `MANAGE_ORG` (`organization.update` in the policy engine).

## Decision

`FeatureFlag::is_self_service()` declares, per variant and in code, whether an organization may toggle the flag itself; `surveys` is self-service, `drive_ai` is not. A member holding `MANAGE_ORG` toggles a self-service flag with `PUT /api/v1/organizations/{organization_id}/features/{flag}`; `GET /api/v1/organizations/{organization_id}/features` also returns the self-service flags with their label, description and state. The use case writes the same activation rows and emits the same `organization.feature_enabled` / `organization.feature_disabled` events as the staff path, attributed to the member. A non-self-service or unknown key, an unknown organization and an organization the caller does not belong to all answer 404. Turning a flag off deletes nothing; the module is hidden and its API answers 404, as before.

## Alternatives rejected

- Every flag self-service: a flag the staff opens on purpose, such as a paid or experimental one, could be switched on by any organization.
- A `self_service` column on a flag table: there is no flag table (0017), and a database value could mark as self-service a flag the running code never meant to open.
- A per-organization permission table listing which flags each organization may toggle: no current need, one more table and back-office screen for the same yes/no the code already answers.

## Consequences

- Gains: the questionnaires module is opened without the staff; one activation table and one event pair for both paths, so the back-office history shows organization changes too.
- Costs: making a flag self-service, or taking that back, takes a deploy.
- What becomes harder: a staff-enabled flag later made self-service can be turned off by the organization; the staff loses the last word on it.
