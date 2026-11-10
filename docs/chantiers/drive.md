# Drive

État : **spec validée le 2026-11-10. D1 (dossiers) implémenté sur `feat/drive`, pas encore recetté dans un navigateur. D2 et D3 à faire.** Premier chantier après la base de connaissances (#668), qui sert de socle.

## Goal

Une entreprise range ses fichiers dans des dossiers, les ouvre dans la webapp sans les télécharger, les retrouve par leur contenu (PDF, Word, Excel), et reçoit des suggestions d'étiquettes d'une IA qu'elle choisit.

## Contexte

La suite prévue : **Drive**, puis **Tables** (façon Airtable), puis **Documents** par blocs (façon Notion). Les trois vivent dans le module « Connaissances », qui devient une section à plusieurs entrées : Drive maintenant, Tables et Documents plus tard.

Ce qui existe après #668 : articles `file` et `note` (`knowledge_articles`), étiquettes, recherche plein texte française, rattachement aux projets, clients et devis (`knowledge_links`), permissions `VIEW_KNOWLEDGE` et `MANAGE_KNOWLEDGE`, outils MCP. Le texte n'est lu que pour les PDF et le texte brut.

Briques réutilisées : le chiffrement des secrets de l'automatisation (`SecretCipher`), le modèle de worker par sondage et réservation (`infrastructure/automation/worker.rs`), `zip` et `quick-xml` déjà dans le workspace.

## Décisions (2026-11-10)

1. **Arborescence de dossiers.** Un fichier ou une note est dans un seul dossier, ou à la racine. Les étiquettes restent en plus.
2. **Pas de droits par dossier dans ce chantier.** Tout le Drive suit `VIEW_KNOWLEDGE` et `MANAGE_KNOWLEDGE`.
3. **La photothèque reste à part.**
4. **IA : suggestions à valider.** L'IA propose des étiquettes et un titre, l'utilisateur accepte ou refuse. Rien n'est modifié sans lui.
5. **IA derrière un port.** Le client branche son propre fournisseur pour la souveraineté et le coût. Un seul adaptateur au départ : l'API compatible OpenAI (`/chat/completions`), que parlent OpenAI, Mistral, Ollama, vLLM et LM Studio.

## Invariants

- Un dossier et son parent appartiennent à la même organisation. [test]
- Pas de cycle : déplacer un dossier sous l'un de ses descendants est refusé. [test]
- Profondeur maximale de 10 niveaux. [constructeur, `Result`]
- Deux dossiers frères n'ont pas le même nom, sans tenir compte de la casse. [index unique en base, test]
- Un article est dans au plus un dossier. [colonne `folder_id` nullable]
- On ne supprime qu'un dossier vide. [test]
- L'IA ne modifie jamais un article : elle écrit des suggestions, seule une action de l'utilisateur les applique. [test]
- Sans fournisseur configuré, aucun appel sortant. [test]
- La clé d'API de l'IA est chiffrée en base et ne sort jamais de l'API (ni réponse HTTP, ni MCP, ni log). [test]
- Le texte envoyé à l'IA est borné (titre + 8 000 premiers caractères du texte extrait). [test]
- Une adresse de fournisseur vers un réseau privé n'est acceptée que si l'instance l'autorise, comme les connecteurs HTTP de l'automatisation. [test]

## Hors périmètre

- Droits par dossier, partage public par lien, partage entre organisations.
- Versions d'un fichier, corbeille, quota de stockage.
- Aperçu des fichiers Word et Excel (ils sont téléchargés ; leur texte est lu pour la recherche).
- OCR des PDF scannés.
- Fusion avec la photothèque.
- Tables et Documents.

## Lots

### D1 : dossiers

- Table `knowledge_folders` : `id`, `org_id`, `parent_id` (nullable), `name`, `created_by`, `created_at`, `updated_at`. Index unique sur `(org_id, coalesce(parent_id, nil), lower(name))`.
- Colonne `knowledge_articles.folder_id` (nullable, même organisation).
- Cas d'usage : créer, renommer, déplacer, supprimer (vide seulement) un dossier ; déplacer un ou plusieurs articles ; lister le contenu d'un dossier (sous-dossiers + articles) avec son fil d'Ariane.
- La recherche texte reste globale, avec un filtre optionnel « dans ce dossier et ses sous-dossiers ».
- Webapp : arbre des dossiers à gauche, contenu à droite, fil d'Ariane, dépôt par glisser-déposer dans le dossier ouvert, déplacement par glisser-déposer ou par menu « Déplacer vers ».
- MCP : `list_knowledge_folder`, `create_knowledge_folder`, `move_knowledge_articles`. Pas de suppression (règle existante).
- Tranché en route : la webapp lit tout l'arbre d'un coup (`GET /knowledge/folder-tree`, 5 000 dossiers au plus) pour l'arbre, le fil d'Ariane et le sélecteur de destination ; le dossier ouvert est dans l'adresse (`?dossier=`). Une recherche porte sur le dossier ouvert et ses sous-dossiers. Le MCP ne renomme ni ne déplace un dossier.

### D2 : aperçu et lecture du texte Word et Excel

- Aperçu dans la webapp des PDF, des images et du texte, à partir de l'URL signée existante.
- Texte des `.docx` (paragraphes et tableaux) et des `.xlsx` (valeurs des cellules, feuille par feuille), avec `zip` et `quick-xml`. Mêmes bornes que les PDF : 500 000 caractères, taille décompressée plafonnée, délai de 30 s.
- Relecture une fois des fichiers Word et Excel déjà déposés (commande ponctuelle, pas au démarrage).

### D3 : suggestions par IA

- Port `TagSuggester` dans le domaine : reçoit le titre, le texte borné et les étiquettes déjà utilisées dans l'organisation ; rend au plus 5 étiquettes et un titre proposé.
- Adaptateur `OpenAiCompatible` (reqwest) : URL de base, modèle, clé. Réponse attendue en JSON, rejetée si invalide.
- Réglages par organisation (page Paramètres, permission `MANAGE_KNOWLEDGE`) : activé ou non, URL, modèle, clé, bouton « Tester ». Désactivé par défaut.
- File `knowledge_suggestions` : `article_id`, `status` (`pending`, `ready`, `failed`, `dismissed`, `applied`), `tags`, `title`, `error`, `claimed_at`, `attempts`. Un dépôt crée une entrée seulement si l'IA est activée. Un worker la traite par sondage, 3 essais au plus.
- Webapp : sur la fiche, « Suggestions de l'IA » avec chaque étiquette à accepter ou refuser, et le titre proposé. Un badge sur les articles qui ont des suggestions en attente.
- MCP : rien dans ce lot. L'assistant du client est déjà une IA, il étiquette par `update_knowledge_article`.

## Acceptance

D1
1. Créer un dossier, puis un sous-dossier : le fil d'Ariane du sous-dossier liste les deux.
2. Créer deux dossiers frères « Devis » et « devis » : le second est refusé.
3. Déplacer un dossier sous son propre sous-dossier : refusé, rien ne change.
4. Créer un 11e niveau : refusé.
5. Supprimer un dossier qui contient un fichier : refusé ; vide : accepté.
6. Déposer un fichier dans un dossier ouvert : il apparaît dans ce dossier et pas à la racine.
7. Déplacer trois articles dans un dossier en une action : les trois y sont.
8. Une organisation ne voit ni ne peut viser les dossiers d'une autre (lecture, déplacement d'article, parent).
9. Rechercher un mot avec le filtre de dossier : seuls les articles de ce dossier et de ses sous-dossiers remontent.
10. MCP : l'assistant liste un dossier, en crée un et y déplace des articles ; il ne peut pas supprimer.

