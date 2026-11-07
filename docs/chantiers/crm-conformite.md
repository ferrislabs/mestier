# CRM : conformité au prototype

Objectif : le CRM du vrai Mestier (front puis back) est identique, en fonctions et en UI, au module CRM du prototype `mestier-rework` (`src/modules/crm`). Le prototype fait foi. Spec : `docs/specs/crm-conformity.md`. ADR 0011 (opportunités), 0012 (dépenses), 0013 (SIRET).

## Décisions (2026-11-07)
- Front d'abord, avec des mocks `localStorage` pour ce que l'API ne porte pas ; les lots backend remplacent les mocks un par un.
- Stripe reste en mock : aucun lot backend Stripe.
- Opportunité (`Deal`) : entité neuve qui remplace `customers.pipeline_stage`. Une migration crée une opportunité par client ayant une étape en cours. ADR à écrire.
- Les fonctions du réel absentes du prototype sont gardées et restylées : boîte de factures fournisseurs, fiche technique produit, passage de relais devis → projet, stepper de facturation, annulation de devis, suppression de paiement, PDF, pagination serveur, retry réseau, permissions, photo de lieu.
- URLs du prototype (`/crm/clients`, `/devis`, `/factures`, `/avoirs`, `/tresorerie`, `/catalogue`, `/fournisseurs`, `/reglages`), anciennes URLs redirigées.
- Numéro d'identité : SIRET (14 chiffres), pas SIREN. Impact backend en B2 : le Factur-X identifie les parties avec le schéma `0002` (SIREN, 9 chiffres) dans `infrastructure/invoice/facturx/cii.rs`. Le SIREN envoyé au Factur-X est dérivé des 9 premiers chiffres du SIRET. Les valeurs de 9 chiffres déjà stockées restent acceptées.
- « Données de démonstration » du prototype : non migré.
- Avoirs : on garde le modèle réel (facture de type avoir) avec un compteur AV distinct.
- Devis : « Annulé » est gardé, « Remplacé » s'ajoute pour les révisions.

## Lots front (branche `crm-front`)
| Étape | Contenu | Dossiers | État |
|---|---|---|---|
| F1 | Shell : URLs, navigation, Trésorerie en 3 vues, Fournisseurs dans le CRM | `modules/registry.ts`, routes, `pages/crm-nav` | intégré, non commité |
| F2 | Clients et fiche | `pages/customers` | intégré, non commité |
| F3 | Pipeline : opportunités, motif de perte | `pages/deals` | intégré, non commité |
| F4 | Devis | `pages/quotes` | intégré, non commité |
| F5 | Factures et Avoirs | `pages/invoices`, `pages/credit-notes` | intégré, non commité |
| F6 | Trésorerie, Catalogue, Fournisseurs, Réglages, Stripe (mock) | `pages/treasury`, `catalog`, `suppliers`, `crm-settings`, `stripe` | intégré, non commité |

F1 d'abord, puis F2 à F6 en parallèle (un sous-agent par partition de dossiers, jamais deux écrivains sur un fichier). Fin de lot : `pnpm check`, `pnpm test`, `pnpm build`, comparaison visuelle 3100/3000.

## Lots backend (une branche et une PR chacun, empilées)
Branches : `backend/crm-b1-settings` sur `feature/crm-front`, puis `backend/crm-b2-customer`, `b3-deals`, `b4-quotes`, `b5-invoices`, `b6-expenses`, `b7-list`, chacune sur la précédente. Base de travail : `mestier_crm` dans le conteneur Postgres local, jamais la base de développement. Chaque lot remplace aussi les mocks front de son périmètre.

