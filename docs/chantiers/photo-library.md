# Photothèque d'organisation

État : **spec figée le 2026-11-09 (contrat W0). Implémentation par lots W1 à W4, branche d'intégration `feat/photo-library`.**

## Goal

Le paysagiste dépose ses photos de chantier dans la webapp, une fois, en lot. Son assistant (ChatGPT, Claude, Mistral) les voit, les légende, les étiquette et les place dans un DOE. Avec le temps, la photothèque devient une base de connaissances de l'entreprise : réalisations, végétaux, réceptions.

## Context

- `add_doe_file` (voir `mcp-doe-lot1.md`) ne marche qu'avec ChatGPT (`openai/fileParams`) ; le repli base64 est plafonné à 5 Mo et un modèle ne peut pas recopier une image. Une photo par appel.
- Le modèle ne transporte jamais d'octets : le dépôt se fait dans la webapp, le MCP lit, légende, étiquette, place.
- Stockage : port `FileStorage` (`upload`, `get`, `delete`, `presigned_get_url`). Un DOE référence ses photos par clé (`DoeFile {key, caption}`, 300 fichiers max, 30 sections max).
- Les routes ouvertes aux tokens MCP sont listées dans `libs/handlers/src/delegated.rs`.

## Décisions

1. **Portée : l'organisation.** Une photothèque par organisation, le projet d'origine est un attribut. Tout est filtré par `org_id`.
2. **Copie figée dans le DOE.** Placer une photo duplique l'objet de stockage ; supprimer ou retoucher la photothèque ne change jamais un DOE.
3. **Droits : `MANAGE_PLANNING`** (webapp) ; côté MCP, tout reste sous le scope et le domaine `doe` existants (`mestier.doe.write`), pas de nouveau scope FerrisKey.
4. **Dépôt dans la webapp uniquement.** Le serveur ne reçoit que du JPEG, PNG ou WebP.
5. **HEIC converti dans le navigateur** (bibliothèque WASM chargée à la demande, seulement quand un .heic/.heif est déposé). Si la conversion échoue, le fichier est refusé avec un message clair.
6. **Étiquettes libres**, pas de lien au catalogue.
7. **Normalisation à l'upload** (rotation EXIF, redimensionnement, retrait du GPS) : dans ce lot, côté serveur.
8. **Les photos des DOE existants remontent dans la photothèque** (lot W4).
9. **Aucune suppression par MCP.**

## Modèle

