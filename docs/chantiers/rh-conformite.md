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
| B1 | Profil RH du membre, archivage | F2 | à faire |
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
