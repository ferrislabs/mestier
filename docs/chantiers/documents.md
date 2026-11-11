# Documents

État : **spec, 2026-11-11** (branche `feat/documents`, empilée sur `feat/tables`). Troisième chantier du module Connaissances, après le Drive et les Tables.

## Goal

Une entreprise écrit ses documents dans Mestier, par blocs comme dans Notion : texte mis en forme, listes, cases à cocher, encadrés, images, fichiers du Drive, mentions de clients, projets et devis, et vues de tables. Les documents vivent dans le Drive, à côté des fichiers, et l'assistant les lit et les écrit en Markdown.

## Contexte

- Vision validée le 2026-11-10 : Drive, puis Tables façon Airtable, puis Documents par blocs façon Notion qui affichent des vues de table. Édition : une personne à la fois, avec un verrou ; l'édition à plusieurs en temps réel est pour plus tard.
- Questions 7 à 10 du 2026-11-11, recommandations retenues (même lot de réponses que les Tables) :
  - 7a : les documents vivent dans le Drive (dossiers, étiquettes, recherche, rattachement) et les notes actuelles deviennent des documents ;
  - 8a : blocs texte, titres, listes, cases à cocher, citation, encadré, séparateur, code, image, fichier, mention, vue de table ;
  - 9a : une vue de table s'affiche en lecture, avec un bouton pour ouvrir la table ;
  - 10a : l'assistant lit et écrit un document en Markdown, sans suppression.
- Existant réutilisé :
  - `knowledge_articles` : titre, `body`, étiquettes, recherche française par déclencheur, dossier, rattachement `knowledge_links` ;
  - permissions `knowledge.view` et `knowledge.manage` ;
  - outils MCP `get_knowledge_article`, `create_knowledge_note`, `update_knowledge_article` ;
  - lignes de table par vue (`GET …/tables/{table_id}/rows?view_id=`) ;
  - URL signée des fichiers (`GET …/articles/{id}/download`) et des photos de la photothèque.

## Décisions

1. **Un document est un article `document` du Drive.** La valeur `note` disparaît : la migration convertit chaque note en document (une ligne de texte devient un paragraphe). Le document garde tout ce qu'a un article : dossier, étiquettes, recherche, rattachement.
2. **Le contenu est un format de blocs propre à Mestier** (ADR 0016), stocké dans `knowledge_articles.content JSONB`. Le domaine le valide (`DocumentContent::parse`). Ce n'est pas le JSON de l'éditeur : la webapp traduit dans les deux sens.
3. **`body` devient le Markdown du document**, réécrit par le serveur à chaque enregistrement. Il sert à la recherche (le déclencheur existant ne change pas), à la lecture par l'assistant et à l'export.
4. **Format des blocs** (version 1) :

   ```json
   { "version": 1, "blocks": [ { "id": "…", "type": "paragraph", "props": {}, "content": [], "children": [] } ] }
   ```

   | `type` | `props` | `content` | `children` |
   |---|---|---|---|
   | `paragraph` | | inline | oui |
   | `heading` | `level` 1 à 3 | inline | non |
   | `bullet_item`, `numbered_item` | | inline | oui |
   | `check_item` | `checked` | inline | oui |
   | `quote` | | inline | non |
   | `callout` | `tone` : `info`, `success`, `warning`, `danger` | inline | non |
   | `divider` | | | non |
   | `code` | `language` (facultatif, 32 caractères) | texte brut, un seul `text` sans style | non |
   | `image` | `source` : `drive` ou `photo`, `id`, `caption` (facultatif, 500 caractères), `width` 10 à 100 (%) | | non |
   | `file` | `article_id` (article `file` du Drive) | | non |
   | `table_view` | `table_id`, `view_id` | | non |

   Inline :
   - `{ "type": "text", "text": "…", "styles": { "bold", "italic", "underline", "strike", "code" } }`, chaque style à `true` ou absent ;
   - `{ "type": "link", "href": "https://…", "content": [text…] }` : `http`, `https` ou `mailto` seulement ;
   - `{ "type": "mention", "resource_type": "customer" | "project" | "quote", "resource_id": uuid, "label": "…" }`. `label` est le nom au moment de l'insertion. La webapp affiche le nom actuel ; le Markdown utilise `label`.

   Un `id` de bloc est une chaîne de 1 à 64 caractères, unique dans le document.
