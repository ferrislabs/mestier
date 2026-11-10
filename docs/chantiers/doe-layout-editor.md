# DOE : éditeur de mise en page

État : **spec, 2026-11-10** (branche `feat/doe-layout-editor`).

## Goal

Un client règle les éléments graphiques de ses DOE dans un éditeur plein écran : il pose des images (envoyées ou prises dans la photothèque), les déplace, les fait pivoter, les retourne en miroir, et voit le résultat tel qu'il sortira dans le PDF.

## Context

- Suite de `doe-layout.md` (#632) : les éléments (image, rectangle, ligne) existent, se glissent et se redimensionnent sur l'aperçu de l'onglet « Éléments ». La rotation y était hors périmètre.
- Demande client du 2026-11-10 : « customiser leurs thèmes plus facilement avec une vraie interface ; prendre des images, les placer, les faire pivoter, changer de sens ».
- Réponses du 2026-11-10 : « changer de sens » = miroir horizontal et vertical ; rotation par poignée, aimantée tous les 15°, plus un champ au degré ; éditeur plein écran avec calques et repères d'alignement ; images envoyées ou prises dans la photothèque ; un élément pivoté peut dépasser de la page, la partie hors page est coupée.
- La photothèque (#659) supprime le fichier quand on supprime la photo. Un style ne peut donc pas pointer sur la clé d'une photo de la photothèque.

## Décisions

1. **Trois champs de plus sur `DoeDecoration`** : `rotation` (degrés entiers, 0 à 359, sens horaire), `flip_x`, `flip_y` (booléens). Absents du JSON = 0 et `false`, donc un style enregistré rend pareil.
2. **Transformation autour du centre du cadre** : miroir d'abord, rotation ensuite (CSS `rotate(θ) scale(sx, sy)`, Typst `rotate(θ, scale(...))`). `x`, `y`, `width`, `height` restent le cadre avant rotation.
3. **Validation inchangée sur le cadre avant rotation** : il doit tenir dans la page. Les coins pivotés peuvent sortir, la page les coupe (PDF et SVG).
4. **Rotation normalisée à l'enregistrement** : `-90` devient `270`, `360` devient `0`, `12.4` devient `12`.
5. **Photothèque par copie** : `POST /organizations/{org}/photo-library/photos/{id}/copy` copie le fichier dans le stockage et renvoie une nouvelle clé, comme un envoi. Supprimer la photo ensuite ne touche pas le style. Même permission que la lecture de la photothèque (`MANAGE_PLANNING`) ; l'onglet est caché sans elle.
6. **Éditeur plein écran, pas de nouvelle route** : un bouton « Ouvrir l'éditeur » dans l'onglet « Éléments » ouvre une fenêtre plein écran sur les éléments du formulaire en cours. « Valider » garde les changements dans le formulaire, « Annuler » les rejette ; l'enregistrement reste celui du formulaire (Paramètres › Documents ou modèle). Les deux points de montage en profitent sans rien changer.
7. **Repères magnétiques** : bords et centre de la page, bords et centres des autres éléments de la page affichée (boîte englobante, rotation comprise), seuil 2 mm. `Alt` pendant le geste les coupe.
8. **Annuler / rétablir** dans l'éditeur (`Ctrl+Z`, `Ctrl+Maj+Z`), 50 pas, perdu à la fermeture.

## Invariants

1. Un style sans `rotation` ni `flip_*` rend le même document qu'avant. [test : désérialisation par défaut ; rendu]
2. Une rotation non finie est refusée à l'enregistrement ; une rotation finie est toujours rangée dans 0 à 359. [type : `Rotation`]
3. Le PDF et l'aperçu SVG posent le même angle dans le même sens que l'éditeur (sens horaire, autour du centre). [test : rendu d'un rectangle à 90° ; test unitaire de la transformation CSS]
4. Copier une photo de la photothèque crée un fichier distinct ; supprimer la photo ne casse pas le style. [test]
5. La copie exige `MANAGE_PLANNING` et une photo de l'organisation ; une photo d'une autre organisation donne 404. [test]
6. L'éditeur ne change le formulaire qu'à « Valider ». [test : lib d'historique]

## Out of scope

- Texte libre, groupes, verrouillage d'un élément, dégradés, ombres.
- Rognage d'image dans l'éditeur.
- Réordonner les calques par glisser-déposer (boutons et raccourcis seulement).
- Éléments propres à un DOE ; outils MCP de mise en page.
- Autres formats que l'A4 portrait.

## Design

- **Core** `domain/doe_model/style.rs` : `Rotation(u16)` (`TryFrom<f32>`, `rem_euclid(360)`, arrondi), `DoeDecoration { rotation, flip_x, flip_y }` avec `#[serde(default)]`, aussi dans `RawDecoration`. Rien à changer dans `DocumentStyle` : les décorations sont sérialisées telles quelles vers Typst.
- **Core** `domain/photo_library` : `PhotoLibraryService::copy_file(org, id, storage)` lit le fichier et l'envoie dans le dossier des fichiers de style, renvoie la clé. Use case `copy_library_photo` (`library_photo, member, role, authz`), même contrôle que `get_library_photo`.
- **doc-render** `doe.typ` : dans `decor`, le contenu de l'élément passe par `scale(x: ±100%, y: ±100%)` puis `rotate(d.rotation * 1deg)`, `origin: center`, `reflow: false`.
- **API** `handlers-planning/src/photo_library/copy.rs` : `POST …/photo-library/photos/{photo_id}/copy` → `{ data: { key } }`. OpenAPI régénéré, `pnpm gen:api`.
- **Webapp** `pages/doe-style/` :
  - `lib/transform.ts` : transformation CSS, boîte englobante pivotée, angle depuis le pointeur et aimantation 15°, miroirs (testés).
  - `lib/snap.ts` : repères et aimantation (testés).
  - `lib/history.ts` : pile annuler / rétablir pure (testée).
  - `feature/layout-editor-feature.tsx` : état de l'éditeur, historique, photothèque (`useQuery`), copie et envoi.
  - `ui/layout-editor.tsx` : fenêtre plein écran. À gauche les pages (couverture, pages intérieures) et les calques (du dessus au dessous, monter, descendre, dupliquer, supprimer). Au centre la page agrandie, l'aperçu SVG et la couche de poignées (déplacer, redimensionner, poignée de rotation, repères). À droite les propriétés (x, y, largeur, hauteur, angle, miroirs, portée, couche, couleur, opacité, ajustement). En haut la barre d'outils de l'élément choisi : pivoter -15° / +15°, miroir horizontal, miroir vertical, devant / derrière, dupliquer, supprimer, annuler, rétablir.
  - `ui/photo-library-picker.tsx` : grille des vignettes de la photothèque, recherche, un clic ajoute l'image.
  - `ui/decoration-overlay.tsx` : applique la rotation et le miroir au cadre affiché ; la couche de poignées de l'onglet « Éléments » reste utilisable.
  - Raccourcis : flèches (1 mm, `Maj` 10 mm), `Suppr`, `Ctrl+D`, `Ctrl+Z`, `Ctrl+Maj+Z`, `Échap` désélectionne.

## Acceptance

1. Un style enregistré avant ce chantier se relit avec `rotation = 0`, sans miroir, et son export est inchangé.
2. `PUT doe-settings` avec un élément `rotation = -90` : enregistré à `270`. `rotation = "NaN"` ou non numérique : 422, rien d'enregistré.
3. Rendu d'un rectangle `rotation = 90` : sa boîte dans le SVG est tournée dans le sens horaire autour de son centre, comme la transformation CSS calculée par `lib/transform.ts`.
4. Une image `flip_x = true` est rendue en miroir dans le PDF et l'aperçu.
5. Un élément pivoté dont un coin sort de la page : enregistré, export 200.
6. `POST …/photos/{id}/copy` : 200 avec une clé différente de celle de la photo ; la photo supprimée ensuite, l'export du style garde l'image. 403 sans `MANAGE_PLANNING`, 404 pour une photo d'une autre organisation.
7. Éditeur : ajouter une image depuis la photothèque, la glisser, la pivoter avec la poignée (aimantée à 45°), la retourner, « Valider », enregistrer : l'export la montre au même endroit, au même angle.
8. Éditeur : « Annuler » après des changements laisse le formulaire tel qu'il était ; `Ctrl+Z` défait le dernier geste.
9. Repères : un élément glissé à moins de 2 mm du centre de la page s'y colle, et un trait de repère s'affiche ; avec `Alt`, non.

## Workstreams

| Id | Mission | Qui |
|---|---|---|
| E-A | `Rotation`, champs `flip_*`, `doe.typ`, copie depuis la photothèque (service, use case, handler), OpenAPI | moi |
| E-B | Webapp : `lib/` (transform, snap, history), éditeur plein écran, photothèque, overlay | sous-agent, après `pnpm gen:api` |

Une branche `feat/doe-layout-editor`, une PR.

## Risks

| Risque | Parade |
|---|---|
| Sens de rotation différent entre Typst et CSS | Acceptance 3, testée sur un rendu réel |
| L'aperçu SVG redessiné saute pendant un geste | Position locale pendant le geste, aperçu à la fin (comme aujourd'hui) |
| Copies répétées de la même photo qui encombrent le stockage | Accepté en v1, comme les envois : les clés de style ne sont pas nettoyées aujourd'hui |

## Verify (exit)

```bash
cargo fmt --all --check
SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p mestier-core -- --ignored
cargo test -p handlers-planning --test http_e2e -- --ignored
cd apps/webapp && pnpm check && pnpm test && pnpm build
```