| Lot | Contenu | Dépend de | État |
|---|---|---|---|
| B1 | Réglages d'organisation : IBAN, BIC, pénalités, indemnité de recouvrement, pied de facture, padding, validité des devis, délai de paiement, TVA par défaut, compteur d'avoirs | – | PR ouverte |
| B2 | Client : type, site web, TVA, responsable, source, notes, délai de paiement, contact et adresse de facturation, dates de conversion et d'archivage, SIRET. Contact : mobile. Adresse : type, pays | – | PR ouverte |
| B3 | Opportunités (`deal`) + migration depuis `pipeline_stage` | B2 | PR ouverte |
| B4 | Devis : validité, acompte, motif de refus, signataire, responsable, opportunité, notes, remise et fréquence par ligne, révisions, « remplacé » | B3 | PR ouverte |
| B5 | Factures : titre, délai de paiement, mentions légales, note interne, envoi et relance, création depuis un devis, suppression de brouillon, date d'émission, mode de règlement typé, remboursement | B1, B4 | fait, à relire |
| B6 | Dépenses et catégories, journal, fournisseurs enrichis, catalogue enrichi | B1 | fait, à relire |
| B7 | Indicateurs : CA 12 mois, encours en retard, dernière activité | B3, B5, B6 | à faire |

## Mocks front à remplacer
- `pages/deals/feature/mocks/{deals-store,deal-quotes}.ts` : B3 (amorcé depuis `pipeline_stage`).
- `pages/quotes/feature/mocks/quote-extras.ts` : B4 (responsable, validité, acompte, notes, remise et fréquence par ligne, signataire, motif de refus, révisions, lien opportunité).
- `pages/suppliers/feature/mocks/supplier-extras.ts`, `pages/catalog/feature/mocks/{catalog-extras,catalog-usage}.ts` : B6, supprimés.
- `pages/customers/feature/mocks/account-extras.ts` : B2.
- `pages/crm-settings/feature/mocks/settings.ts` : B1.
- `pages/treasury/feature/mocks/ledger.ts` : B6, supprimé.
- `pages/catalog/feature/mocks/catalog-extras.ts` : B6.
- `pages/stripe/feature/mocks/stripe-events.ts` et réglages Stripe : restent.

## Reports à traiter dans les lots backend
- B4/B5 : l'API n'expose pas l'article du catalogue sur les lignes de devis et de factures, d'où « Jamais utilisé » partout dans le Catalogue. Ajouter la liaison.
- B5 : `quote_id` sur `CreateInvoiceRequest` et déduction des acomptes de la facture complète, côté serveur ; date d'émission choisie ; solde par facture dans la liste (colonne « Reste dû », filtre « À rembourser »).
- B7 : ville et contact principal dans la liste des clients (aujourd'hui 2 requêtes par client, jusqu'à 100 par page) ; « en retard » exact (aujourd'hui TTC des factures échues plafonné par l'encours, calculé côté front, contraire à la règle du dépôt).
- B4 : le devis d'origine passe « Remplacé » dès la création de la révision (mock) ; le prototype le fait à l'envoi.

## Reports issus de B1
- B2 et B6 : migrer les pages customers, catalog et stripe de `useCrmSettings()` (shim dans `crm-settings/feature/mocks/settings.ts`) vers `#/hooks/use-billing-settings`, puis réduire le shim à Stripe seul.
- Le champ « Immatriculation » (RCS) a été retiré du formulaire Entreprise : aucune colonne backend. À décider : une colonne dans un lot ultérieur.
- Le formulaire Entreprise écrase une organisation non assujettie à la TVA en « assujettie » (il exige un numéro de TVA). Comportement du prototype, à arbitrer.
- Antérieur à B1 : `quote_number_counters.next_number` et `invoice_number_counters.next_number` contiennent le dernier numéro attribué ; le seed de la migration quote (`MAX+1`) saute un numéro.
- `cargo test -p mestier-core -- --ignored` échoue sur les harnais cucumber (`unexpected argument '--ignored'`), comme avant B1 : utiliser `--lib -- --ignored --test-threads=1`.

## Reports issus de B2
- `people.ts` (responsables fictifs) reste utilisé par `pages/deals`, `pages/treasury` et la création d'opportunité de la fiche client : à remplacer par les vrais membres en B3 et B6.
- L'identifiant client Stripe reste un mock (`pages/customers/feature/mocks/stripe-customer-ids.ts`).
- Les connecteurs d'automatisation n'exposent pas les nouveaux champs client (leurs échantillons de catalogue changeraient). Leur `update_customer` conserve le profil existant.
- Une valeur `registration_number` déjà stockée et invalide (ni 9 ni 14 chiffres) se lit sans erreur, mais la modification de la fiche est refusée tant qu'elle n'est pas corrigée.
- Un id client d'une autre organisation répond 403 (barrière d'appartenance existante), pas 404 comme le spec le fixe. Spec amendé : le comportement existant est gardé.
- Les deux requêtes par client de la liste (`use-customer-directory`) restent : B7.