5. **Bornes** : 5 000 blocs, 6 niveaux d'imbrication, 20 000 caractères de texte par bloc, 1 Mo de JSON. Au-delà, l'enregistrement est refusé (422).
6. **Références sans clé étrangère** : une image, un fichier, une table, une vue ou une mention qui pointe vers un élément supprimé reste dans le document. La webapp affiche « Élément supprimé » et ne tombe jamais en erreur. À l'enregistrement, le serveur vérifie seulement que les ids nouvellement ajoutés appartiennent à l'organisation.
7. **Verrou d'édition** :
   - Pour modifier le contenu dans la webapp, il faut prendre le verrou du document : `POST …/articles/{id}/lock`.
   - Le verrou dure 90 secondes. La webapp le renouvelle toutes les 30 secondes tant que la page est ouverte, et le rend en quittant la page (`DELETE …/lock`).
   - Si quelqu'un d'autre le tient, la réponse est 409 avec son id et l'heure d'expiration. Le document s'ouvre alors en lecture, avec le bandeau « En cours de modification par X ».
   - Un verrou expiré est libre : la personne suivante le prend sans action particulière.
   - Le titre, les étiquettes, le dossier et le rattachement ne demandent pas le verrou.
8. **Enregistrement** : `PUT …/articles/{id}/content` avec `{ content, revision }`.
   - Il faut tenir le verrou, et `revision` doit être la révision actuelle. Sinon 409, avec le motif `lock_lost` ou `revision_conflict`.
   - La révision augmente de 1 à chaque enregistrement.
   - La webapp enregistre 1 seconde après la dernière frappe et affiche « Enregistré ».
9. **Écriture en Markdown** (API et assistant) :
   - `PATCH …/articles/{id}` avec `body`, et `POST …/notes`, reçoivent du Markdown. Le serveur le convertit en blocs, avec `pulldown-cmark`.
   - Ces écritures sont refusées (409) si quelqu'un d'autre tient le verrou. Elles augmentent la révision.
   - Liens `mestier:` dans le Markdown :
     - `[Nom](mestier:customer/<id>)`, de même avec `project` et `quote`, devient une mention ;
     - `![légende](mestier:drive/<id>)` ou `![légende](mestier:photo/<id>)`, seul dans son paragraphe, devient une image ;
     - `[titre](mestier:file/<id>)`, seul dans son paragraphe, devient un bloc fichier ;
     - `[Vue](mestier:table/<table_id>?view=<view_id>)`, seul dans son paragraphe, devient une vue de table.
   - Le rendu Markdown produit ces mêmes formes. Un encadré devient une citation qui commence par `**Info :**` (ou `Succès`, `Attention`, `Danger`).
   - Ce que le Markdown ne connaît pas (tableaux, HTML, images externes) devient du texte.
10. **Assistant (MCP)** :
    - Pas de nouvel outil. On garde les noms existants, parce que les clients MCP gardent la liste des outils en cache.
    - `get_knowledge_article` rend le Markdown d'un document.
    - `create_knowledge_note` crée un document à partir de Markdown. Sa description change ; son nom reste.
    - `update_knowledge_article` avec `body` remplace le contenu à partir de Markdown. Il est refusé si le document est en cours de modification.
    - Aucune suppression.