Table `library_photos` : migration `migrations/20261109000008_photo_library.{up,down}.sql` (déjà écrite, c'est le contrat).

| Colonne | Sens |
|---|---|
| `file_key`, `thumbnail_key` | Objets de stockage, clés opaques : `<préfixe>/library-<org_id>/<uuid>` (le préfixe est celui du stockage) |
| `mime_type`, `width`, `height`, `size_bytes` | De l'image normalisée |
| `content_hash` | SHA-256 de l'image normalisée, unique par organisation |
| `caption`, `tags` | Légende (nulle si vide), jusqu'à 20 étiquettes |
| `project_id` | Projet d'origine, facultatif |
| `taken_at` | `DateTimeOriginal` de l'EXIF, lu avant de retirer les métadonnées |

« À classer » = pas de légende **et** aucune étiquette. Ce n'est pas stocké, c'est un filtre.

### Normalisation (serveur, à l'upload)

1. Décoder JPEG, PNG ou WebP (type détecté sur le contenu, pas sur l'en-tête). Autre chose : 422.
2. Lire `DateTimeOriginal` et l'orientation EXIF (JPEG).
3. Appliquer l'orientation, ré-encoder : toutes les métadonnées (GPS compris) disparaissent.
4. Si le plus grand côté dépasse 2400 px, redimensionner à 2400.
5. Encoder en JPEG qualité 85 (une transparence est aplatie sur blanc). `mime_type` vaut donc `image/jpeg`.
6. Miniature : plus grand côté 400 px, JPEG qualité 75.
7. `content_hash` = SHA-256 des octets normalisés. Si l'organisation a déjà cette image, ne rien stocker et renvoyer l'existante.
8. Plafond d'entrée : `file_storage.max_upload_bytes` (10 Mo par défaut), 413 au-delà.

## Contrat HTTP

Préfixe `/api/v1/organizations/{organization_id}`. Permission `MANAGE_PLANNING` partout, lectures comprises : un membre sans ce droit reçoit 403 avant tout 404. Les réponses suivent l'enveloppe de l'API : `{"data": …}` ; la liste ajoute `"pagination"` (`per_page`, `current_page`, `total`, …) ; le placement renvoie `{"data": {"doe", "results"}}`. `LibraryPhoto` :

```json
{
  "id": "uuid", "file_key": "string", "thumbnail_key": "string",
  "mime_type": "image/jpeg", "width": 2400, "height": 1600, "size_bytes": 812345,
  "caption": "string|null", "tags": ["string"], "project_id": "uuid|null",
  "taken_at": "RFC3339|null", "created_at": "RFC3339", "updated_at": "RFC3339"
}
```

| Route | Rôle |
|---|---|
| `POST /photo-library/photos?project_id=` | Corps brut, `Content-Type` image. **201** + `LibraryPhoto` si nouvelle, **200** + la photo existante si doublon. |
| `GET /photo-library/photos` | Liste paginée (`page`, `per_page` comme `libs/pagination`), tri `created_at` décroissant. Filtres : `q` (texte, insensible à la casse, sur légende et étiquettes), `tag` (répétable, ET), `project_id`, `to_sort=true` (à classer). |
| `GET /photo-library/photos/{id}` | Une photo. 404 si autre organisation. |
| `GET /photo-library/photos/{id}/thumbnail` | Octets JPEG de la miniature (`Content-Type: image/jpeg`). Sert le MCP. |
| `PATCH /photo-library/photos/{id}` | `caption` (chaîne vide efface), `tags` (remplace la liste), `project_id` (null efface). Champs absents inchangés. |
| `DELETE /photo-library/photos/{id}` | Supprime la ligne et les deux objets. Ne touche à aucun DOE. |
| `POST /does/{doe_id}/photos/place` | Placement, ci-dessous. |

Étiquettes : coupées, minuscules, sans doublon, 1 à 40 caractères, 20 au plus (sinon 422).

### Placement

Corps : `{ "placements": [ { "photo_id", "target", "section_id"?, "section_title"?, "supply_id"?, "caption"? } ] }`, 50 éléments au plus.

- `target` : `section` (section photos, `section_id` ou `section_title`, facultatif s'il n'y a qu'une section photos), `supply` (`supply_id`), `cover` (photo de couverture, remplace l'ancienne).
- `caption` absente : reprend la légende de la photothèque.
- **Si le DOE n'a aucune section photos et que `target=section`, elle est créée** (« Photos générales du chantier »).
- Chaque photo placée est **copiée** : nouvel objet de stockage (même convention de clé que le dépôt de fichiers DOE), la clé de la photothèque n'est jamais référencée par un DOE.
- Les éléments invalides (photo inconnue ou d'une autre organisation, section ou fourniture introuvable, plafond de 300 fichiers dépassé) sont ignorés et rapportés ; les valides s'écrivent **en une seule écriture** du DOE.
- Réponse 200 : `{ "doe": <DOE mis à jour>, "results": [ { "photo_id", "status": "placed"|"failed", "key"?, "error"? } ] }`.
- DOE inconnu ou d'une autre organisation : 404.

## Contrat MCP

Les trois outils sont dans le domaine `doe` (scope `mestier.doe.write` pour l'écriture, `mestier.read` pour la lecture). Leurs routes sont à ouvrir dans `libs/handlers/src/delegated.rs`. `add_doe_file` reste inchangé.

| Outil | Entrée | Sortie |
|---|---|---|
| `search_photo_library` (lecture) | `organization?`, `query?`, `tags?[]`, `project_id?`, `to_sort?`, `limit?` (défaut 8, 12 si miniatures, 50 sinon), `page?`, `include_thumbnails?` (défaut vrai) | Texte JSON : `{photos:[{id, caption, tags, project_id, taken_at, width, height}], page, total}` puis une image JPEG par photo (miniature), dans l'ordre. |
| `tag_library_photos` | `organization?`, `items:[{photo_id, caption?, add_tags?[], remove_tags?[]}]` (50 au plus) | `{results:[{photo_id, status, error?}]}` |
| `place_doe_files` | `organization?`, `doe_id`, `placements:[…comme ci-dessus]` (50 au plus) | `{doe, results}` du placement |

Descriptions d'outils : courtes, en anglais, avec les valeurs de `target` énumérées. Les limites (50, 12) sont dans les descriptions.

## Webapp

- Module **Projet** : entrée « Photothèque » dans la barre latérale (`/projet/photos`), permission `MANAGE_PLANNING`, déclarée dans `apps/webapp/src/modules/registry.ts`.
- Écran : zone de dépôt multi-fichiers (réutilise `file-drop-zone.tsx`), 3 envois en parallèle, état par fichier (en cours, envoyé, doublon, refusé). Grille de miniatures, filtres (texte, étiquette, projet, « à classer »), ouverture d'une photo pour modifier légende, étiquettes et projet, suppression avec confirmation.
- HEIC/HEIF : converti en JPEG dans le navigateur avant envoi (import dynamique de la bibliothèque, jamais chargée sans fichier HEIC). Échec : message « Ce fichier HEIC n'a pas pu être converti, exportez-le en JPEG ».
- Éditeur du DOE : bouton « Choisir dans la photothèque » dans l'onglet photos et dans la fiche fourniture, qui appelle le placement. Même séparation feature/ui que le reste (`ui/` sans hook).

## Lots, fichiers possédés, dépendances

Un fichier n'a qu'un seul propriétaire. Aucun lot ne commit : l'orchestrateur commit par lot.

| Lot | Possède | Dépend de |
|---|---|---|
| W0 | `docs/chantiers/photo-library.md`, `migrations/20261109000008_*` | rien (fait) |
| W1 backend | `libs/core/src/{domain,application,infrastructure}/photo_library/**` (+ une méthode de placement dans le contexte `doe`), `libs/core/src/infrastructure/registry.rs`, `libs/core/src/lib.rs`, `libs/handlers-planning/src/photo_library/**` et les 2 lignes de son `lib.rs`, `apps/api/src/{router,openapi}.rs`, `.sqlx/`, `Cargo.toml` (workspace et crates), tests | W0 |
| W2 webapp | `apps/webapp/src/pages/photo-library/**`, `apps/webapp/src/hooks/use-photo-library.ts`, `apps/webapp/src/routes/**` (routes photothèque), `apps/webapp/src/modules/registry.ts`, sélecteurs dans `apps/webapp/src/pages/doe/**` | contrat HTTP (types écrits à la main depuis cette spec) |
| W3 MCP | `apps/mcp-server/**`, `libs/mcp-tools/**`, `libs/handlers/src/delegated.rs`, `docs/chantiers/mcp.md` | contrat HTTP (le faux serveur des tests suit cette spec) |
| W4 données | Binaire `apps/api/src/bin/backfill_photo_library.rs` (idempotent via `content_hash`), cas d'usage `backfill_organization_photos`, ses tests | W1 mergé |

W2 et W3 démarrent en parallèle de W1. Un écart entre un lot et le contrat se règle en amendant cette spec, pas en divergeant.

## Acceptance

1. Déposer un JPEG avec GPS et orientation EXIF : la photo stockée n'a aucune métadonnée, est tournée, et `taken_at` est lu.
2. Déposer une image de 5000 px : le plus grand côté stocké vaut 2400, la miniature 400.
3. Déposer deux fois la même image : 201 puis 200, une seule ligne et deux objets.
4. Déposer un PDF, un fichier vide, un fichier corrompu : 422 ; au-delà du plafond : 413.
5. `GET` avec `to_sort=true` ne renvoie que les photos sans légende ni étiquette ; `tag` répété fait un ET ; `q` cherche dans légende et étiquettes.
6. Un membre d'une autre organisation reçoit 404 sur lecture, modification, suppression, miniature et placement.
7. Une étiquette vide, trop longue, ou une 21e : 422.
8. Placer 3 photos dans un DOE sans section photos : la section est créée, 3 copies existent, `key` des fichiers du DOE ≠ `file_key` de la photothèque.
9. Supprimer une photo de la photothèque après placement : le DOE et son PDF sont inchangés.
10. Placement avec un élément invalide parmi 3 : les 2 valides sont écrits en une écriture, l'invalide est rapporté `failed`.
11. Placement au-delà de 300 fichiers : les éléments en trop sont rapportés `failed`, le DOE reste valide.
12. `target=cover` remplace la photo de couverture.
13. MCP : `search_photo_library` renvoie le texte puis autant d'images que de photos (miniatures) ; `include_thumbnails=false` n'en renvoie aucune.
14. MCP : `tag_library_photos` et `place_doe_files` renvoient un résultat par élément, un élément en échec n'empêche pas les autres ; aucun outil ne supprime.
15. Les tokens MCP ouvrent exactement les routes de lecture, de modification d'une photo et de placement, pas la suppression ni l'upload.
16. Webapp : déposer 5 fichiers dont 1 HEIC et 1 PDF : 4 envoyés (le HEIC converti), le PDF refusé avec son message, états visibles par fichier.
17. Webapp : « Choisir dans la photothèque » dans l'onglet photos place la sélection et rafraîchit l'éditeur.
18. Reprise (W4) : relancée deux fois, elle ne crée aucun doublon ; chaque photo des DOE existants (sections, fournitures, couverture) a son entrée avec le projet d'origine.

## Lancer la reprise (W4)

Une organisation à la fois, avec la même configuration que l'API (base, stockage) :

```bash
PHOTO_BACKFILL_ORGANIZATION=<uuid> cargo run -p api --bin backfill_photo_library
```

Elle lit chaque DOE de l'organisation, une transaction par DOE. Reprend la couverture, les sections photos et les photos des fournitures, avec la légende et le projet d'origine. Les plans et fiches techniques ne sont pas des photos et restent en place. Une copie placée depuis la photothèque est reconnue par son hash et n'est pas dupliquée. Le binaire sort en erreur si une photo n'a pas pu être copiée ; le relancer reprend les manquantes.

## Vérification (sortie)

```bash
cargo fmt --all --check
SQLX_OFFLINE=true cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p mestier-core -- --ignored        # avec DATABASE_URL, migration appliquée
cd apps/webapp && pnpm check && pnpm test
```

`cargo sqlx prepare --workspace -- --all-targets` est à relancer après tout nouveau `query!` (W1, W4). La migration doit se rejouer vers le bas puis vers le haut.

## Hors périmètre

Suppression par MCP, recherche sémantique, lien des étiquettes au catalogue, dépôt par lien mobile sans connexion, édition d'image (recadrage), partage entre organisations.
