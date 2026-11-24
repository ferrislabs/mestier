# DAF : Demandes d'Agrément de Fourniture

État : **W0 à W4 intégrés le 2026-11-18 sur `feat/daf`, PR à ouvrir, recette navigateur à faire**.

## Goal

Un utilisateur prépare les DAF d'un projet dans Mestier. Une DAF est la fiche d'une
fourniture soumise à l'agrément de la maîtrise d'œuvre. L'utilisateur suit le statut
de chaque DAF dans un sommaire et télécharge soit une DAF en PDF, soit le dossier
complet (le sommaire puis toutes les DAF).

## Context

- Besoin : Treyve Paysages. Exemples fournis le 2026-11-18 : une DAF (« D.A.F n°9,
  Jeux - Ensemble table bois ») et un sommaire (« Récapitulatif des D.A.F »).
- Mise en page : on reprend le style DOE du projet (couleurs, polices, logo). Les
  couleurs de l'exemple ne sont pas copiées.
- Proche du DOE ([doe.md](doe.md)) : copie figée du fournisseur, de la photothèque et
  des fiches techniques ; même rendu Typst (`libs/doc-render`), même droit
  `planning.manage`, même famille d'outils MCP.
- Décisions produit (2026-11-18) :
  1. Le sommaire est généré à partir des DAF du projet. Il n'y a rien à saisir.
  2. Une DAF a 4 statuts : en attente, acceptée, refus avec observations, refus ferme.
     Le sommaire colore chaque ligne selon son statut. Le cadre VISAS du PDF est
     toujours imprimé vide.
  3. Un dossier DAF par projet porte l'en-tête commun : titre, émetteur, maîtrise
     d'ouvrage, maîtrise d'œuvre. Une DAF peut remplacer la MOA et la MOE.
  4. Une DAF a une seule image. On la dépose directement ou on la choisit dans la
     photothèque. L'IA peut aussi la choisir dans la photothèque.
  5. Les pièces jointes (PDF ou images) sont insérées en pages après la fiche, et
     leurs noms sont listés dans la case « Pièces jointes ».
  6. On peut créer une DAF depuis un produit du catalogue. Le fournisseur, les
     caractéristiques, la première photo et les fiches techniques sont copiés.
  7. Deux exports : un PDF par DAF et un PDF du dossier complet. Pas d'export Word.
  8. Par MCP, l'IA peut lister, lire, créer et modifier les DAF, modifier le dossier,
     poser une photo de la photothèque et exporter. Elle ne peut pas supprimer.

## Glossary

| Terme | Définition | Synonyme rejeté |
|---|---|---|
| DAF (`Daf`) | Demande d'agrément d'une fourniture, adressée à la MOE avant la pose | fiche agrément, « demande » seul |
| Dossier DAF (`DafDossier`) | L'en-tête commun aux DAF d'un projet (titre, émetteur, MOA, MOE) | registre |
| Sommaire | La page « Récapitulatif des D.A.F », générée à partir des DAF | index, tableau de bord |
| Statut (`DafStatus`) | La réponse de la MOE à une DAF | visa (le visa est le cadre signé sur papier) |
| Observation de retour (`review_note`) | La remarque de la MOE, imprimée dans la colonne « Observation » du sommaire | — |
| Observations (`observations`) | Le bloc libre « OBSERVATIONS » de la fiche (destination, référence, garanties) | — |

## Invariants