11. **Éditeur : BlockNote 0.56.0** (`@blocknote/core`, `@blocknote/react`, `@blocknote/shadcn`, licence MPL-2.0), fixé à cette version. Pas les paquets « XL » (IA, export PDF, multi-colonnes), qui ne sont pas sous MPL. Le schéma de l'éditeur ne contient que les blocs de la décision 4 : le menu « / » ne propose rien d'autre. La page de l'éditeur est chargée à la demande, pour ne pas alourdir le reste de l'application.
12. **Droits** :
    - `knowledge.view` lit un document et voit qui tient le verrou ;
    - `knowledge.manage` crée, prend le verrou, enregistre et supprime.
    - Sans `knowledge.manage`, le document s'ouvre toujours en lecture.

## Invariants

1. Toute lecture et écriture est bornée par `org_id`. Un document, ou un id ajouté dans un bloc (fichier, photo, table, vue, client, projet, devis), d'une autre organisation est « introuvable ». [test]
2. Le contenu stocké est toujours un `DocumentContent` valide. [type : construit seulement par `DocumentContent::parse` ou par la conversion du Markdown ; test]
3. `body` est toujours le rendu Markdown de `content`, écrit dans la même requête SQL. [use case ; test]
4. Une seule personne tient un verrou non expiré à la fois. [prise du verrou par un seul `UPDATE … WHERE lock_holder IS NULL OR lock_expires_at < now() OR lock_holder = $user` ; test de concurrence]
5. Le contenu ne change jamais sans que la révision augmente. [test]
6. Une écriture dans la webapp ne réussit qu'avec le verrou en cours et la bonne révision. [test]
7. Rendre puis relire en Markdown redonne les mêmes blocs, sauf les ids, la largeur des images et le `language` d'un bloc de code vide. [test de propriété]
8. Un lien n'accepte que `http`, `https` et `mailto`. [constructeur ; test]
9. La migration convertit toutes les notes sans perte de texte, et sa version `.down.sql` les restaure en notes. [test]

## Hors périmètre

- L'édition à plusieurs en temps réel (curseurs, Yjs), l'historique des versions, les commentaires.
- Les sous-pages et l'arborescence à la Notion : les dossiers du Drive en tiennent lieu.
- Une vue de table modifiable dans le document (9b).
- Le partage public et l'export PDF ou Word d'un document.
- Les tableaux simples dans un document (hors vue de table), la vidéo, l'audio, les colonnes.
- Les mentions de membres et de documents.
- Un événement de workflow « document modifié ».
- La création automatique d'un rattachement quand on mentionne un client, un projet ou un devis.

## Conception

- **Migration** `20261120000001_knowledge_documents` :
  - ajoute à `knowledge_articles` : `content JSONB NULL`, `revision INT NOT NULL DEFAULT 0`, `lock_holder UUID NULL`, `lock_expires_at TIMESTAMPTZ NULL` ;
  - convertit les notes en documents et passe `kind` à `document` ;
  - change la contrainte de forme : un document a `content` et `body` non nuls, et `body` peut être vide. Un fichier n'a ni `content` ni verrou.
  - `.down.sql` : `kind` repasse à `note`. Un document au `body` vide reçoit son titre comme `body`. Les colonnes sont supprimées.
- **Core**, dans `domain/knowledge/documents/` :
  - `content.rs` : `DocumentContent`, `Block`, `BlockKind`, `Inline`, `TextStyles`, les bornes et `parse` ;
  - `markdown.rs` : `render(&DocumentContent) -> String` et `from_markdown(&str) -> DocumentContent` ;
  - `lock.rs` : `EditLock`, avec la durée et la règle « libre ou expiré » ;
  - `references.rs` : la liste des ids référencés et la différence avec le contenu précédent ;
  - le port `KnowledgeDocumentRepository` : prendre, renouveler et rendre le verrou, enregistrer le contenu avec la révision, lire.
  - La vérification des références ajoutées réutilise les dépôts existants : articles, photos, tables, clients, projets, devis.