## Reports issus de B3
- `people.ts` n'est plus utilisé que par `pages/treasury` : à remplacer par les vrais membres en B6.
- Le pipeline charge au plus 100 opportunités et 100 devis, sans pagination.
- Le connecteur d'automatisation `update_customer` perd son champ `pipeline_stage` ; les workflows existants gardent la clé dans leur config sans effet. Non vérifié : la validation du graphe rejette-t-elle une clé inconnue ?
- Le `down` de B3 supprime toute la table `deals`, y compris les opportunités créées après la migration.
- `DealQuoteStatus` du front gagne `cancelled` ; B4 y ajoute `superseded`.
- `tsc --noEmit` passe de 43 à 48 erreurs, toutes dans des fixtures de tests (`quote-new-ui.test.tsx`, `quote-edit-ui.test.tsx`) auxquelles manquent les champs client de B2. À corriger quand B4 touche ces tests.
- Le bloc de sortie de B3 a été interrompu par un disque plein (`target/debug/incremental`, 21 Go, supprimé) ; relancé en entier par l'orchestrateur.

## Reports issus de B4
- TVA désormais calculée une fois par taux sur la base agrégée (moitié éloignée de zéro) : 2 × 100,33 € à 20 % donne 20066 / 4013 au lieu de 4014. Les totaux déjà stockés ne changent pas, seuls les brouillons ré-enregistrés.
- Le PDF ne lit pas `issuer_snapshot` / `recipient_snapshot` : il rend l'identité en direct. À décider.
- Le compteur d'usage du catalogue (4-13) est reporté en B6.
- Contact et adresse de facturation du devis : stockés, sans champ d'édition dans l'UI.
- `tsc --noEmit` : 43 erreurs, hors périmètre, antérieures à B2.
- Vérification de B4 allégée (clippy, tests Rust, migration, Biome, vitest) ; les tests de base et l'e2e ont été lancés par l'agent (810 et 19/19).

