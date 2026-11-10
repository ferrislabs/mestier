# Base de connaissances

État : **spec validée le 2026-11-09, K0 à K4 implémentés sur `feat/knowledge-base`. Pas encore recetté dans un navigateur ni avec un client MCP réel.**

## Goal

Une entreprise dépose dans la webapp ses documents (devis, tarifs fournisseurs, CGV, fiches) et ses notes. Son assistant (ChatGPT, Claude, Mistral) les retrouve par le MCP, s'en sert pour créer des ressources (devis, projets) et rattache le document source à la ressource créée.

## Décisions déjà prises (2026-11-09)

1. **Contenu : fichiers et notes texte.** Pas de fiches typées (fournisseur, produit, prix) dans ce chantier.
2. **Recherche : plein texte Postgres et étiquettes.** Pas d'embeddings. Le texte des PDF est extrait au dépôt.
3. **Rattachement : projets, clients, devis.** Un article peut être lié à plusieurs ressources, une ressource à plusieurs articles. Pas de DOE dans ce lot.
4. **Droits : deux permissions dédiées `VIEW_KNOWLEDGE` et `MANAGE_KNOWLEDGE`** (même découpage lecture et écriture que les autres domaines), accordées aux rôles admin par migration, pas au rôle member. Isolation par `org_id`.

## Hypothèses à confirmer

- **Portée : l'organisation**, comme la photothèque. Tout est filtré par `org_id`.
- **Dépôt dans la webapp uniquement.** Le modèle ne transporte jamais d'octets (voir `photo-library.md`). Le MCP lit, cherche, crée des notes, étiquette et rattache.
- **Fichiers acceptés : PDF, DOCX, XLSX, images, texte.** Texte extrait pour PDF et texte brut seulement. Les PDF scannés n'ont pas de texte : pas d'OCR dans ce lot, l'article reste cherchable par titre et étiquettes.
- **Taille : 25 Mo par fichier**, texte extrait plafonné à 500 000 caractères.
- **Aucune suppression par MCP.** Le MCP peut créer, étiqueter et rattacher, pas supprimer.
- **Le rattachement est une copie de référence, pas du contenu** : supprimer un article détache ses liens, ne touche pas la ressource.

## Modèle (proposition)

Table `knowledge_articles` : `id`, `org_id`, `title`, `kind` (`file` | `note`), `body` (texte de la note), `file_key`, `mime_type`, `size_bytes`, `extracted_text`, `tags text[]` (20 max), `search tsvector` (généré, config `french`, pondéré titre > étiquettes > texte), `created_by`, `created_at`, `updated_at`.

Table `knowledge_links` : `article_id`, `org_id`, `resource_type` (`project` | `customer` | `quote`), `resource_id`, `created_at`, clé primaire sur les quatre.

Migrations réversibles, `.down.sql` testé. Index GIN sur `search` et sur `tags`.

## API webapp

Sous `libs/handlers-knowledge` (nouveau crate) ou un module de `handlers-planning` : à trancher.

- `POST /knowledge/articles` (multipart fichier ou JSON note), `GET` liste filtrée (texte, étiquettes, ressource), `GET /{id}`, `PATCH /{id}` (titre, note, étiquettes), `DELETE /{id}`.
- `GET /{id}/download` : URL signée.
- `PUT /{id}/links`, `DELETE /{id}/links/{resource}`.

## Outils MCP

`search_knowledge(query, tags, resource)`, `get_knowledge_article(id)` (renvoie le texte extrait, borné), `create_knowledge_note`, `tag_knowledge_articles`, `link_knowledge_article(article_id, resource_type, resource_id)`, `list_resource_knowledge(resource_type, resource_id)`.

Nouveau domaine et scope FerrisKey `mestier.knowledge.read` et `mestier.knowledge.write` : **à déclarer dans FerrisKey** avant la recette (même chantier ouvert que pour crm, quote, planning, reference).

## Webapp

Nouvelle page « Base de connaissances » : liste, recherche, filtres par étiquette, dépôt par glisser-déposer, éditeur de note, panneau « rattaché à ». Un bloc « Documents liés » sur la fiche projet, client et devis.

## Lots

- **K0** : migration, domaine, extraction de texte PDF, tests unitaires.
- **K1** : API et permission.
- **K2** : page webapp.
- **K3** : outils MCP et scopes.
- **K4** : blocs « Documents liés » sur projet, client, devis.

## Ce qui a été tranché en route

- Un crate `handlers-knowledge` à part, et non un module de `handlers-planning`.
- Le devis rattaché est un devis Mestier existant ; les anciens devis PDF sont des fichiers libres.
- Pas de partage entre organisations.
- Le dépôt d'un fichier est le corps brut de la requête, le titre et les étiquettes sont en paramètres. Le texte est lu hors de l'exécuteur async, avec un délai de 30 s. Un fichier dont le texte est illisible est gardé, pas refusé.
- La clé de stockage ne sort jamais de l'API : on lit un fichier par la route `download`.
- Les routes ouvertes au MCP : lister, lire, créer une note, modifier, rattacher. Pas de dépôt, de suppression ni de détachement (règle existante : aucune route déléguée ne supprime).
- Scope FerrisKey `mestier.knowledge.write` (les lectures passent par `mestier.read`) et règle Cerbos associée dans `deploy/cerbos/policies/mcp_tool.yaml`.

## Reste à faire

- Déclarer le scope `mestier.knowledge.write` dans FerrisKey et déployer la politique Cerbos.
- Recette dans un navigateur : dépôt par glisser-déposer, note, rattachement depuis la fiche projet, client et devis, droits en lecture seule.
- Recette avec un client MCP réel.
- Lire le texte des fichiers Word et Excel (acceptés au dépôt, pas lus), et l'OCR des PDF scannés.