- **Use cases**, dans `application/knowledge/documents.rs` : `create_knowledge_document`, `acquire_document_lock`, `release_document_lock`, `save_document_content`. Les use cases existants `create_knowledge_note` et `update_knowledge_article` passent par `from_markdown`.
- **API**, dans `handlers-knowledge/src/documents.rs`, sous `/api/v1/organizations/{organization_id}/knowledge/articles` :
  - `POST /documents` `{ title, folder_id?, tags? }` crée un document vide (201) ;
  - `POST /{id}/lock` rend `{ holder_id, expires_at, revision }` (200), ou 409 `{ holder_id, expires_at }` ;
  - `DELETE /{id}/lock` rend 204, et ne fait rien si le verrou n'est pas à soi ;
  - `PUT /{id}/content` `{ content, revision }` rend `{ revision, updated_at }` (200), 409 `{ reason }`, ou 422 ;
  - `GET /{id}` rend aussi `content`, `revision` et `lock` (`{ holder_id, expires_at }` ou `null`) pour un document.
  - Corps limité à 1 Mo + 64 Ko sur `content`.
- **Webapp** :
  - route `_app.o.$organizationSlug.connaissances.documents.$articleId.tsx`, page `src/pages/knowledge-documents/{feature,ui,lib}` ;
  - `lib/blocks.ts` traduit les blocs BlockNote vers le format Mestier et inversement, avec des tests ;
  - blocs personnalisés : encadré, image (Drive ou photothèque, ou dépôt d'une nouvelle image dans le dossier du document), fichier, vue de table (en lecture, 50 premières lignes de la vue, bouton « Ouvrir la table ») ;
  - mention par « @ » : clients, projets, devis ;
  - Drive : bouton « Nouveau document » dans le dossier ouvert ; un document s'ouvre sur sa page, pas dans la fenêtre de détail. La fenêtre « Nouvelle note » disparaît. Sur la page, un panneau « Infos » donne les étiquettes, le dossier et le rattachement.
  - Bandeau de verrou, indicateur « Enregistrement… » puis « Enregistré ». En cas de 409, l'éditeur passe en lecture avec « Ce document a été modifié ailleurs » et un bouton « Recharger ».
- **Tranché en route (O2)** :
  - Le JSON envoyé à l'enregistrement contient toujours `props`, `content` et `children` (`content: []` pour séparateur, image, fichier et vue de table) et `styles` sur chaque texte. Une image porte toujours `width` (100 par défaut) et `caption` seulement si elle n'est pas vide. Un bloc de code sans `language` est le langage « text » de l'éditeur.
  - La webapp ne peut pas envoyer ce que le format refuse, donc elle l'écarte à l'enregistrement : un bloc image, fichier ou vue de table dont la cible n'est pas encore choisie, les paragraphes vides en fin de document, et un lien autre que `http`, `https` ou `mailto` (le texte reste, le lien disparaît). Un lien sans schéma qui ressemble à un domaine reçoit `https://`. Les enfants d'un bloc qui n'en accepte pas (titre, citation, encadré, code) remontent au même niveau, juste après lui.
  - Le verrou se renouvelle par le même `POST …/lock`. Si la révision rendue n'est pas celle lue avec le document, ou si le verrou est refusé, l'éditeur passe en lecture avec « Ce document a été modifié ailleurs ». Quand la page se cache (`pagehide`), la webapp envoie l'enregistrement en attente, sinon le `DELETE …/lock`, avec `keepalive`. Si un enregistrement est déjà parti, ou si le contenu dépasse 60 Ko (les navigateurs plafonnent le corps d'une requête `keepalive` autour de 64 Ko), seul le `DELETE …/lock` est envoyé. Une révision qui change pendant un renouvellement n'est pas un conflit si un enregistrement de la même page a eu lieu entre-temps.
  - Une image du Drive se charge par `GET …/articles/{id}/download` (les octets, pas une URL signée). Une photo de la photothèque passe par l'URL signée de sa clé.
  - Une mention affiche le nom actuel lu sur la fiche du client, du projet ou du devis. Une réponse 404 donne « Élément supprimé » ; toute autre erreur laisse le `label` enregistré. La liste du « @ » reprend les 100 premiers clients, projets et devis, comme le reste de la webapp.
  - La page de document ouverte sur un article `file` répond « Cet élément n'est pas un document » et renvoie au Drive.
