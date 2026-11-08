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
| B2 | Contrats, horaires, documents | B1 | fait |
| B3 | Motifs d'absence et circuit de validation | B2 | fait |
| B4 | Compteurs et ajustements | B3 | fait |
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

## Reports issus de B2
- Énumérations en SCREAMING_SNAKE sur le fil (`type` : CDI, CDD, APPRENTICESHIP, INTERIM, MANAGER ; `kind` de document : CONTRACT, IDENTITY, DIPLOMA, MEDICAL, OTHER), minuscules du prototype côté modèle : `lib/rh/member-wire.ts` porte les tables de correspondance explicites, testées en aller-retour. Horaire en snake_case, union étiquetée sur `kind` (weekly|rotation|variable).
- Pas de chevauchement de contrats d'un même membre : contrainte d'exclusion (`btree_gist`) en base plus contrôle du service. Le refus est un 422 `E_MEMBER_CONTRACT_OVERLAP` sur `start_date` ; une course perdue contre la contrainte peut revenir en 409. Le front traite les deux comme un chevauchement, affiché près des dates.
- Les refus de validation sont des 422 avec `details.field` : `start_date`, `end_date`, `trial_end_date`, `schedule.week[i]`, `schedule.blocks[i].length`, `schedule.blocks[i].week[j]`, `schedule.default_day`. Les dates s'affichent sous leur champ ; une erreur d'horaire s'affiche dans la fenêtre avec le jour nommé (« Rythme, mercredi : … »), le formulaire d'horaire n'ayant pas d'emplacement par jour.
- CDD, apprentissage (APPRENTICESHIP) et intérim exigent une date de fin ; la fin d'essai doit tomber pendant le contrat.
- Limites des rotations : 1 à 52 semaines ou 1 à 12 mois par séquence, 12 séquences au plus, ancre avant le 29 pour des séquences en mois.
- Risque inter-locataires : `/files` n'a pas de notion de propriétaire et la clé d'un document est fournie par l'appelant. Supprimer le fichier à la suppression de la ligne aurait permis à un gestionnaire de membres de détruire le fichier d'une autre organisation. La suppression d'un document ne supprime donc plus que la ligne (commit `fix(hr): stop deleting stored files when a document row is removed`) ; le fichier reste orphelin. À reprendre quand `/files` portera un propriétaire.
- Lecture : `GET /member-contracts` et `GET /member-documents` exigent `member.manage` ; un 403 laisse l'écran ouvert, sans contrat ni document et sans contrôle d'écriture. Les écritures sont masquées sans `MANAGE_MEMBERS`.
- Front : `hooks/use-member-contracts.ts`, `hooks/use-member-documents.ts`, `feature/use-contract-actions.ts` (écritures puis relecture), surcouche dans `rh-store` (contrats et documents API remplacent ceux du store, jamais les valeurs du prototype). Réducteurs `addContract`, `updateContract`, `removeContract`, `setContractSchedule`, `addDocument`, `removeDocument` supprimés ; documents du jeu de départ retirés.
- Documents : le fichier passe par `POST /files` (`useUploadFile`) puis la ligne est créée avec la clé retournée ; le nom devient un lien via `/files/url` (`useFileUrls`).
- Un membre sans contrat est désormais possible (membre réel créé sans contrat) : badge « Aucun contrat », pas de bouton « Modifier le rythme ».
- Premier contrat à la création : non repris. Le formulaire de création réel n'a aucun champ de contrat (type, dates, horaire) ; en créer un par défaut (CDI à la date du jour) inventerait une donnée contractuelle. Le premier contrat s'ajoute depuis l'onglet Contrats.
- Le contrôle « un membre garde au moins un contrat » reste côté interface seulement ; l'API n'impose rien.
- `tsc --noEmit` : 38 erreurs avant et après.

