# DOE : Dossier des Ouvrages Exécutés

État : **découpage validé le 2026-10-07, W0 intégré, W1 à W3b en cours**. Mis à jour à chaque intégration.

## Goal

Un utilisateur produit le PDF du DOE d'un projet depuis Mestier : il part d'un modèle,
coche ses thèmes, pioche les fournitures dans le catalogue, ajoute photos et plans,
et télécharge un PDF mis en page.

## Context

- Besoin : Erwan (paysagiste, Treyve Paysages), exemple de 28 pages fourni hors dépôt.
- Maquette validée : https://claude.ai/artifact/SD1XDmThsvYcJoigEjSVdr
- Le DOE n'est pas propre au paysagisme : tous les corps d'état du bâtiment en remettent
  un à la réception. Le moteur est générique ; le paysagisme est un **modèle de départ**.
- ADR : [0003, rendu Typst](../adr/0003-typst-document-rendering.md).
- Décisions produit (2026-10-07) :
  1. Une fourniture dans un DOE est une **copie** de la fiche catalogue, pas un lien.
  2. Le fournisseur d'une fiche est un `Supplier`, qui gagne adresse, téléphone, email.
     Le fabricant / la pépinière reste un texte libre.
  3. Les intervenants sont des blocs copiés dans le DOE. La maîtrise d'ouvrage est
     pré-remplie depuis le client du projet, l'entreprise depuis l'organisation.
  4. Un DOE par projet. L'indice est un champ texte. Le PDF est généré à la demande,
     pas archivé.
  5. Droits : `planning.manage` pour le DOE, `reference.manage` pour fiches catalogue,
     fournisseurs, modèles de DOE et réglages de mise en page.
  6. Interface : page DOE accessible depuis le projet (Planification) ; modèles de DOE
     à côté des modèles de projet.

## Glossary

| Terme | Définition | Synonyme rejeté |
|---|---|---|
| DOE (`Doe`) | Dossier remis au client à la réception, décrivant ce qui a été posé | as-built file |
| Section | Un thème numéroté du DOE (Végétaux, Photos…) | thème, chapitre (UI : « thème ») |
| Fourniture (`DoeSupply`) | Copie, dans un DOE, d'une fiche technique | entrée, item |
| Fiche technique | Les champs DOE d'un `Product` : fournisseur, fabricant, caractéristiques, documents | — |
| Modèle de DOE (`DoeModel`) | Liste de sections pré-réglées, propre à l'organisation | gabarit |
| Modèle de départ (`StarterModel`) | Modèle livré dans le code, copié dans l'organisation à la demande | template système |
| Intervenant (`DoeParty`) | MOA, MOE ou entreprise, copié dans le DOE | acteur |

## Invariants

1. Modifier un produit ou un fournisseur ne change aucun DOE existant. [type : `DoeSupply` ne contient aucune référence lue au rendu, `source_product_id` est informatif]
2. Une section ne contient que le contenu de son genre (pas de photo dans une section Fournitures). [type : `DoeSectionContent` porte le contenu dans la variante]
3. Au plus un DOE par projet. [contrainte `UNIQUE (project_id)` + test]
4. Toute lecture et écriture est scopée par `organization_id` ; un DOE, un modèle ou un fichier d'une autre organisation est introuvable. [test]
5. L'indice n'est jamais vide et fait au plus 8 caractères. [type : `Revision::try_from`]
6. Nombre de pages libres entre 1 et 20. [type : `BlankPageCount::try_from`]
7. La couleur d'accent est `#rrggbb`. [type : `HexColor::try_from`]
8. Une section désactivée n'apparaît ni dans le PDF ni dans la numérotation. [test sur `DoeDocument::from_doe`]
9. Le rendu ne lit ni disque ni réseau : il n'utilise que les fichiers fournis. [type : le `World` Typst ne reçoit qu'un `FileBundle`]

## Out of scope