- **Tranché en route (O1)** :
  - `from_markdown` rend un `Result` : un Markdown qui dépasse les bornes de la décision 5 est refusé (422), il n'est jamais tronqué. Les titres de niveau 4 à 6 deviennent des titres de niveau 3, et un saut de ligne dans un titre devient une espace.
  - Le Markdown ne garde pas les enfants d'un paragraphe : au rendu, ils suivent le paragraphe au même niveau. Un paragraphe vide et une citation vide disparaissent du Markdown (le `content` les garde). Le test de propriété porte donc sur des contenus sans bloc vide ni espace en bord de bloc.
  - Le rendu écrit le souligné et le barré en `<u>` et `<del>`, et le gras et l'italique en `<strong>` et `<em>` quand `**` et `*` ne se reliraient pas (ponctuation au bord, balise voisine). La relecture lit ces balises comme des styles ; tout autre HTML reste du texte.
  - Un bloc fichier et une vue de table se rendent `[Fichier](mestier:file/…)` et `[Vue](mestier:table/…?view=…)` : le rendu ne connaît pas les titres.
  - La vérification des références passe par un port `DocumentReferenceRepository` et une requête `EXISTS` par id et par organisation, au lieu des dépôts des entités : un fichier ou une image du Drive doit être un article `file`, une vue doit appartenir à sa table. Ce port sert aussi à `POST …/notes` et au `PATCH` avec `body`.
  - Les réponses 409 du verrou et de l'enregistrement gardent le corps d'erreur habituel (`code`, `message`, `status`) avec `holder_id` et `expires_at`, ou `reason`, au même niveau. Les réponses 200 gardent l'enveloppe `data`. `lock` est toujours présent dans un article (`null` si libre, expiré, ou pour un fichier).
  - Un `PATCH` avec `body` ou un `POST …/notes` sur un document que la même personne verrouille passe ; le `body` vide ou fait d'espaces reste refusé (422), comme avant. Un document créé depuis du Markdown est à la révision 0.
  - Migration : une ligne de plus de 20 000 caractères est coupée en plusieurs paragraphes, et `body` devient les paragraphes séparés par une ligne vide, sans échappement. Le `.down.sql` rend à `body` une ligne par paragraphe pour les documents faits seulement de paragraphes de texte ; les autres gardent leur Markdown. `lock_holder` référence `users` avec `ON DELETE SET NULL`.
  - Un `PATCH` avec `body` ne prend pas de révision : il remplace le contenu même si le détenteur du verrou (la même personne) a des modifications non enregistrées ; l'éditeur se recharge alors sur `revision_conflict`.
  - Le `body` migré est le texte brut de la note, pas le rendu échappé ; le prochain enregistrement le réécrit.
  - Migration : une note qui donnerait plus de 5 000 paragraphes est découpée par 10 000 caractères au plus et ses lignes consécutives sont réunies (séparées par `\n`) dans un même paragraphe, pour rester dans les bornes.
  - Le Markdown refuse plus de 32 conteneurs imbriqués (citations et listes) avant même le contrôle des 6 niveaux, pour ne pas bâtir un arbre sans borne.

## Lots

| Lot | Contenu | Qui |
|---|---|---|
| O1 | Migration, domaine (blocs, bornes, Markdown dans les deux sens, verrou, références), dépôt, use cases, API, adaptation des routes notes et des outils MCP, tests | agent backend |
| O2 | Webapp : dépendances BlockNote, traduction des blocs, éditeur et blocs personnalisés, verrou et enregistrement, intégration au Drive | agent webapp |

