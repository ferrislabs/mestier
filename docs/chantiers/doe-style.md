# DOE : style du PDF

État : **spec, 2026-10-11**.

## Goal

Une organisation règle l'apparence de ses DOE (couleurs, polices, couverture, en-tête, pied de page, libellés), un modèle DOE peut la remplacer, et un bouton génère un PDF d'aperçu avec le style en cours avant d'enregistrer.

## Context

- Demande client du 2026-10-11, relayée après la recette MCP (#541).
- Aujourd'hui : `doe_settings` (une ligne par organisation) porte `accent` et `logo_key`, réglés dans Paramètres › Documents (`documents-section.tsx`). Le reste est figé dans `libs/doc-render/src/doe.typ` : police Inter, blanc sur l'accent, libellés, en-tête « DOE – titre », pied « date / page ».
- Un DOE ne garde pas le modèle dont il vient (`CreateDoeCommand.model_id` sert à copier les sections, rien n'est stocké).
- Le serveur MCP réécrit un DOE à partir d'une liste fixe de champs (`apps/mcp-server/src/session.rs`, `update_body`) : un champ ajouté au `PUT` et absent de cette liste serait effacé à chaque écriture de l'assistant.
- Réponses du 2026-10-11 : tout est réglable (couleurs, police, couverture, en-tête/pied/libellés) ; organisation par défaut, remplacée par le modèle ; aperçu par bouton.

## Décisions

1. **Un objet `DoeStyle`**, le même pour l'organisation et le modèle. Un modèle a `style: Option<DoeStyle>` : absent, le style de l'organisation s'applique ; présent, il remplace **tout** le style (pas de fusion champ par champ, l'écran propose « partir du style de l'organisation »).
2. **Le style est lu à l'export, pas copié dans le DOE.** Le DOE retient son modèle (`does.model_id`, nul pour les DOE existants et si le modèle disparaît) ; changer le style change les prochains exports. Les fournitures restent des copies, le style non : c'est de l'habillage.
3. **La photo de couverture appartient au DOE** (`does.cover_photo_key`), le style ne dit que si et comment elle s'affiche.
4. **Polices intégrées**, pas d'envoi de police : Inter, Montserrat, Source Serif 4, Barlow (OFL, régulier + gras, vendorisées comme Inter). Police des titres et police du texte au choix, séparément.
6. **Aperçu en direct** (ajout du 2026-10-11) : le serveur rend la couverture, les intervenants et une fiche de fourniture du DOE d'exemple en SVG (`typst-svg`, même gabarit que le PDF), redessinés pendant la saisie. Mesuré : 6 ms en release, 323 Ko pour les 3 pages. Le bouton PDF reste pour le document entier.
7. **Thèmes prêts à l'emploi** : quelques points de départ (couleurs, polices, couverture) proposés dans l'éditeur. Un thème appliqué devient le style de l'organisation ou du modèle ; logo, en-tête, pied et libellés sont gardés. Pas de bibliothèque de thèmes enregistrés : le style de l'organisation est le thème que ses utilisateurs reçoivent.
5. **Gabarits d'en-tête et de pied** : texte libre avec `{titre}`, `{lieu}`, `{lot}`, `{indice}`, `{date}`, remplacés côté Rust dans `DoeDocument::from_doe` (le renderer ne décide rien).

## `DoeStyle`

| Champ | Type | Défaut (rendu actuel) |
|---|---|---|
| `primary` | `HexColor` | `#0f3d36` (ancien `accent`) |
| `secondary` | `HexColor` | `#0f3d36` (titres de niveau 2, libellés d'intervenant) |
| `on_primary` | `HexColor` | `#ffffff` (texte sur les bandeaux) |
| `heading_font`, `body_font` | `DoeFont` (`inter`, `montserrat`, `source_serif`, `barlow`) | `inter` |
| `logo_key` | `Option<String>` | `None` (ancien `logo_key`) |
| `cover_logo_position` | `left`, `center`, `right` | `left` |
| `cover_logo_size` | `small`, `medium`, `large` | `medium` (2,4 cm) |
| `cover_photo` | `bool` | `true` (sans effet si le DOE n'a pas de photo) |
| `header_logo` | `bool` | `false` |
| `header_text` | `DoeTemplate` | `DOE – {titre}` |
| `footer_text` | `DoeTemplate` | `{date}` |
| `cover_title` | `DoeLabel` | `DOSSIER DES OUVRAGES EXÉCUTÉS (D.O.E.)` |
| `parties_title` | `DoeLabel` | `INTERVENANTS` |
| `contents_title` | `DoeLabel` | `SOMMAIRE` |
| `supply_table_title` | `DoeLabel` | `Identification de la fourniture` |

`DoeLabel` : non vide, 120 caractères au plus. `DoeTemplate` : 200 caractères au plus, vide permis (rien d'affiché), seules les cinq variables connues acceptées. L'indice et le numéro de page restent à droite de l'en-tête et du pied, non réglables. Tous les champs sont facultatifs dans le JSON (défaut ci-dessus), pour qu'un style enregistré reste lisible quand un champ s'ajoute.

## Invariants

1. Un style par défaut produit le même document qu'avant ce chantier : mêmes couleurs, police, libellés, en-tête et pied. [test : `DoeStyle::default()` comparé aux valeurs d'avant ; rendu]
2. Couleurs, polices, positions et tailles ne peuvent pas être invalides une fois construites. [type : `HexColor`, enums]
3. Un gabarit avec une variable inconnue (`{client}`) ou un libellé vide est refusé à l'enregistrement, pas au rendu. [type : `DoeTemplate`, `DoeLabel`]
4. Style appliqué : celui du modèle du DOE s'il en a un, sinon celui de l'organisation, sinon le défaut. Un modèle archivé garde son style ; un modèle supprimé ou un DOE sans modèle prend celui de l'organisation. [test]
5. L'assistant MCP n'efface jamais la photo de couverture : `update_body` la reporte. [test]
6. L'aperçu n'écrit rien, ne lit aucune donnée de chantier, et passe par les mêmes limites de rendu que l'export (2 rendus simultanés, 120 s). [test]
7. Toute écriture de style exige `reference.manage` ; l'aperçu aussi. [test]
8. Migration réversible : `accent`/`logo_key` passent dans `style` et en reviennent. [test : up puis down sur une base avec réglages]

## Out of scope

- Envoi de polices par l'organisation.
- Style par DOE.
- Mise en page libre de la couverture (blocs déplaçables), choix du format de page.
- Libellés des champs du tableau de fourniture (Désignation, DPGF…).
- Outils MCP de réglage du style.

## Design

- **Migration** `20261011000001_doe_style` : `doe_settings.style JSONB NOT NULL` rempli depuis `accent`/`logo_key`, puis suppression des deux colonnes ; `doe_models.style JSONB NULL` ; `does.model_id UUID NULL REFERENCES doe_models(id) ON DELETE SET NULL`, `does.cover_photo_key TEXT NULL`. Down : l'inverse (accent et logo repris de `style`).
- **Core** `domain/doe_model` : `DoeStyle` et ses types ; `DoeSettings { organization_id, style }` ; `DoeModel.style`. `domain/doe` : `Doe.model_id`, `Doe.cover_photo_key` (`UpdateDoeCommand` porte la photo, pas le modèle) ; `DoeService::document` résout le style (invariant 4) ; `DoeDocument` reçoit `style` résolu, `header`/`footer` déjà remplis et `cover_photo_key`. Aperçu : `DoeDocument::sample(style)` (DOE fictif : une section de fournitures, une section de photos vide) rendu par le renderer existant.
- **doc-render** : 4 familles de polices dans `fonts/`, `doe.typ` lit tout de `doc.style`.
- **API** : `GET/PUT /organizations/{org}/doe-settings` → `{ style }` ; `style` facultatif sur création et mise à jour de modèle ; `POST /organizations/{org}/doe-style/preview` (corps `DoeStyle`, réponse `application/pdf`) ; `cover_photo_key` dans le `PUT` et la réponse du DOE ; `model_id` dans la réponse. OpenAPI régénéré.
- **MCP** : `update_body` reporte `cover_photo_key` ; `add_doe_file` accepte `target: cover_photo` (photo uniquement), `delete_doe_file` sait la retirer.
- **Webapp** : composant d'édition de style partagé (`pages/doe-style/` : couleurs, polices, couverture, en-tête et pied, libellés, bouton aperçu) monté dans Paramètres › Documents et dans l'éditeur de modèle (case « style propre à ce modèle », préremplie depuis l'organisation) ; photo de couverture dans l'en-tête de l'éditeur de DOE.

## Acceptance

1. Une organisation sans réglage exporte un DOE aux valeurs d'avant (couleur `#0f3d36`, Inter, libellés d'avant).
2. `PUT doe-settings` avec couleurs, polices et libellés modifiés, puis export : le DOE les reflète (vérifié sur les données passées au renderer et par un rendu réel).
3. `PUT doe-settings` avec `{client}` dans `header_text`, un libellé vide ou `#12345` : refusé comme les autres erreurs de validation de l'API (409 ou 422), rien d'enregistré.
4. Un modèle avec style propre : ses DOE prennent ce style ; un DOE d'un modèle sans style, ou sans modèle, prend celui de l'organisation.
5. `header_text = "{titre} – {lot}"` donne « Avenue du 8 mai – Lot 2 » dans l'en-tête ; un champ vide donne une chaîne vide, pas `{lot}`.
6. Photo de couverture : enregistrée par le `PUT`, affichée si `cover_photo = true`, absente sinon ; comptée dans les limites de fichiers.
7. Aperçu : 200 `application/pdf` sans rien écrire ; 403 sans `reference.manage` ; style invalide refusé.
8. MCP : `update_doe_header` sur un DOE avec photo de couverture la conserve ; `add_doe_file target=cover_photo` la pose, un PDF y est refusé.
9. Migration : up sur une base avec `accent = #123456` et un logo, le style les contient ; down les remet en colonnes.
11. Aperçu en pages : `POST /organizations/{org}/doe-style/preview/pages` renvoie 3 SVG, 403 sans `reference.manage` ; l'écran les redessine pendant la saisie (anti-rebond), et un clic sur un thème change l'aperçu sans enregistrer.
10. Webapp : régler le style dans Paramètres › Documents, aperçu, enregistrer ; cocher « style propre » dans un modèle préremplit depuis l'organisation.

## Workstreams

| Id | Mission | Qui |
|---|---|---|
| S-A | Migration, types `DoeStyle`, settings, modèle, DOE (`model_id`, `cover_photo_key`), résolution, `DoeDocument`, handlers, aperçu, OpenAPI | moi |
| S-B | Polices + `doe.typ` piloté par `style`, tests de rendu | sous-agent, après le contrat `DoeDocument` |
| S-C | MCP : report de la photo, `target: cover_photo` | moi |
| S-D | Webapp : éditeur de style, modèle, photo de couverture | sous-agent, après `pnpm gen:api` |

Une branche `feat/doe-style`, une PR.

## Risks

| Risque | Parade |
|---|---|
| Binaire plus lourd (4 familles × 2 graisses) | Fichiers statiques régulier + gras seulement, ~2 Mo en tout ; mesuré dans la PR |
| Couleur de texte illisible sur le bandeau | Choix laissé à l'utilisateur, l'aperçu le montre ; pas de contrôle de contraste en v1 |
| L'aperçu sert à saturer le rendu | Mêmes sémaphore et délai que l'export |

## Verify (exit)

```bash
cargo fmt --all --check
SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p mestier-core -- --ignored
cargo test -p handlers-planning --test http_e2e -- --ignored
cd apps/webapp && pnpm check && pnpm test && pnpm build
```
