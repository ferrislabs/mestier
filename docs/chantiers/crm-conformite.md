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
| B1 | Réglages d'organisation : IBAN, BIC, pénalités, indemnité de recouvrement, pied de facture, padding, validité des devis, délai de paiement, TVA par défaut, compteur d'avoirs | – | à faire |
| B2 | Client : type, site web, TVA, responsable, source, notes, délai de paiement, contact et adresse de facturation, dates de conversion et d'archivage, SIRET. Contact : mobile. Adresse : type, pays | – | à faire |
| B3 | Opportunités (`deal`) + migration depuis `pipeline_stage` | B2 | à faire |
| B4 | Devis : validité, acompte, motif de refus, signataire, responsable, opportunité, notes, remise et fréquence par ligne, révisions, « remplacé » | B3 | à faire |
| B5 | Factures : titre, délai de paiement, mentions légales, note interne, envoi et relance, création depuis un devis, suppression de brouillon, date d'émission, mode de règlement typé, remboursement | B1, B4 | à faire |
| B6 | Dépenses et catégories, journal, fournisseurs enrichis, catalogue enrichi | B1 | à faire |
| B7 | Indicateurs : CA 12 mois, encours en retard, dernière activité | B3, B5, B6 | à faire |

## Mocks front à remplacer
- `pages/deals/feature/mocks/{deals-store,deal-quotes}.ts` : B3 (amorcé depuis `pipeline_stage`).
- `pages/quotes/feature/mocks/quote-extras.ts` : B4 (responsable, validité, acompte, notes, remise et fréquence par ligne, signataire, motif de refus, révisions, lien opportunité).
- `pages/invoices/feature/mocks/invoice-extras.ts` et `invoice-activity.ts` : B5.
- `pages/suppliers/feature/mocks/supplier-extras.ts`, `pages/catalog/feature/mocks/{catalog-extras,catalog-usage}.ts` : B6.
- `pages/customers/feature/mocks/account-extras.ts` : B2.
- `pages/crm-settings/feature/mocks/settings.ts` : B1.
- `pages/invoices/feature/mocks/invoice-activity.ts` (envoi, relance, note interne) : B5.
- `pages/treasury/feature/mocks/ledger.ts` : B6.
- `pages/catalog/feature/mocks/catalog-extras.ts` : B6.
- `pages/stripe/feature/mocks/stripe-events.ts` et réglages Stripe : restent.

## Reports à traiter dans les lots backend
- B4/B5 : l'API n'expose pas l'article du catalogue sur les lignes de devis et de factures, d'où « Jamais utilisé » partout dans le Catalogue. Ajouter la liaison.
- B5 : `quote_id` sur `CreateInvoiceRequest` et déduction des acomptes de la facture complète, côté serveur ; date d'émission choisie ; solde par facture dans la liste (colonne « Reste dû », filtre « À rembourser »).
- B7 : ville et contact principal dans la liste des clients (aujourd'hui 2 requêtes par client, jusqu'à 100 par page) ; « en retard » exact (aujourd'hui TTC des factures échues plafonné par l'encours, calculé côté front, contraire à la règle du dépôt).
- B4 : le devis d'origine passe « Remplacé » dès la création de la révision (mock) ; le prototype le fait à l'envoi.

## À vérifier au lot concerné
- Sémantique de `POST /invoices/{id}/cancel` et d'un montant négatif sur un paiement (B5, remboursement).
- `registry` (RCS/RM) couvert par `registration_number` ? (B1).
- TVA des recettes codée à 20 % en dur dans `pages/treasury/feature/ledger-feature.tsx:140` (B6).
- Lien entre `cost` du catalogue et `cost_basis` de `handlers-reference` (B6).

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
