# DOE : éléments de mise en page répétés

État : **implémenté, 2026-11-07** (branche `feat/doe-layout`). Écarts avec la spec ci-dessous.

## Goal

Une organisation (ou un modèle) ajoute des éléments graphiques à ses DOE (image, rectangle, ligne), les place sur la page, et choisit s'ils apparaissent sur la couverture, sur les pages intérieures ou sur toutes.

## Context

- Demande client du 2026-11-07 : « une sorte de layout qui permet d'ajouter des éléments graphiques et les répliquer sur chaque page ».
- `DoeStyle` (voir `doe-style.md`) fixe aujourd'hui couleurs, polices, logo, en-tête, pied et libellés. La « mise en page libre de la couverture » était hors périmètre ; elle entre ici.
- Réponses du 2026-11-07 : éléments = image, rectangle, ligne (pas de texte, l'en-tête et le pied le font déjà) ; placement par glisser-déposer sur l'aperçu ; portée choisie par élément.
- La couverture a été réordonnée dans la même demande (titre dans le cadre, photo sous le rédacteur) : PR séparée, déjà faite.

## Décisions

1. **`DoeStyle.decorations: Vec<DoeDecoration>`**, 20 au plus, dans l'ordre d'empilement (le dernier est dessus). Absent du JSON = liste vide, donc un style enregistré reste lisible.
2. **Coordonnées en millimètres sur la page A4** (210 × 297), origine en haut à gauche : `x`, `y`, `width`, `height`. L'éditeur convertit pixels et millimètres, le renderer ne fait que poser.
3. **Trois sortes** : `image` (une clé de fichier, `fit` contain ou cover, `opacity` 0 à 1), `rectangle` (`fill` HexColor, `opacity`, `radius` mm), `line` (`color`, `thickness` en mm ; c'est un rectangle de hauteur ou largeur fine, sans type propre côté renderer).
4. **Portée** : `pages` vaut `cover`, `inner` ou `all`. `inner` = toutes les pages sauf la couverture. Les pages de fiches techniques PDF plein cadre (`full_pages`) sont exclues : elles n'ont ni en-tête ni pied, on ne les recouvre pas.
5. **Derrière ou devant le contenu** : champ `layer` = `behind` (défaut, via `page(background:)`) ou `front` (`page(foreground:)`).
6. **Les images viennent du stockage de l'organisation**, comme le logo : même envoi, même limite de taille, comptées dans le paquet de fichiers (`FileBundle`) et dans les limites d'export.
7. **Géométrie validée à l'enregistrement** : dans la page (marge zéro permise, débordement refusé), taille > 0.

## Invariants

1. Un style sans `decorations` rend le même document qu'avant. [test : rendu comparé]
2. Un élément hors page, de taille nulle ou d'opacité hors [0, 1] est refusé à l'enregistrement, jamais au rendu. [type : `Millimeters`, `Opacity`]
3. Plus de 20 éléments, ou une clé d'image inconnue de l'organisation : refusé. [test]
4. Une image supprimée du stockage ne casse pas l'export : l'élément est ignoré et l'export réussit. [test]
5. Un élément `inner` n'apparaît ni sur la couverture ni sur une page de fiche technique PDF ; `cover` n'apparaît que sur la couverture. [test : texte/pixels par page]
6. L'aperçu SVG montre les mêmes éléments, aux mêmes endroits, que le PDF. [test : même gabarit]
7. Toute écriture exige `reference.manage`, comme le reste du style. [test]

## Out of scope

- Texte libre (en-tête et pied le couvrent), rotation, groupes, alignement magnétique.
- Éléments propres à un DOE (le style reste org ou modèle).
- Dégradés, ombres.
- Autres formats de page que l'A4 portrait.

## Design

- **Core** `domain/doe_model/style.rs` : `DoeDecoration { kind, x, y, width, height, pages, layer }` en enum interne `kind` (`Image`, `Rectangle`, `Line` se réduit à `Rectangle` côté rendu). Types `Millimeters`, `Opacity` validés à la construction. `DocumentStyle.decorations` les recopie, clés d'image comprises.
- **Application** : l'enregistrement du style vérifie que chaque clé d'image appartient à l'organisation ; `FileBundle` reçoit ces images à l'export et à l'aperçu.
- **doc-render** `doe.typ` : une fonction `decor(pages_filter)` construit le `place(top + left, dx:, dy:, ...)` de chaque élément ; appelée dans `background` / `foreground` des trois `set page`, avec le filtre `cover` / `inner`. Les `full_pages` passent `background: none, foreground: none`.
- **API** : aucun nouveau point d'entrée, `decorations` voyage dans `DoeStyle` (settings, modèle, aperçu). OpenAPI régénéré, `pnpm gen:api`.
- **Webapp** `pages/doe-style/` : onglet « Éléments » dans l'éditeur de style.
  - Liste des éléments (ajouter image, rectangle, ligne ; réordonner ; supprimer ; dupliquer).
  - Aperçu en pages existant, sur lequel une couche transparente porte des poignées : déplacer et redimensionner au pointeur, flèches du clavier pour ajuster au millimètre.
  - Panneau de l'élément sélectionné : valeurs numériques (x, y, largeur, hauteur), portée, couche, couleur, opacité, image.
  - Logique de conversion pixels ↔ mm et de bornage dans `lib/` (testée), la couche de poignées dans `ui/` sans hook de données.
- **MCP** : rien. Les outils de style restent hors périmètre.

## Acceptance

1. `PUT doe-settings` avec un rectangle `inner` en bas de page : l'export le montre sur les pages intérieures, pas sur la couverture.
2. Une image `all` en `behind` : visible sur couverture et pages intérieures, sous le texte.
3. Un élément `x = 200, width = 20` (déborde) : 409 ou 422, rien d'enregistré.
4. Une image dont le fichier a disparu : export 200, élément absent.
5. Un style sans `decorations` : sortie identique à celle de la PR couverture.
6. Éditeur : ajouter un rectangle, le glisser, l'aperçu suit ; enregistrer ; l'export le reflète au même endroit (à 1 mm près).
7. Un modèle avec style propre garde ses éléments, et le bouton « partir du style de l'organisation » copie aussi les éléments.

## Workstreams

| Id | Mission | Qui |
|---|---|---|
| L-A | Types, validation, `DocumentStyle`, vérification des clés d'image, `FileBundle`, OpenAPI | moi |
| L-B | `doe.typ` : `decor`, filtres, exclusion des `full_pages`, tests de rendu | sous-agent, après le contrat `DocumentStyle` |
| L-C | Webapp : liste, panneau, couche de poignées, conversions | sous-agent, après `pnpm gen:api` |

Une branche `feat/doe-layout`, posée sur `feat/doe-cover-layout`.

## Risks

| Risque | Parade |
|---|---|
| Le glisser-déposer sur un SVG redessiné à chaque frappe saute | Position locale pendant le geste, requête d'aperçu à la fin du geste (anti-rebond existant) |
| Grosses images qui alourdissent chaque page | Une image est incluse une fois dans le PDF par Typst ; limite de poids à l'envoi, mesurée dans la PR |
| Élément qui masque le texte | Couche `behind` par défaut, aperçu visible avant d'enregistrer |

## Verify (exit)

```bash
cargo fmt --all --check
SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p mestier-core -- --ignored
cd apps/webapp && pnpm check && pnpm test && pnpm build
```

## Écarts constatés à l'implémentation

- **Opacité d'une image** : Typst n'a pas d'opacité sur les images. Elle est rendue par un voile blanc, donc juste en couche `behind` ; en `front`, elle est sans effet. Le rectangle n'a pas cette limite.
- **Clé d'image inconnue de l'organisation** (invariant 3) : non vérifiée, comme pour le logo (les clés ne sont pas contrôlées aujourd'hui). Une clé inexistante est ignorée à l'export (invariant 4).
- **Redimensionnement au clavier** : non fait, seules les flèches de déplacement existent ; la taille se règle à la souris ou dans les champs numériques.
- **Gestes** : testés en unitaire (conversions, bornage), pas dans un navigateur.