1. Deux DAF d'un même projet n'ont jamais le même numéro. [contrainte `UNIQUE (project_id, number)` → 409, test]
2. Un numéro est compris entre 1 et 999. [type : `DafNumber::try_from`]
3. La désignation n'est jamais vide et fait au plus 200 caractères. [type : `DafDesignation::try_from`]
4. Modifier un produit, un fournisseur ou une photo de la photothèque ne change aucune DAF. [type : `Daf` ne garde que des copies, `source_product_id` est informatif ; la photo placée est une copie de fichier, comme pour le DOE]
5. Toute lecture et toute écriture est limitée à l'`organization_id`. Une DAF d'une autre organisation est introuvable (404). [test]
6. Le PDF reste borné : au plus 200 DAF par projet (409 à la création), 20 pièces jointes par DAF (409 à l'enregistrement). Les limites de rendu existantes (600 pages insérées, 256 Mo par export) s'appliquent au dossier. [test]
7. Une DAF sans MOA ni MOE propre imprime celles du dossier. [type : `Option` sur la DAF, résolu dans `DafDocument::build`, test unitaire]
8. Le rendu ne lit ni le disque ni le réseau. [type : le `World` Typst ne reçoit qu'un `FileBundle`]

## Out of scope

- Signature et remplissage des visas, envoi par e-mail à la MOE, historique des statuts.
- Aperçu SVG en direct dans l'éditeur. Le PDF s'ouvre dans un nouvel onglet.
- Export Word, archivage des PDF générés.
- Lien entre une DAF et une fourniture du DOE.
- Événements de domaine : le DOE n'en émet pas, la DAF non plus en v1.

## Design

**Domain** (`libs/core/src/domain/daf/`)
- `mod.rs` : `DafId`, `DafNumber`, `DafDesignation`, `DafStatus`, `DafAttachment`, `Daf`, `DafDossier`, `DafSummary`.
- `commands.rs` : `CreateDaf`, `UpdateDaf`, `UpdateDafDossier`.
- `ports.rs` : `DafRepository` (dossiers et DAF), `DafRenderer`.
- `render.rs` : `DafDocument`, `DafSheet`, `DafSummaryRow`, et `DafDocument::build`, qui est pure.
- `service.rs` : `DafService` : dossier par défaut (pré-rempli), création (numéro suivant, brouillon depuis un produit), mise à jour, placement de la photo, document de rendu.

**Persistance** : deux tables. `daf_dossiers` (`project_id` PK, `org_id`, titre, émetteur,
MOA, MOE) et `dafs` (une ligne par DAF, colonnes typées). `supplier` et `attachments` sont
en JSONB, parce que ce sont des copies jamais interrogées. `status` est une colonne `TEXT`
avec `CHECK`, pour que le sommaire filtre et trie en SQL. Le dossier est créé avec la
première DAF, ou par un `PUT` du dossier. Un `GET` sans dossier renvoie le dossier
pré-rempli sans l'enregistrer.

**Pré-remplissage du dossier** : titre = nom du projet ; émetteur = nom de l'organisation ;
MOA = nom du client du projet ; MOE = MOE du DOE du projet s'il existe, sinon vide.

**Style** : le style DOE du projet (`doe_project_styles`). S'il n'existe pas, celui du
modèle du DOE du projet, puis celui de l'organisation, puis le style par défaut. C'est la
même chaîne que `DoeService::document`, réutilisée sans copie.

**Rendu** : `DafRenderer` est implémenté par `TypstDafRenderer` dans `libs/doc-render`,
avec le gabarit `daf.typ`. Il reprend les erreurs `DoeRenderError` et le `FileBundle` du
DOE (une dette assumée : ces deux types vivent dans `domain/doe/render.rs`). Chaque fiche
fait une page A4 portrait, structurée comme l'exemple : bandeau (logo, titre,
« > Demande d'Agrément de Fourniture », numéro, émetteur, date de diffusion),
IDENTIFICATION FOURNITURE (désignation, correspondance, fournisseurs, assemblage et pose,
caractéristiques techniques à gauche, image à droite, pièces jointes), OBSERVATIONS,
DIFFUSION (MOA, MOE), puis VISAS vides (3 colonnes, 2 lignes). Les pièces jointes
suivent en pages. Le sommaire reprend le même bandeau avec « SOMMAIRE », une légende des
4 statuts et le tableau N° DAF / N° prix / Désignation / Envoyé / Diffusion / Retour /
Observation.

**HTTP** (préfixe `/api/v1/organizations/{organization_id}`, crate `handlers-planning`, module `daf`)

| Méthode | Chemin | Droit | Réponse |
|---|---|---|---|
| GET | `/projects/{project_id}/dafs` | membre | `{ dossier, dafs: [DafSummary] }` triées par numéro |
| PUT | `/projects/{project_id}/daf-dossier` | planning.manage | `DafDossier` |
| POST | `/projects/{project_id}/dafs` `{ designation, product_id? }` | planning.manage | `Daf` (201) |
| GET | `/dafs/{daf_id}` | membre | `Daf` |
| PUT | `/dafs/{daf_id}` (contenu complet) | planning.manage | `Daf` |
| DELETE | `/dafs/{daf_id}` | planning.manage | 204 |
| POST | `/dafs/{daf_id}/photo/place` `{ photo_id }` | planning.manage | `Daf` |
| GET | `/dafs/{daf_id}/pdf` | membre | PDF |
| GET | `/projects/{project_id}/dafs/pdf` | membre | PDF du dossier |

Liste blanche des tokens délégués (`libs/handlers/src/delegated.rs`) : toutes ces routes
sauf `DELETE`.

**Errors** : `CoreError` existant. Entrée invalide, numéro déjà pris ou limite atteinte :
`Conflict`. `MissingFile` au rendu : `Conflict` qui nomme la DAF.

**MCP** (`libs/mcp-tools`, `apps/mcp-server/src/daf.rs`, domaine `doe`, scope
`mestier.doe.write` existant, donc pas de changement FerrisKey ni Cerbos)

| Outil | Accès | Route |
|---|---|---|
| `list_dafs` `{ project_id }` | read | GET liste |
| `get_daf` `{ daf_id }` | read | GET DAF |
| `create_daf` `{ project_id, designation, product_id? }` | write | POST |
| `update_daf` `{ daf_id, champs… }` (seuls les champs donnés changent, `""` efface) | write | GET puis PUT |
| `update_daf_dossier` `{ project_id, title?, issuer?, project_owner?, project_manager? }` | write | GET puis PUT |
| `place_daf_photo` `{ daf_id, photo_id }` | write | POST place |
| `export_daf_pdf` `{ daf_id }` ou `{ project_id }` (dossier) | read | GET pdf, renvoyé comme `export_doe_pdf` |

**Front** (`apps/webapp/src/pages/daf/{feature,ui}`)
- Projet : un onglet « DAF » à côté de « DOE » (route `/projet/$projectId/daf`). Il affiche l'en-tête du dossier (modifiable), le sommaire sous forme de tableau avec un badge de statut, « Nouvelle DAF » (désignation, ou choix d'un produit du catalogue), « PDF du dossier ».
- Éditeur `/projet/$projectId/daf/$dafId` : formulaire TanStack Form avec tous les champs, le statut et les dates de suivi. Image : `FileDropZone` et le sélecteur de photothèque du DOE, réutilisés. Pièces jointes : dépôt de fichiers. Boutons « PDF » et « Supprimer ».

## Frozen contracts

Écrits en W0 dans les fichiers « orchestrateur seul » ; ce sont eux qui font foi. Résumé :

```rust
pub struct DafId(pub Uuid);
pub struct DafNumber(u16);               // TryFrom<u16> : 1..=999
pub struct DafDesignation(String);       // TryFrom<String> : trim, 1..=200
#[serde(rename_all = "snake_case")]
pub enum DafStatus { Pending, Accepted, RejectedWithComments, Rejected }
pub struct DafAttachment { key: String, name: String }

pub struct DafDossier {
    organization_id, project_id,
    title: String, issuer: String,
    project_owner: Option<String>, project_manager: Option<String>,
    updated_at: Option<DateTime<Utc>>,   // None : pré-rempli, pas encore enregistré
}

pub struct Daf {
    id: DafId, organization_id, project_id, number: DafNumber,
    designation: DafDesignation,
    price_ref: Option<String>,           // « Correspondance » : N° PRIX
    source_product_id: Option<ProductId>,
    supplier: Option<SupplierSnapshot>,  // réutilisé depuis domain/doe
    installer: Option<String>,           // « Assemblage et pose », multiligne
    specs: Option<String>,               // « Caractéristiques techniques »
    observations: Option<String>,
    photo_key: Option<String>,
    attachments: Vec<DafAttachment>,
    project_owner: Option<String>,       // None : celle du dossier
    project_manager: Option<String>,
    status: DafStatus,
    sent_on: Option<NaiveDate>,          // « Envoyé »
    issued_on: NaiveDate,                // « Diffusion », défaut : aujourd'hui
    returned_on: Option<NaiveDate>,      // « Retour »
    review_note: Option<String>,         // colonne « Observation » du sommaire
    created_at, updated_at,
}

pub struct DafSummary { id, number, designation, price_ref, status, sent_on, issued_on, returned_on, review_note }

pub struct DafDocument { header: DafHeader, summary: Option<Vec<DafSummaryRow>>, sheets: Vec<DafSheet>, style: DocumentStyle }
impl DafDocument {
    pub fn single(dossier: &DafDossier, daf: &Daf, style: &DoeStyle) -> Self;
    pub fn dossier(dossier: &DafDossier, dafs: &[Daf], style: &DoeStyle) -> Self;
    pub fn file_keys(&self) -> Vec<&str>;
}
pub trait DafRenderer: Send + Sync {
    fn render(&self, document: &DafDocument, files: &FileBundle) -> Result<Vec<u8>, DoeRenderError>;
}
```

## Acceptance (Gherkin)

```gherkin
@spec-daf-1 Scenario: dossier pré-rempli
  Given un projet "Parc Montgroux" pour le client "Commune de Cébazat", sans DAF
  When je lis les DAF du projet
  Then le dossier a pour titre "Parc Montgroux", MOA "Commune de Cébazat", émetteur le nom de l'organisation
  And il n'est pas enregistré

@spec-daf-2 Scenario: numérotation
  Given un projet avec les DAF n°1 et n°2
  When je crée la DAF "Ganivelle"
  Then elle porte le n°3, statut "en attente", diffusion aujourd'hui
  And renuméroter la n°3 en n°1 répond 409

@spec-daf-3 Scenario: création depuis le catalogue, copie figée
  Given un produit "Ensemble table bois" du fournisseur "KOMPAN" avec une fiche technique PDF et une photo
  When je crée une DAF depuis ce produit
  And je change l'adresse de "KOMPAN"
  Then la DAF garde l'ancienne adresse, la photo du produit et la fiche en pièce jointe

@spec-daf-4 Scenario: photo depuis la photothèque
  Given une photo dans la photothèque
  When je la pose sur une DAF puis je supprime la photo de la photothèque
  Then la DAF a toujours une image lisible

@spec-daf-5 Scenario: diffusion héritée
  Given un dossier avec MOE "Atelier Casa" et une DAF sans MOE propre
  When je construis son document
  Then la fiche imprime "Atelier Casa" en maîtrise d'œuvre

@spec-daf-6 Scenario: PDF du dossier
  Given 2 DAF, dont une avec une pièce jointe PDF de 2 pages
  When je télécharge le PDF du dossier
  Then il fait 5 pages : sommaire + (1) + (1 + 2)
  And son texte contient "Récapitulatif des D.A.F", "Demande d'Agrément de Fourniture" et "VISAS"

@spec-daf-7 Scenario: isolation
  Given une DAF de l'organisation A
  When un membre de l'organisation B la lit, la modifie ou télécharge son PDF
  Then la réponse est 404

@spec-daf-8 Scenario: droits
  Given un membre sans planning.manage
  When il crée ou modifie une DAF
  Then la réponse est 403
  And il peut lire les DAF et télécharger les PDF

@spec-daf-9 Scenario: MCP
  Given un token délégué avec mestier.doe.write
  When l'IA crée une DAF, la modifie, pose une photo de la photothèque et exporte le dossier
  Then chaque appel réussit
  And DELETE /dafs/{id} avec ce token est refusé

@spec-daf-10 Scenario: éditeur (front)
  Given l'onglet DAF d'un projet
  When je crée "Ganivelle", choisis une photo de la photothèque, passe le statut à "acceptée" et enregistre
  Then le sommaire montre la ligne n°1 "Ganivelle" avec le badge "Acceptée"
```

Tests : 1, 2, 3, 4, 7, 8 en intégration (`#[ignore]`, base réelle) ; 5 et les newtypes en
unitaires ; 6 en test du crate `doc-render` (page count via `lopdf`) ; 9 en test de la
liste blanche (`delegated.rs`) et des schémas d'arguments MCP ; 10 en vitest sur la feature.

## Simulation

Non. Deux créations simultanées peuvent viser le même numéro : la contrainte `UNIQUE`
renvoie 409 à la seconde, qui réessaie. C'est accepté en v1.

## Workstreams

| Id | Mission | Dépend de | Modèle |
|---|---|---|---|
| W0 | Contrats : migration (up/down), types de domaine, ports, commandes, `render.rs` (types), marqueur de registry, squelette `daf` dans handlers-planning (chemins, DTO), spec, glossaire. **Orchestrateur.** | — | orchestrateur |
| W1 | `doc-render` : `daf.typ` + `TypstDafRenderer` + test @spec-daf-6 | W0 | sonnet |
| W2 | Core : `DafService`, `DafDocument::build`, repository Postgres, use cases, handlers, liste blanche, tests 1 à 5, 7, 8 | W0 | sonnet |
| W3 | MCP : 7 outils dans `mcp-tools` et `apps/mcp-server/src/daf.rs`, test @spec-daf-9 côté outils | W0 (DTO figés) | sonnet |
| W4 | Front : onglet, sommaire, éditeur, test @spec-daf-10 | W2 intégré + `pnpm gen:api` | sonnet |

W1, W2 et W3 partent ensemble après W0. W4 part dès que W2 est intégré.

### Fichiers

- **W1** possède `libs/doc-render/src/daf.typ`, `libs/doc-render/src/daf.rs`, et ajoute une ligne `pub use` dans `libs/doc-render/src/lib.rs`.
- **W2** possède `libs/core/src/domain/daf/service.rs`, `libs/core/src/infrastructure/daf/**`, `libs/core/src/application/daf/**`, `libs/handlers-planning/src/daf/**` sauf les `TypedPath` de `mod.rs`, `request.rs` et `response.rs`, `libs/handlers/src/delegated.rs` (lignes DAF).
- **W3** possède `libs/mcp-tools/src/lib.rs` (variantes DAF), `apps/mcp-server/src/daf.rs`, l'enregistrement des outils dans `apps/mcp-server/src/tools.rs`.
- **W4** possède `apps/webapp/src/pages/daf/**`, `apps/webapp/src/hooks/use-daf.ts`, les routes `projet.$projectId_.daf*`, l'onglet dans `pages/projects/feature/project-detail-feature.tsx`.

**Orchestrateur seul** : `migrations/*`, `Cargo.toml`, `libs/core/src/domain/daf/{mod,ports,commands,render,document}.rs`, `libs/core/src/{domain,application,infrastructure}/mod.rs`, `libs/core/src/infrastructure/registry.rs`, `libs/core/src/lib.rs`, `libs/handlers-planning/src/{lib.rs,daf/request.rs,daf/response.rs}`, `apps/api/src/{router,openapi}.rs`.

**Régénérés à l'intégration** : `.sqlx/`, `Cargo.lock`, `apps/webapp/src/api/api.client.ts`, `apps/webapp/src/routeTree.gen.ts`.

## Risks / open questions

| Risque | Porteur | Parade |
|---|---|---|
| Une image haute dans la case de droite déborde de la page | W1 | Image contenue dans sa case (`fit: "contain"`), hauteur fixe |
| Des caractéristiques longues font passer la fiche sur 2 pages | W1 | Corps de texte réduit dans la case ; si ça déborde quand même, la fiche continue sur la page suivante, sans couper le cadre VISAS |
| `DoeRenderError` et `FileBundle` partagés entre DOE et DAF | orchestrateur | Dette assumée ; à déplacer dans un module `document` commun si un troisième rendu arrive |

## Verify (exit)

```bash
cargo fmt --all --check
SQLX_OFFLINE=true cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p mestier-core -- --ignored
cd apps/webapp && pnpm check && pnpm test && pnpm build
```

## Statut

| Lot | État |
|---|---|
| W0 | intégré |
| W1 | intégré |
| W2 | intégré |
| W3 | intégré |
| W4 | intégré |

## Decision log

- 2026-11-18 : outils MCP DAF sous le domaine `doe` et le scope `mestier.doe.write`. Rejeté : un nouveau scope `mestier.daf.write`, car il faudrait toucher FerrisKey, Cerbos et le consentement, alors que la DAF appartient au même dossier de chantier que le DOE.
- 2026-11-18 : DAF en table à colonnes typées, pas en JSONB comme le DOE. Raison : le sommaire trie et filtre par numéro et statut, et l'unicité du numéro doit être garantie par la base.
- 2026-11-18 : MOA et MOE de la DAF en `Option` qui hérite du dossier, pas copiées à la création. Raison : corriger le nom de la MOE dans le dossier doit corriger toutes les DAF non encore personnalisées.

Amended: 2026-11-18 W0 écrit aussi `DafDocument::single/dossier/file_keys` (`document.rs`, pur, testé). Les chemins sont dans `daf/mod.rs`, les DTO dans `daf/request.rs` et `daf/response.rs`. Migration `20261127000001_create_daf`. `CreateDafCommand.designation` est optionnelle quand un produit est donné.
- 2026-11-18 : le DAF ne s'imprime pas avec le glyphe « ▸ » (absent des polices embarquées) : petits triangles dessinés. Bandeaux de section gris, couleur primaire sur le texte. Une fiche qui déborde continue sans bandeau sur la page suivante.
- 2026-11-18 : les PDF d'une DAF et du dossier, le service et les outils MCP sont testés ; @spec-daf-10 est couvert en vitest, pas encore vérifié dans un vrai navigateur.