## Reports issus de B3
- Forme de requête rétrocompatible : l'ancienne création (`kind`, `starts_at`, `ends_at`, `all_day`) reste acceptée, prend le motif par défaut de son `kind` et est validée aussitôt. Les écrans RH n'utilisent que la nouvelle forme (`type_id`, `start_date`, `start_part`, `end_date`, `end_part`, `note`, `proof_file_key`).
- Les absences gagnent les colonnes `start_date` et `end_date` ; `created_by` et `decided_by` sont nullables (absences d'avant B3, validation automatique).
- Seules les absences APPROVED bloquent le planning. Les écrans hors RH qui montrent une disponibilité (aperçu Temps de travail, fiche Temps de travail d'un membre) filtrent donc sur `APPROVED`.
- Le PATCH garde les champs de l'ancienne fenêtre (`starts_at`, `ends_at`, `all_day`) : il sert encore au calendrier de planification, plus à l'écran RH, qui n'a ni modification ni suppression (le prototype n'a que l'annulation).
- Approuver, refuser et annuler n'émettent aucun événement : les absences n'en émettent aucun aujourd'hui.
- Pas de libre-service employé : la création exige `MANAGE_REFERENCE`, donc un employé ne peut pas poser sa propre absence. Le front masque « Poser une absence » (barre d'actions, planning, fiche membre, popover du jour) sans cette permission ; Valider, Refuser et Annuler exigent `MANAGE_PLANNING`. L'annulation par le créateur seul n'est pas offerte : le modèle front ne connaît pas le membre connecté (`actorId` du store est le premier membre).
- 36 tests de base de données échouent ensemble sur la base partagée mais passent isolément (même famille que les instables connus). Le filtre `automation` seul : 1 échec connu.
- Chevauchement : le contrôle du prototype (pas deux absences actives sur les mêmes demi-journées d'un membre) reste dans `lib/rh/absences.ts` côté interface. L'API ne le refuse pas : l'ajouter exige une requête de lecture sur la période et un `query!` ; non fait, à décider avec le spec.
- Motifs : les 7 motifs par défaut sont créés à la première lecture (le prototype en comptait 8 avec « Indisponibilité »). `kind` (LEAVE, SICK, UNAVAILABLE) est requis par l'API et absent du prototype : le formulaire de motif ne le montre pas, un nouveau motif est LEAVE, la modification conserve le `kind` lu.
- Compte débité : `account_id` est un UUID côté API, les comptes de congés restent mockés (id `cp`, `rtt`). Le lien motif vers compte reste donc dans `rh-store` (mémoire, jamais envoyé s'il n'est pas un UUID) et les motifs « Congés payés » et « RTT » sont reliés par leur nom pour garder les soldes. B4 remplace ce lien.
- Justificatif : le fichier passe par `POST /files`, la clé est envoyée en `proof_file_key` et affichée en lien (`/files/url`) sur la demande à valider. Approuver sans justificatif un motif qui l'exige : 422 sur `proof_file_key`, affiché dans le bandeau de l'écran. Refuser sans commentaire : bloqué avant l'appel, et 422 sur `comment` affiché dans la fenêtre.
- Archiver un membre annule désormais ses demandes futures en attente par l'API (`cancel`, au mieux : un échec n'annule pas l'archivage), à la place de l'ancien réducteur du store.
- La liste des absences est lue sur une seule page de 100 (limite de l'API, sans filtre par membre) : au-delà, la fin manque dans l'historique.
- Lecture : une liste de motifs ou d'absences en 403 ouvre l'écran vide, sans bandeau d'erreur ; tout autre échec bloque l'écran.
- Absence d'un motif archivé : l'API le refuse (422 sur `type_id`) ; le formulaire ne propose que les motifs actifs.
- Le client OpenAPI a été régénéré (le précédent datait d'avant B3). `kind`, `starts_at`, `ends_at` de la création sont désormais optionnels dans les types générés.
- `tsc --noEmit` : 38 erreurs avant et après.

## Reports issus de B4
- Les jours sont stockés en centièmes entiers (`acquisition_hundredths`, `carry_over_max_hundredths`, `days_hundredths`) : exacts, jamais arrondis. Le fil porte des nombres à deux décimales au plus ; une troisième décimale est refusée (422).
- Le compte CP par défaut est à 2,08 j par mois (25 j par an, report maximal 5, début en juin) et non 2,5 comme le spec : le prototype compte 25/12, que deux décimales ne représentent pas. Sur une année pleine l'acquis reste 25 j, mais un mois intermédiaire peut différer d'une demi-journée de l'arrondi du prototype (3 mois : 6,24 au lieu de 6,25, donc 6 j au lieu de 6,5). RTT : 10 j par an, début en janvier, aucun report. Pour retrouver 25/12 exactement il faudrait une échelle plus fine ou une fraction.
- Les motifs et les comptes par défaut sont créés ensemble, sous le même verrou de l'organisation, à la première lecture de l'un ou de l'autre ; les motifs « Congés payés » et « RTT » sont reliés à leur compte par leur nom. La migration crée les deux comptes des organisations qui ont déjà des motifs, relie ces deux motifs et ajoute la clé étrangère `absence_types.account_id`. Les `account_id` libres de B3 sont remis à null (aucun compte n'existait). Le retour arrière supprime la clé et remet les `account_id` à null.
- Pas d'endpoint de suppression d'un compte : archiver ou restaurer. Un compte archivé reste lisible, reste sur les motifs et les mouvements passés, mais refuse un nouveau mouvement et un nouveau motif (422 sur `account_id`) ; un motif qui le porte déjà peut être modifié sans changer de compte.
- Lectures : les comptes se lisent comme les motifs (tout membre). Les mouvements de l'organisation exigent `MANAGE_PLANNING` ou `MANAGE_REFERENCE` (soldes personnels) ; sans l'un des deux, 403, et l'écran s'ouvre sans mouvement (soldes d'acquisition seuls, sans bandeau d'erreur). Un membre ordinaire ne voit donc pas son propre solde corrigé des mouvements : il n'existe pas de lecture « mes soldes », à décider avec le spec.
- Écritures : comptes `MANAGE_REFERENCE`, mouvements `MANAGE_PLANNING`. L'auteur est le membre de l'appelant. Un report doit être positif et au plus le maximum du compte (règle du prototype, reprise côté API) ; le motif est obligatoire pour tout mouvement, le front envoie « Report » quand un report n'en a pas.
- Aucun calcul de solde en Rust (ADR 0014) : le front calcule à partir des comptes, des mouvements et des absences. Le total de reports par membre et par période reste contrôlé côté interface seulement.
- Les identifiants de compte sont de vrais UUID : le lien en mémoire `typeAccounts` et `DEFAULT_ACCOUNT_BY_TYPE_NAME` de B3 sont supprimés, `account_id` des motifs vient de l'API. L'ancien `DEFAULT_ACCOUNT_ID` (`cp`) disparaît : sans `?compte=`, l'écran Droits et la barre latérale prennent le premier compte non archivé. Les seeds `ACCOUNTS` et `ADJUSTMENTS` ne servent plus qu'aux tests de calcul ; `addAdjustment` et `updateAccount` sont passés dans `lib/rh/balance-fixtures.ts`, réservé aux tests.
- Archiver et restaurer un compte : bouton dans la fenêtre de paramétrage (`MANAGE_REFERENCE`) ; l'onglet d'un compte archivé est suffixé « (archivé) », et la barre latérale et la carte Soldes d'un membre ne le montrent plus. Il n'y a pas de création de compte dans l'interface (le prototype n'en a pas).
- Le paramétrage refuse désormais une acquisition nulle (positive stricte, comme l'API).
- Le client OpenAPI a été régénéré ; la limite de lecture des mouvements n'est pas paginée.
- `tsc --noEmit` : 38 erreurs avant et après.
