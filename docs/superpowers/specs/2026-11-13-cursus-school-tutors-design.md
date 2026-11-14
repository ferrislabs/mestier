# Cursus: school tutors — design

## Intent
A triad is three people: the apprentice, the company tutor (on the contract) and the school tutor, usually a staff member. Today the triad's "Tuteur pédagogique" is any organization member, with no directory of school tutors. The Tuteurs page must list both kinds, and creating a tutor starts by choosing company or school.

## Decisions (approved with a mockup)
- A school tutor is an organization **member** declared as tutor, with a job title (Fonction). It is not a participant (ADR 0015): no invitation, the person uses the console.
- Tuteurs page: two tabs, "Entreprise · N" (current list) and "École · N". École tab: search, Actifs / Archivés / Tous, columns Nom, Fonction, Trinômes suivis (count of open triads where they are the school tutor), Statut.
- "Nouveau tuteur": first a choice between "Tuteur entreprise" (existing 3-step flow, invitation) and "Tuteur école" (pick a member not yet declared, Fonction; "Créer le tuteur").
- Triad dialog: the field is labelled "Tuteur école" and offers only active school tutors; the company tutor stays on the contract.
- Members already set as pedagogic tutor on a triad become school tutors automatically (migration backfill).
- Archiving a school tutor keeps them on the triads they already follow; they disappear from the triad picker.

## Data
`cursus.school_tutor (id, org_id, member_id, job_title NULL, archived_at NULL, created_at, updated_at, UNIQUE (org_id, member_id))`, composite FK `(org_id, member_id)` → members. `cursus.triad.pedagogic_tutor_member_id` stays; a composite FK `(org_id, pedagogic_tutor_member_id)` → `school_tutor (org_id, member_id)` enforces it. The up migration backfills school tutors before adding the FK; the down migration drops the FK and the table.