D2
11. Un `.docx` contenant « béton désactivé » est retrouvé par la recherche « béton ».
12. Un `.xlsx` contenant « Paulownia » dans une cellule est retrouvé par « paulownia ».
13. Un `.docx` piégé (bombe zip) est gardé sans texte, sans faire tomber l'API.
14. Un PDF et une image s'ouvrent en aperçu dans la webapp ; un `.docx` propose le téléchargement.
15. La relecture ponctuelle remplit le texte des `.docx` et `.xlsx` déjà déposés.

D3
16. IA désactivée : un dépôt ne crée aucune suggestion et aucun appel sortant.
17. IA activée (fournisseur simulé) : après un dépôt, la fiche affiche les étiquettes et le titre proposés ; l'article n'a pas changé.
18. Accepter deux étiquettes sur trois : l'article a exactement ces deux étiquettes en plus.
19. Fournisseur en erreur : 3 essais, puis `failed` avec le message ; l'article reste intact.
20. La clé n'apparaît dans aucune réponse HTTP ni aucun log ; la page réglages affiche « clé enregistrée ».
21. Le texte envoyé au fournisseur ne dépasse pas la borne.
22. Une URL de fournisseur vers `localhost` ou un réseau privé est refusée, sauf si l'instance l'autorise.

## Verify (exit)

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
SQLX_OFFLINE=true cargo test --workspace
cargo test -p mestier-core -- --ignored knowledge
cd apps/webapp && pnpm check && pnpm test
```

Plus une recette navigateur par lot, et pour D3 une recette avec un vrai fournisseur (Mistral ou Ollama local).

## Risques et questions ouvertes

- **Nom du module dans le menu.** Proposition : « Connaissances » devient une section avec l'entrée « Drive ». À confirmer par l'utilisateur à la recette.
- **Coût de l'IA** : à la charge du client, sur sa clé. Aucune clé fournie par Mestier dans ce chantier.
- **Données envoyées à un tiers** : seulement si le client active l'IA et choisit son fournisseur. À écrire dans l'écran de réglage.

Amended: 2026-11-10 ajout de la route `folder-tree` pour l'arbre de la webapp.
