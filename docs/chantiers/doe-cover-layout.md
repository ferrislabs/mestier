# DOE : couverture à blocs placés librement

État : **spec, 2026-11-12** (branche `feat/drive-preview` pour l'instant, à rebaser sur une branche dédiée).

## Goal

Un client place où il veut les éléments de la couverture de son DOE : logo, infos entreprise, lieu, bandeau titre, photo principale, bloc rédacteur / date / indice. Il les glisse et les redimensionne dans l'éditeur plein écran existant et voit le PDF tel qu'il sortira.

## Context

- Demande client du 2026-11-12 : « personnaliser là où ils veulent placer leurs éléments comme ils veulent, le titre, l'emplacement de l'image principale, les infos rédacteur et DOE ».
- Réponses du 2026-11-12 : **couverture seulement** ; placement **libre, comme les décorations** (position et taille en mm, glisser-déposer, repères magnétiques).
- Aujourd'hui la couverture est un flux vertical fixe dans `doe.typ` : logo + infos entreprise, lieu, bandeau, tableau rédacteur, photo qui prend la place restante. Seuls la position et la taille du logo se règlent (`cover_logo_position`, `cover_logo_size`).
- Les décorations (`DoeDecoration`) posent déjà des images et rectangles en mm ; l'éditeur (`layout-editor`) sait les déplacer et les redimensionner sur l'aperçu SVG rendu par l'API.

## Décisions

1. **Un champ `cover_layout` sur `DoeStyle`**, `Option<CoverLayout>`, `#[serde(default)]`. `None` = la couverture en flux d'avant, rendu inchangé. `Some` = chaque bloc est posé à son cadre.
2. **Six blocs fixes**, un cadre chacun : `logo`, `company` (infos de l'entreprise adjudicatrice), `place`, `title` (bandeau : titre du document, titre du DOE, lot), `meta` (date, rédacteur, indice), `photo`. Pas de bloc ajouté ni dupliqué.
3. **Un cadre** = `x`, `y`, `width`, `height` en mm (type `Millimeters`), plus `visible: bool` (défaut `true`). Il doit tenir dans la page A4, largeur et hauteur > 0, comme une décoration. Pas de rotation ni de miroir.
4. **Contenu d'un bloc inchangé**, seul son emplacement change : le bandeau garde la couleur principale et ses polices, `company` reste aligné à droite, `meta` en tableau, le logo et la photo se contiennent dans leur cadre (`fit: contain` pour le logo, `cover` pour la photo). `cover_logo_position` et `cover_logo_size` ne servent plus quand `cover_layout` existe.
5. **Un texte qui dépasse son cadre est coupé** (`clip: true`), pas réduit. L'éditeur le montre : le cadre est ce qui s'imprime.
6. **Un bloc masqué** (`visible = false`) n'est pas imprimé. `photo` masqué équivaut à `cover_photo = false`.
7. **Valeurs de départ** : « Personnaliser la couverture » dans l'éditeur remplit `cover_layout` avec des cadres qui reproduisent la couverture en flux (logo en haut à gauche, `company` en haut à droite, bandeau au tiers, etc.). `None` revient par « Rétablir la couverture par défaut ».
8. **Données de la couverture inchangées** : toujours `doc.title`, `doc.lot`, `doc.place`, `doc.author`, `doc.revision`, `style.cover_title`, `cover_photo_key`, l'entreprise adjudicatrice. Aucun nouveau champ dans le DOE.
9. **Les décorations restent au-dessus ou au-dessous** des blocs selon leur `layer`, comme aujourd'hui.
10. **Éditeur** : les blocs sont des calques de la page « Couverture », avec les mêmes poignées, repères magnétiques, annuler / rétablir et propriétés (x, y, largeur, hauteur, visible). Un bloc ne se supprime ni ne se duplique ; on le masque.
11. **Pas de nouvelle API** : `cover_layout` voyage dans `DoeStyle`, donc dans `doe-settings`, les modèles et le style de projet.

## Invariants

1. Un style sans `cover_layout` rend la même couverture qu'avant. [test : désérialisation par défaut ; rendu]
2. Un cadre hors de la page, ou de largeur ou hauteur nulle, est refusé à l'enregistrement. [type : désérialisation validée]
3. Le PDF et l'aperçu SVG placent chaque bloc au même endroit que l'éditeur (mm depuis le coin haut gauche de la page, marges ignorées). [test : rendu]
4. Un bloc masqué n'apparaît ni dans le PDF ni dans l'aperçu. [test : texte du PDF]
5. La couverture reste une seule page, quelles que soient les valeurs. [test : nombre de pages]

## Out of scope

- Autres pages (intervenants, sommaire, fiches), en-tête et pied de page.
- Blocs de texte libres, blocs supplémentaires, rotation, miroir, ordre des calques entre blocs.
- Réduction automatique du texte, alignement et police réglables par bloc.
- Outils MCP de mise en page.

## Design

- **Core** `domain/doe_model/style.rs` : `CoverBlock { x, y, width, height, visible }` (validation du cadre comme `RawDecoration`), `CoverLayout { logo, company, place, title, meta, photo: CoverBlock }`, `DoeStyle.cover_layout: Option<CoverLayout>`. `DocumentStyle` le recopie.
- **doc-render** `doe.typ` : la page de couverture choisit entre le flux actuel (`cover_layout == none`) et `place(top + left, dx, dy, block(width, height, clip: true, …))` par bloc. Les contenus des six blocs deviennent des fonctions partagées par les deux modes.
- **Webapp** `pages/doe-style/` :
  - `lib/cover-layout.ts` : cadres de départ, validation, conversion bloc ↔ élément d'éditeur (testés).
  - `ui/layout-editor.tsx` et `feature/layout-editor-feature.tsx` : les blocs apparaissent dans les calques de la couverture ; propriétés réduites (x, y, largeur, hauteur, visible).
  - `ui/doe-style-form.tsx` : boutons « Personnaliser la couverture » et « Rétablir la couverture par défaut ».
  - `lib/style.ts`, `themes.ts` : `cover_layout: null` au défaut.
  - `pnpm gen:api` après le Rust.

## Acceptance

1. Style enregistré avant ce chantier : `cover_layout` absent, export identique.
2. `PUT doe-settings` avec un cadre hors page ou de taille nulle : 422, rien d'enregistré.
3. Avec `cover_layout` où `title` est en bas de page et `photo` en haut : le PDF montre le bandeau en bas et la photo en haut, à ±0,5 mm des valeurs.
4. `place.visible = false` : le lieu n'est plus dans le texte de la page 1.
5. Cadre trop petit pour le titre : export 200, couverture sur une seule page, texte coupé.
6. Éditeur : « Personnaliser la couverture », glisser le bandeau, « Valider », enregistrer : l'export le place au même endroit.
7. « Rétablir la couverture par défaut » puis enregistrer : retour au flux d'origine.

## Workstreams

| Id | Mission | Qui |
|---|---|---|
| C-A | Core (`CoverLayout`, validation), `DocumentStyle`, `doe.typ`, tests de rendu, OpenAPI | moi |
| C-B | Webapp : `lib/cover-layout.ts`, calques de blocs dans l'éditeur, boutons du formulaire | sous-agent, après `pnpm gen:api` |

## Risks

| Risque | Parade |
|---|---|
| Un cadre trop petit rend un texte illisible | Texte coupé et visible dans l'éditeur (décision 5) ; pas de garde-fou en v1 |
| L'aperçu SVG et le PDF divergent d'un mm | Acceptance 3, testée sur un rendu réel |
| Style existant dont le thème change les couleurs mais pas le layout | `cover_layout` n'est pas dans `ThemeLook` : un thème ne le touche pas |

## Verify (exit)

```bash
cargo fmt --all --check
SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd apps/webapp && pnpm check && pnpm test && pnpm build
```