O2 démarre en parallèle sur la traduction des blocs et l'éditeur, avec le contrat de cette spec. Il branche l'API générée quand O1 est fini.

## Acceptance

O1
1. Créer un document dans un dossier : il apparaît dans ce dossier avec `kind = document`, un contenu vide et la révision 0.
2. Prendre le verrou, enregistrer un contenu avec la révision 0 : réponse révision 1, `body` contient le Markdown, et la recherche retrouve un mot du document.
3. Deux personnes prennent le verrou en même temps : une seule l'obtient, l'autre reçoit 409 avec l'id de la première.
4. Un verrou non renouvelé depuis 90 secondes est pris par une autre personne sans erreur.
5. Enregistrer sans verrou, avec le verrou d'un autre, ou avec une révision périmée : 409 avec le bon motif, rien ne change.
6. Un contenu invalide est refusé avec 422 : type inconnu, titre de niveau 4, lien `javascript:`, id de bloc en double, 7 niveaux d'imbrication, plus de 5 000 blocs, bloc de code avec style.
7. Un bloc fichier, une image, une vue de table ou une mention qui vise un élément d'une autre organisation : 404, rien n'est enregistré.
8. Un document dont la table référencée a été supprimée se lit et se réenregistre sans erreur.
9. `PATCH` avec `body` en Markdown (titres, listes, cases à cocher, citation, code, lien, mention `mestier:`) : les blocs attendus sont stockés et la révision augmente. Refusé avec 409 si quelqu'un d'autre tient le verrou.
10. Rendu puis relecture en Markdown : mêmes blocs (test de propriété sur des contenus générés).
11. Migration : une note de trois lignes devient un document de trois paragraphes, avec le même texte. Après `.down.sql`, c'est de nouveau une note avec le même `body`.
12. Une personne avec `knowledge.view` seulement lit le document et voit le verrou. Elle reçoit 403 sur la prise du verrou et sur l'enregistrement.
13. MCP : l'assistant crée un document en Markdown, le lit en Markdown et le modifie. La modification est refusée pendant qu'une personne l'édite. Aucune suppression.

O2
14. La traduction BlockNote vers le format Mestier, puis retour, redonne les mêmes blocs, pour chaque type de bloc et d'inline.
15. Le menu « / » ne propose que les blocs de la décision 4.
16. Ouvrir un document que personne n'édite : l'éditeur est modifiable, et le verrou est renouvelé toutes les 30 secondes.
17. Ouvrir un document qu'un autre édite : lecture seule, avec le bandeau qui donne son nom.
18. Taper du texte : « Enregistré » s'affiche, et le texte est là après rechargement de la page.
19. Perdre le verrou (409 `lock_lost`) : l'éditeur passe en lecture et propose « Recharger ».
20. Insérer une mention « @ » d'un client : elle affiche le nom actuel du client et mène à sa fiche.
21. Insérer une vue de table : elle montre les lignes de la vue en lecture, et « Ouvrir la table » mène à la table sur cette vue. Une table supprimée affiche « Élément supprimé ».
22. Insérer une image du Drive et une de la photothèque : elles s'affichent. Une nouvelle image déposée arrive dans le dossier du document.
23. Dans le Drive, « Nouveau document » crée un document dans le dossier ouvert et ouvre sa page. Un document s'ouvre sur sa page.

## Verify (exit)

```bash
cargo fmt --all --check
SQLX_OFFLINE=true cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p mestier-core -- --ignored knowledge
cargo test -p handlers-knowledge --test http_e2e
cargo test -p mcp-server
docker run --rm -v "$PWD/deploy/cerbos/policies:/policies" ghcr.io/cerbos/cerbos:0.56.0 compile --tests=/policies/tests /policies
cd apps/webapp && pnpm check && pnpm test && pnpm build
```
