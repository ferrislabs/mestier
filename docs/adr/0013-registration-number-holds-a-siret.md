# 0013. The registration number holds a SIRET, the SIREN is derived

Status: proposed
Date: 2026-11-07

## Context
The rework labels the identity number SIRET (14 digits). The backend stores `registration_number` and the Factur-X writer sends it under scheme `0002` (SIRENE, the 9-digit SIREN) in `infrastructure/invoice/facturx/cii.rs`. Existing rows hold 9 digits.

## Decision
`registration_number` accepts 14 digits (SIRET) or the 9 digits already stored. The Factur-X writer sends the first 9 digits under scheme `0002`. No data migration.

## Alternatives rejected
- Add a separate `siret` column: two fields for one identity, and every consumer must choose.
- Send the SIRET under scheme `0009`: changes what receivers key on for domestic invoices.

## Consequences
- Gains: the UI label matches the stored value without touching existing rows.
- Costs: a derivation at the Factur-X boundary, and a validation that accepts two lengths.
- What becomes harder: nothing in the domain; the SIREN is always the prefix.
