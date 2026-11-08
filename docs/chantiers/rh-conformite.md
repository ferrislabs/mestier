# RH : conformité au prototype

Objectif : le module RH du vrai Mestier est identique, en fonctions et en UI, au module RH du prototype `mestier-rework` (`src/modules/rh`). Le prototype fait foi. Spec : `docs/specs/rh-conformity.md`. ADR 0014.

## Décisions (2026-11-08)
- Les écrans du prototype sont déjà dans `pages/hr/ui/rh` (#594) sur un store `localStorage` lié aux vrais membres et aux vraies absences. Reste le portage de la coque (URLs `/rh/*`, barre latérale, liste et fiche membre) puis le remplacement du mock par l'API, un lot à la fois.
- Les fonctions du réel absentes du prototype sont gardées et restylées : Temps de travail, Matériel, historique de coût, invitation, rôles.
- Soldes, planning et récapitulatifs restent calculés dans le webapp (ADR 0014).
- Exécution autonome de nuit : les questions du spec sont tranchées ici, sans lot d'attente. Branches empilées, une PR par lot.

## Lots (branches empilées sur `main`)
`feature/rh-front` (F1 + F2) sur main, puis `backend/rh-b1-profile`, `b2-contracts`, `b3-absences`, `b4-accounts`, `b5-planning`, `b6-extra`, chacune sur la précédente. Base de travail Postgres : `mestier_rh` dans le conteneur local, jamais la base de développement.

| Lot | Contenu | Dépend de | État |
|---|---|---|---|
| F1 | Coque : URLs `/rh/*`, redirections `/hr`, barre latérale, compteurs, routes « nouvelle » | – | fait |
| F2 | Liste et fiche membre, invitation et rôles conservés | F1 | fait |
| B1 | Profil RH du membre, archivage | F2 | fait |
| B2 | Contrats, horaires, documents | B1 | à faire |
| B3 | Motifs d'absence et circuit de validation | B2 | à faire |
| B4 | Compteurs et ajustements | B3 | à faire |
| B5 | Exceptions de jour et flux du planning | B2, B3 | à faire |
| B6 | Majorations et heures extra | B3 | à faire |
| Fin | Suppression de `rh-store`, `rh-seed`, `rh-binding` | B6 | à faire |

## Mocks front à remplacer
`pages/hr/feature/mocks/{rh-store,rh-seed,rh-binding}.ts`, découpés par lot : profil (B1), contrats et documents (B2), motifs et absences (B3), compteurs (B4), exceptions (B5), majorations et extras (B6).

## Reports
## Reports issus de F1
- Les clés de recherche passent en français (`onglet`, `statut`, `membre`, `compte`, `vue`, `mois`). La redirection `/hr/**` renomme chemins et clés ; un `/hr` inconnu va sur `/rh`.
- Les compteurs (absences et extras en attente) vivent dans la barre latérale dépliée ; le rail replié n'en montre pas (pas de source `ModuleSection.badge`).
- `moduleLandingPath` ignore `requiredPermission` : l'atterrissage `/rh/membres` est caché sans `MANAGE_MEMBERS`.
- `app-breadcrumb.tsx` garde des regex CRM sur les anciens chemins `/crm/customers` et `/crm/quotes`.
- `INITIAL_RH_STATE` compte 8 motifs d'absence, le prototype 7 ; les autres écarts de seed ne sont pas vérifiés.

## Reports issus de F2
- La liste affiche `fullName` (prénom nom) comme le prototype, pas `display_name` (nom prénom) de l'API.
- Le formulaire de création garde la création réelle (membre + profil de coût), pas le premier contrat du prototype : B2 le reprendra.
- `tsc --noEmit` : 43 erreurs avant, 38 après (tests supprimés), aucune nouvelle.
- B1 remplace `status`, archivage, restauration, `managerId`, poste et e-mails du store ; garder `RhScreen` comme porte de chargement. Si la fiche clignote « introuvable » après création, attendre `isBound`.
- Pas de contrôle visuel : pas d'authentification possible en autonomie.

## Reports issus de B1
- Table `member_hr_profiles` séparée de `employees` : celle-ci a des colonnes contractuelles `NOT NULL`, un membre sans coût doit pouvoir avoir un profil.
- Newtype `ContactEmail` ajouté ; `list_active_by_organization` écarte les membres archivés (plus de planning pour eux).
- Les endpoints de profil répondent 422 avec `details.field` et le code `E_MEMBER_PROFILE_MANAGER_CYCLE` pour un cycle de responsables.
- Ajout hors périmètre initial : `GET /organizations/{org}/member-profiles` (liste de tous les profils, membres sans profil absents, test `spec_rh_1_5_*`). Les deux lectures de profil exigent `member.manage` ; un 403 sur la liste n'empêche pas l'écran, le front retombe sur les défauts en lecture seule.
- Front : `hooks/use-member-profiles.ts` (liste, PUT, archivage, restauration, invalidation), `lib/rh/member-profile.ts` (overlay pur, corps de requête), `feature/use-profile-actions.ts`. Le store superpose le profil API à chaque membre ; un membre sans profil reçoit les défauts vides (e-mail pro = e-mail du compte), jamais les valeurs du prototype.
- Réducteurs `updateMember`, `archiveMember`, `restoreMember` supprimés ; `cancelUpcomingAbsences` reste dans le store (absences encore mockées).
- `POST .../archive` accepte `{reassign_reports_to: member_id|null}` : les subordonnés directs actifs passent au nouveau responsable (même organisation, sans cycle) dans la transaction de l'archivage, sinon rien n'est écrit (422). Le PUT d'un membre sans profil écrit son e-mail de compte en e-mail pro : accepté.
- Modifier, archiver et réactiver sont masqués sans `MANAGE_MEMBERS` (`RequirePermission`).
- Un échec de la liste des profils autre que 403 bloque l'écran RH (message d'erreur).
- Le front tronque la date d'archivage à la date UTC de `archived_at`.
- Trois tests ignorés du dispatcher d'automatisation (`one_pass_fans_a_committed_event_out_to_a_workflow_run`, `a_disabled_subscription_receives_no_run`, `a_disabled_workflow_receives_no_run_but_its_event_is_still_done_with`) échouent dans le run ignoré complet sur la base partagée. Même échec sur le commit de base `31cbfbd6` (avec trois autres tests de `postgres::run` instables sur la base partagée) : antérieur à B1, sans lien avec lui.
- `tsc --noEmit` : 38 erreurs avant et après.