## Reports issus de B5
- Les avoirs se créent par ligne source et quantité (ou `full`), avec un motif obligatoire ; `allow_exceeding_total`, `allow_exceeding_invoice_total` et `due_at` ont disparu des requêtes, le délai de paiement les remplace (le formulaire du projet convertit sa date d'échéance en jours).
- Les actions d'automatisation `mestier.invoice.*` suivent : `issue_credit_note` prend `reason` et des `lines` optionnelles, `record_payment` prend un `method` typé, `due_at` est retiré. Un workflow existant qui passe encore l'un de ces champs n'est pas migré. Non vérifié : la validation du graphe rejette-t-elle une clé inconnue ?
- Les moyens de règlement déjà saisis en texte libre sont migrés vers `TRANSFER`, le texte saisi est ajouté à la note du paiement.
- `status` reste la lecture historique (une facture entièrement créditée et non payée reste `ISSUED`) ; l'état affiché vient de `display_state`.
- Le PDF de facture rend désormais la remise de ligne et les mentions légales ; ses caractères non ASCII sortent en `?` comme tout le reste du texte du PDF.
- `lastIssuedDate` (borne de la date d'émission du dialogue) est toujours calculée côté front sur la page chargée ; le serveur refuse de toute façon (`E_INVOICE_DATE_RANGE`).
- Restent calculés côté front, hors périmètre B5 : l'aperçu de l'avoir (`credit-note-request.ts`) et le total d'une ligne de brouillon (`invoiceLineTotalCents`, désormais avec remise et arrondi moitié éloignée de zéro). L'aperçu du montant d'un acompte a été retiré.
- Les e-mails d'envoi et de relance ne partent pas : le lot enregistre l'événement et `sent_at`.
- Le montant d'un acompte de projet (`issue_deposit`) et le solde (`issue_final_invoice`) restent des lignes forfaitaires sans ventilation de TVA, comme avant.

## Reports issus de B6
- Les recettes sans facture vivent dans `unallocated_payments`, pas dans `invoice_payments` : l'affectation enregistre un vrai paiement de facture puis supprime la ligne. Un paiement Stripe n'existe pas dans le journal réel (l'origine « Stripe » n'est plus affichée que par le mock Stripe).
- Les neuf catégories par défaut sont créées à la première lecture d'une organisation qui n'en a aucune. Deux premières lectures simultanées peuvent se heurter à l'index unique (409).
- Aucun écran de gestion des catégories : l'API les crée, renomme et supprime, le front ne les expose pas.
- Le journal renvoie toutes les écritures sans pagination ; le filtre par type, catégorie, statut et texte reste côté front.
- Calculs restants côté front, hors périmètre du spec : le préremplissage de la TVA d'une dépense à partir d'un taux et l'aperçu du total du formulaire, `toCollect` (somme de l'encours par client) et les cumuls par fournisseur de `supplier-model.ts` (dépensé sur 12 mois, à payer).
- Le justificatif d'une dépense est téléversé par `/files` et seule sa clé est conservée ; le détail affiche le nom du fichier, sans lien de téléchargement.
- `useCrmSettings` a disparu, le shim `crm-settings/feature/mocks/settings.ts` ne garde que Stripe. `people.ts` (mock) est supprimé, la Trésorerie lit les vrais membres.
- Les actions d'automatisation `mestier.achats.*` et `mestier.referentiel.*` n'exposent pas les nouveaux champs ; `update_supplier` conserve le profil existant et `update_product` / `update_service_rate` gardent les champs de catalogue.
- Un fournisseur dont le numéro d'immatriculation déjà stocké est invalide se lit sans erreur, sa modification est refusée tant qu'il n'est pas corrigé.

## À vérifier au lot concerné
- `POST /invoices/{id}/cancel` : conservé tel quel (statut `CANCELLED`, sans avoir) ; un montant négatif sur un paiement est un remboursement (B5).
- `registry` (RCS/RM) couvert par `registration_number` ? (B1).
- TVA des recettes codée à 20 % en dur : supprimée en B6, la TVA du journal vient de `GET /ledger`.
- Lien entre `cost` du catalogue et `cost_basis` de `handlers-reference` : aucun, `cost_basis` est le coût horaire d'un salarié (B6).

## Partition F2 à F6 (écrivains uniques)
- F2 : `pages/customers` hors pipeline. Propriétaire de `types.ts` et `feature/mocks/account-extras.ts`.
- F3 : nouveau dossier `pages/deals` (opportunités, store mock) et `routes/…crm.pipeline.tsx`. Les fichiers pipeline de `pages/customers` restent, à supprimer à l'intégration.
- F4 : `pages/quotes`.
- F5 : `pages/invoices`, `pages/credit-notes`.
- F6a : `pages/treasury`, `pages/stripe`, `pages/crm-settings`.
- F6b : `pages/catalog`, `pages/suppliers`.
- Orchestrateur : `modules/registry.ts`, `routeTree.gen.ts`, `pages/crm-nav`, ce document.

## Journal
- 2026-11-07, F1 : URLs françaises appliquées à 68 fichiers, routes renommées, redirections des anciennes URLs par la route `crm/$` (`modules/legacy-crm-path.ts`) et `purchase/suppliers`, sidebar CRM groupée (`pages/crm-nav`), Fournisseurs retiré du module Achats. `/crm/devis/$id/handover` devient `/crm/devis/$id/passation` (hors prototype, choix de l'orchestrateur). Vérifié : Biome propre, 4824 tests verts, `pnpm build` passe, 43 erreurs `tsc` préexistantes (fixtures de tests). Comparaison visuelle non faite (fin de lot).
- 2026-11-07 : audit en 4 parties (Clients/Pipeline/Réglages, Devis/Factures/Avoirs, Trésorerie/Catalogue/Fournisseurs/Stripe, backend). Découpage validé.
- 2026-11-07, F2 à F6 : six partitions intégrées. Ancien pipeline client supprimé (10 fichiers de `pages/customers`). Opportunités branchées dans la fiche client, `?opportunite=` lu à la création d'un devis, filtre « Archivés » dans l'URL des fournisseurs. Vérifié : Biome propre, 5075 tests verts (405 fichiers), `pnpm build` passe, 43 erreurs `tsc` préexistantes (fixtures de tests), aucune dans les pages CRM. Comparaison visuelle 3100/3000 non faite : extension Chrome non connectée.