- Pré-remplissage depuis les lignes de devis (`QuoteLine` n'a pas de `product_id`) et depuis les factures fournisseurs.
- Historique des indices, archivage des PDF générés, signature.
- Modèles de départ autres que Paysagiste.
- Migration des PDF devis/factures vers Typst.
- Génération asynchrone (job) : le rendu est synchrone, voir Risks.

## Design

**Domain** (`libs/core/src/domain/`)
- `doe/` : `Doe`, `DoeSection`, `DoeSectionContent`, `DoeSupply`, `DoeFile`, `DoeParty`, newtypes. `DoeDocument::from_doe` (pur, numérote, filtre les sections désactivées). `DoeService` : créer depuis un modèle, remplacer le contenu, brouillon de fourniture depuis un produit, document de rendu.
- `doe_model/` : `DoeModel`, `DoeModelSection`, `StarterModel`, `DoeSettings`. `DoeModelService`.
- `product/` : `Product` gagne la fiche technique.
- `supplier_invoice/` : `Supplier` gagne ses coordonnées ; nouveau port `SupplierRepository`.

**Persistance** : le contenu d'un DOE (intervenants + sections) et les sections d'un modèle sont stockés en **JSONB** sur leur ligne. C'est un document copié, jamais interrogé par l'intérieur ; le remplacer entier est l'opération normale (voir décision 1).

**Ports** (dispatch statique, générique, comme les repositories existants)
- `DoeRepository`, `DoeModelRepository`, `DoeSettingsRepository`, `SupplierRepository` : `#[repository(domain = X, backend = Postgres)]`.
- `DoeRenderer` : synchrone, `Send + Sync`, implémenté par `libs/doc-render` (`TypstDoeRenderer`, unit struct, polices dans un `OnceLock`).

**HTTP** (préfixe `/api/v1`)
| Méthode | Chemin | Crate | Droit |
|---|---|---|---|
| GET | `/projects/{project_id}/doe` | handlers-planning | membre |
| POST | `/projects/{project_id}/doe` `{model_id}` | handlers-planning | planning.manage |
| PUT | `/does/{doe_id}` (contenu complet) | handlers-planning | planning.manage |
| POST | `/does/{doe_id}/supplies/draft` `{product_id}` → `DoeSupply` non persisté | handlers-planning | planning.manage |
| GET | `/does/{doe_id}/pdf` | handlers-planning | membre |
| GET/POST | `/doe-models` | handlers-planning | membre / reference.manage |
| GET/PUT | `/doe-models/{id}` ; POST `/doe-models/{id}/archive`, `/restore` | handlers-planning | membre / reference.manage |
| GET | `/doe-models/starters` ; POST `/doe-models/from-starter` `{starter}` | handlers-planning | membre / reference.manage |
| GET/PUT | `/doe-settings` | handlers-planning | membre / reference.manage |
| GET/POST | `/suppliers` ; PUT `/suppliers/{id}` | handlers-purchase | membre / reference.manage |
| (existant) | `/products…` gagne les champs de fiche technique | handlers-reference | reference.manage |

Le PDF : le handler appelle `usecase.doe_document(...)`, charge chaque clé de `DoeDocument::file_keys()` via `state.file_storage`, puis `TypstDoeRenderer.render` dans `spawn_blocking`.

**Errors** : `CoreError` existant (`NotFound`, `Conflict`, `Forbidden`) ; une entrée invalide est un `Conflict`, comme ailleurs dans le code ; `DoeRenderError` converti en `CoreError::Internal` par le handler, sauf `MissingFile` → `Conflict` (fichier supprimé du stockage).

**Front** (`apps/webapp`)
- Catalogue : formulaire produit gagne la section « Fiche technique » (fournisseur choisi dans la liste, fabricant, caractéristiques, documents).
- Achats : liste et formulaire fournisseurs (nom, coordonnées).
- Planification › Projets › projet : bouton « DOE » → route `/planification/projects/$projectId/doe` ; éditeur conforme à la maquette ; sauvegarde par `PUT` complet.
- Planification › Modèles de DOE : liste, édition des sections, « Créer depuis Paysagiste ».
- Réglages : section « Mise en page des documents » (couleur, logo).

## Acceptance (Gherkin)

```gherkin
@spec-doe-1 Scenario: créer un DOE depuis un modèle
  Given un projet "Avenue du 8 mai" pour le client "Commune de Lapalisse"
  And le modèle "Paysagiste" de l'organisation
  When je crée le DOE du projet avec ce modèle
  Then le DOE a les 7 sections du modèle, dans l'ordre, "Jeux" désactivée
  And l'intervenant MOA s'appelle "Commune de Lapalisse"
  And l'intervenant Entreprise porte le nom de l'organisation

@spec-doe-2 Scenario: un seul DOE par projet
  Given un projet qui a déjà un DOE
  When je crée un DOE pour ce projet
  Then la réponse est 409

@spec-doe-3 Scenario: la fourniture est une copie
  Given un produit "Banc Garden" avec fournisseur "PLAS ECO" et 2 pages de fiche technique
  When je demande un brouillon de fourniture pour ce produit et l'enregistre dans le DOE
  And je renomme le produit "Banc Garden V2" et change l'adresse de "PLAS ECO"
  Then le DOE affiche toujours "Banc Garden" et l'ancienne adresse

@spec-doe-4 Scenario: numérotation sans les sections désactivées
  Given un DOE avec Végétaux (2 fournitures), Matériaux, Mobilier, Jeux désactivée, Photos
  When je construis le document de rendu
  Then les sections sont numérotées 1 à 4 et "Photos" porte le numéro 4
  And les fournitures de Végétaux sont "1.1" et "1.2"

@spec-doe-5 Scenario: le PDF contient tout
  Given un DOE avec 2 fournitures dont une avec un PDF fournisseur de 2 pages, 5 photos à 2 par page, 2 plans PDF d'une page, 1 page libre
  When je télécharge le PDF
  Then la réponse est un PDF de 14 pages : 3 (garde, intervenants, sommaire) + 1 + (1 + 2) (fournitures) + 3 (photos) + 3 (liste + 2 plans) + 1
  And son texte contient "Pépinière" et "Identification des végétaux" avec accents

@spec-doe-6 Scenario: isolation des organisations
  Given un DOE de l'organisation A
  When un membre de l'organisation B le lit, le modifie ou télécharge son PDF
  Then la réponse est 404

@spec-doe-7 Scenario: droits
  Given un membre sans planning.manage
  When il modifie un DOE
  Then la réponse est 403
  And il peut toujours le lire et télécharger le PDF

@spec-doe-8 Scenario: modèle de départ
  When je crée un modèle depuis le modèle de départ "landscaping"
  Then l'organisation a un modèle "Paysagiste" de 7 sections modifiable
  And le modifier ne change pas le modèle de départ

@spec-doe-9 Scenario: fiche technique et fournisseur
  When je crée un fournisseur "DUGOUR POUZZOLANE" avec adresse et téléphone
  And un produit avec ce fournisseur, fabricant "DUGOUR", caractéristiques et un PDF
  Then relire le produit rend ces champs
  And un fournisseur d'une autre organisation est refusé (404)

@spec-doe-10 Scenario: fichier manquant
  Given un DOE dont une photo a été supprimée du stockage
  When je télécharge le PDF
  Then la réponse est 409 et nomme la fourniture concernée

@spec-doe-11 Scenario: éditeur (front)
  Given le DOE d'un projet ouvert dans l'éditeur
  When je décoche "Jeux", pioche "Pouzzolane 10/20" dans le catalogue et enregistre
  Then l'onglet "Jeux" disparaît, la fourniture apparaît sous "2.n" et la page rechargée la montre
```

Tests : 1, 2, 3, 6, 7, 8, 9, 10 en intégration (`#[ignore]`, base réelle) ; 4 et les newtypes en unitaires ; 5 en test du crate `doc-render` (fixture en mémoire, page count via `lopdf`) ; 11 en vitest sur la feature.

## Simulation

Non : pas de concurrence ni de temps dans ce chantier. Deux éditions simultanées du même DOE : la dernière gagne, accepté en v1 (un DOE a un rédacteur).

## Workstreams

| Id | Mission | Dépend de | Modèle |
|---|---|---|---|
| W0 | Contrats : migrations, types de domaine, ports, commandes, marqueurs registry, crate `doc-render` vide, spec. **Orchestrateur.** | — | orchestrateur |
| W1 | `doc-render` : `TypstDoeRenderer` + gabarit `.typ` + tests (spike Typst inclus) | W0 | sonnet |
| W2 | Fiche technique produit + coordonnées et CRUD fournisseurs (core + handlers-reference + handlers-purchase) | W0 | sonnet |
| W3a | Modèles de DOE + modèle de départ + réglages (core + handlers-planning/doe_model) | W0 | sonnet |
| W3b | DOE : service, repo, use cases, handlers dont PDF (core + handlers-planning/doe) | W0 (port de rendu figé ; adapter branché à l'intégration) | sonnet |
| W4a | Front : fiche technique au catalogue, écran fournisseurs | W2 intégré + client API régénéré | sonnet |
| W4b | Front : éditeur DOE, modèles de DOE, réglages de mise en page | W3a + W3b intégrés + client API régénéré | sonnet |

W1, W2, W3a, W3b partent ensemble après W0. W4a part dès W2 intégré, W4b dès W3 intégré.

### Fichiers

- **W1** possède `libs/doc-render/**`, son `Cargo.toml` compris (Typst y est épinglé localement, comme `lopdf` dans `mestier-core`).
- **W2** possède `libs/core/src/domain/product/{service,commands}.rs`, `libs/core/src/infrastructure/product/**`, `libs/core/src/application/product/**`, `libs/core/src/domain/supplier_invoice/service.rs` (partie fournisseur), `libs/core/src/infrastructure/supplier_invoice/postgres/supplier_repository.rs` (nouveau), `libs/core/src/application/supplier/**` (nouveau), `libs/handlers-reference/src/product/**`, `libs/handlers-purchase/src/supplier/**` (nouveau).
- **W3a** possède `libs/core/src/domain/doe_model/{service,starter}.rs`, `libs/core/src/infrastructure/doe_model/**`, `libs/core/src/application/doe_model/**`, `libs/handlers-planning/src/doe_model/**`.
- **W3b** possède `libs/core/src/domain/doe/{service,document}.rs` (le type `DoeDocument` est dans `render.rs`, orchestrateur ; W3b n'écrit que ses `impl`), `libs/core/src/infrastructure/doe/**`, `libs/core/src/application/doe/**`, `libs/handlers-planning/src/doe/**`.
- **W4a** possède `apps/webapp/src/pages/catalog/**`, `apps/webapp/src/pages/suppliers/**` (nouveau), leurs routes.
- **W4b** possède `apps/webapp/src/pages/doe/**`, `apps/webapp/src/pages/doe-models/**` (nouveaux), la section réglages, leurs routes.

**Orchestrateur seul** : `migrations/*`, `Cargo.toml` (workspace et crates), `libs/core/src/domain/{doe,doe_model}/{mod,ports,commands}.rs`, `libs/core/src/domain/product/mod.rs`, `libs/core/src/domain/supplier_invoice/{mod,ports}.rs`, `libs/core/src/domain/mod.rs`, `libs/core/src/application/mod.rs`, `libs/core/src/infrastructure/{mod,registry}.rs`, `libs/core/src/lib.rs`, tous les `lib.rs`/`paths.rs`/`response.rs` de handlers, `apps/api/src/{router,openapi}.rs`, `apps/webapp/src/modules/registry.ts`.

**Régénérés à l'intégration, jamais édités à la main**
- `.sqlx/` : `cargo sqlx prepare --workspace -- --all-targets` (base migrée).
- `Cargo.lock` : `cargo check --workspace`.
- `apps/webapp/src/api/api.client.ts` : `pnpm gen:api` contre un serveur construit depuis la branche.
- `apps/webapp/src/routeTree.gen.ts` : `pnpm build`.

### Environnement par lot

Chaque lot a sa base (`mestier_wt_doe_w1`…`w3b`) et son `CARGO_TARGET_DIR` (`target/doe-w*`), voir `parallelisation.md`.

## Frozen contracts

Écrits en W0 dans les fichiers « orchestrateur seul » ci-dessus ; ce sont eux qui font foi. Résumé :

```rust
// domain/doe/mod.rs
pub struct DoeId(pub Uuid);
pub struct Revision(String);           // TryFrom<String> : 1..=8 caractères, trim
pub struct BlankPageCount(u8);         // TryFrom<u8> : 1..=20
pub enum PhotosPerPage { One, Two, Four }
pub enum MakerLabel { Manufacturer, Nursery }          // « Fabricant » / « Pépinière »
pub enum PartyRole { ProjectOwner, ProjectManager, Contractor } // MOA / MOE / entreprise
pub struct DoeParty { role, name: String, address, phone, email: Option<String>, logo_key: Option<String> }
pub struct DoeFile { key: String, caption: Option<String> }
pub struct SupplierSnapshot { name: String, address, phone, email: Option<String> }
pub struct DoeSupply {
    id: Uuid, source_product_id: Option<ProductId>, title: Option<String>, designation: String,
    dpgf_ref: Option<String>, destination: Option<String>, supplier: Option<SupplierSnapshot>,
    maker: Option<String>, specs: Option<String>, photos: Vec<DoeFile>, datasheet_keys: Vec<String>,
}
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DoeSectionContent {
    Supplies { maker_label: MakerLabel, supplies: Vec<DoeSupply> },
    Photos { per_page: PhotosPerPage, photos: Vec<DoeFile> },
    Plans { plans: Vec<DoeFile> },
    BlankPages { count: BlankPageCount },
}
pub struct DoeSection { id: Uuid, title: String, enabled: bool, content: DoeSectionContent }
pub struct Doe {
    id: DoeId, organization_id: OrganizationId, project_id: ProjectId,
    title: String, place: Option<String>, lot: Option<String>, issued_on: NaiveDate,
    author: Option<String>, revision: Revision, parties: Vec<DoeParty>, sections: Vec<DoeSection>,
    created_at, updated_at,
}
// domain/doe/document.rs (données de rendu, pures)
pub struct DoeDocument { header, parties, sections: Vec<NumberedSection>, accent: HexColor, logo_key: Option<String> }
impl DoeDocument { pub fn from_doe(doe: &Doe, settings: &DoeSettings) -> Self; pub fn file_keys(&self) -> Vec<&str>; }
pub struct FileBundle(HashMap<String, BundledFile { mime_type: String, bytes: Vec<u8> }>);
pub enum DoeRenderError { MissingFile { key: String }, UnsupportedFile { key: String, mime_type: String }, Typeset(String) }
pub trait DoeRenderer: Send + Sync { fn render(&self, doc: &DoeDocument, files: &FileBundle) -> Result<Vec<u8>, DoeRenderError>; }

// domain/doe_model/mod.rs
pub struct DoeModelId(pub Uuid);
pub struct HexColor(String);           // TryFrom<String> : #rrggbb
pub enum DoeModelSectionKind { Supplies { maker_label }, Photos { per_page }, Plans, BlankPages { count } }
pub struct DoeModelSection { title: String, enabled: bool, kind: DoeModelSectionKind }
pub struct DoeModel { id, organization_id, name: String, description: Option<String>, sections: Vec<DoeModelSection>, archived_at, created_at, updated_at }
pub enum StarterModel { Landscaping }
pub struct DoeSettings { organization_id, accent: HexColor, logo_key: Option<String> }  // défaut : #0f3d36, pas de logo

// domain/product/mod.rs : Product et ses commandes gagnent
supplier_id: Option<SupplierId>, manufacturer: Option<String>, technical_specs: Option<String>, datasheet_keys: Vec<String>,
// domain/supplier_invoice/mod.rs : Supplier gagne
address: Option<String>, phone: Option<String>, email: Option<String>,
```

## Risks / open questions

| Risque | Porteur | Parade |
|---|---|---|
| Typst 0.15 n'insère pas les PDF comme images, ou mal | W1 | Spike en premier ; repli `lopdf` (ADR 0003) sans changer le port |
| Rendu lent sur 40 pages de photos lourdes | W1 | Mesure dans le test ; si > 10 s, redimensionner les images avant rendu (lot suivant) |
| Limite d'upload trop basse pour les plans A1 | orchestrateur | Vérifier `max_upload_bytes` à l'intégration |
| Format des fiches fournisseurs chez Erwan (PDF, image, lien) | user | Le PDF et les images sont gérés ; un lien seul n'est pas rendu en v1 |
| Clés de fichiers non préfixées par organisation | orchestrateur | Hors chantier ; le DOE ne référence que des clés venues de ses propres écritures |

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
| W1 | intégré (a8054d3) |
| W2 | intégré (cd42271) |
| W3a | in-flight |
| W3b | in-flight |
| W4a | pending |
| W4b | pending |

## Decision log

- 2026-10-07 : W1, insertion PDF native Typst (`image(path, page: n)`), pas de repli `lopdf` ; police Inter embarquée ; `comemo::evict(0)` après chaque rendu pour un serveur à mémoire stable.
- 2026-10-07 : W2, `SupplierService` déplacé de `application/supplier/` vers `domain/supplier_invoice/supplier_service.rs` à l'intégration (règle hexagonale).
- 2026-10-07 : W4a attend la fin de W3 pour qu'un seul `pnpm gen:api` serve les deux lots front.

- 2026-10-07 : JSONB pour le contenu du DOE et des modèles, plutôt que 5 tables. Rejeté : tables normalisées, car rien n'interroge l'intérieur d'un DOE et le remplacement complet est l'opération normale.
- 2026-10-07 : fournisseurs sous `reference.manage`. Rejeté : un nouveau droit achats, car aucun n'existe et un fournisseur est un référentiel.
- 2026-10-07 : sauvegarde de l'éditeur par `PUT` complet. Rejeté : endpoints fins par fourniture, car ils multiplient les routes pour un document à un seul rédacteur.
- 2026-10-07 : le brouillon de fourniture est construit côté backend (`/supplies/draft`). Rejeté : copie côté front, car la règle de copie (dont l'adresse du fournisseur) doit avoir une seule source.

Amended: 2026-10-07 W1 possède le `Cargo.toml` de `doc-render` ; `DoeDocument` vit dans `domain/doe/render.rs` (orchestrateur) ; erreurs de validation en `Conflict` ; les chemins HTTP suivent la convention existante `/api/v1/organizations/{organization_id}/…` quand la ressource est listée par organisation.
