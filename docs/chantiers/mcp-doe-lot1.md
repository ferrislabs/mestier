# MCP DOE, lot 1 : fichiers, fournitures, sections

État : **validée le 2026-10-10 ; L1-A et L1-B faits, L1-C en cours**.

## Goal

Par MCP, l'assistant gère tout le contenu d'un DOE qui existe déjà dans le modèle : il modifie et supprime des fournitures, organise les sections, et ajoute, renomme ou retire des photos, des plans et les pièces d'une fourniture (photos, fiches techniques), fichiers compris.

## Context

- Demande du 2026-10-10 (évolution du MCP DOE, 5 phases). Ce lot couvre les phases 1 et 2, et la partie fichiers de la phase 3. Lot 2 : documents typés, notes, plans enrichis. Lot 3 : contrôle, complétude, duplication. Lot 4 : versionnement, ZIP, aperçu, index.
- Décisions du 2026-10-10 : l'assistant peut supprimer **dans** un DOE (fourniture, fichier, section vide), jamais un DOE entier ; les fichiers arrivent par le paramètre fichier de ChatGPT, ou en base64 plafonné pour les autres clients, jamais par une URL libre.
- Spec MCP v1 : `docs/chantiers/mcp.md`. Ce lot amende son invariant 7 (plus de « aucune suppression » au sein d'un DOE).
- Faits vérifiés :
  - ChatGPT transmet un fichier à un outil qui déclare `_meta["openai/fileParams"]` : l'argument reçoit `{download_url, file_id, mime_type?, file_name?}`, et `download_url` est une URL temporaire. Bug connu sur l'app mobile : références incomplètes.
  - L'API reçoit un fichier brut sur `POST /api/v1/files?folder=<slug>` (`Content-Type` = type du fichier, 10 Mo par défaut), sans contrôle de type ni d'organisation.
  - Le modèle a déjà tout ce qu'il faut : sections `photos` et `plans` (`DoeFile {key, caption}`), fournitures avec `photos` et `datasheet_keys`, limites de 30 sections et 300 fichiers par DOE.

## Outils (9 de plus, 26 au total)

Moins d'outils qu'en demande : `get_doe` renvoie déjà tout le contenu (fournitures, fichiers et leurs clés), donc pas de `list_*` ni `get_*` séparés. Les `update_*` ne changent que les champs donnés.

| Outil | Rôle | Remplace dans la demande |
|---|---|---|
| `update_doe_supply` | Modifier une fourniture (titre, désignation, DPGF, destination, fournisseur, fabricant, caractéristiques) | `update_doe_supply` |
| `delete_doe_supply` | Retirer une fourniture du DOE | `delete_doe_supply` |
| `add_doe_section` | Ajouter une section (fournitures, photos, plans, pages libres), à une position donnée | `add_doe_section` |
| `update_doe_section` | Renommer, activer/désactiver, régler (pépinière/fabricant, photos par page, nombre de pages) | `update_doe_section` |
| `delete_doe_section` | Supprimer une section **vide** | `delete_doe_section` |
| `reorder_doe_sections` | Réordonner toutes les sections | `reorder_doe_sections` |
| `add_doe_file` | Ajouter un fichier : photo de chantier, plan, photo d'une fourniture ou fiche technique d'une fourniture | `add_doe_photo`, `add_doe_plan`, `add_doe_supply_photo`, `add_doe_supply_document` |
| `update_doe_file` | Changer la légende d'un fichier, ou déplacer une photo/un plan vers une autre section du même genre | `update_doe_photo`, `update_doe_plan` |
| `delete_doe_file` | Retirer un fichier du DOE | `delete_doe_photo`, `delete_doe_plan`, `delete_doe_supply_photo`, `delete_doe_supply_document` |

`list_doe_photos`, `get_doe_photo`, `list_doe_supplies`, `get_doe_supply`, `list_doe_plans`, `get_doe_section` : couverts par `get_doe`.

### `add_doe_file`

- `target` : `photo` (section photos), `plan` (section plans), `supply_photo`, `supply_datasheet` ; `section_id` ou `section_title` pour les deux premiers, `supply_id` pour les deux autres ; `caption` facultative.
- Le fichier, au choix :
  - `file` : paramètre fichier ChatGPT (`_meta["openai/fileParams"] = ["file"]`), objet `{download_url, file_id, mime_type?, file_name?}` ;
  - `content_base64` + `file_name` + `mime_type`, pour les clients sans paramètre fichier.
- Le serveur MCP récupère les octets, vérifie le type, dépose le fichier par `POST /api/v1/files?folder=doe`, puis ajoute la clé au DOE.

## Invariants

1. Le serveur MCP ne télécharge un `download_url` que s'il est en `https`, sur un hôte de la liste autorisée (`MCP_SERVER_FILE_HOSTS`, hôtes de fichiers d'OpenAI par défaut), sans suivre de redirection, en 15 s au plus, et dans la limite de taille. [test]
2. Taille : 10 Mo par fichier (aligné sur l'API), 5 Mo en base64. Au-delà, erreur avant tout dépôt. [test]
3. Le type vient des **octets** (signature JPEG, PNG, WebP, PDF), pas du nom ni du type déclaré. Photos : JPEG, PNG, WebP. Plans et fiches techniques : les mêmes plus PDF. Tout autre type est refusé avant dépôt. [test]
4. Aucun outil ne supprime un DOE ; la liste blanche des routes déléguées n'ouvre aucun `DELETE`. [type : catalogue fermé ; test : liste blanche]
5. `delete_doe_section` refuse une section qui contient quelque chose (même règle que l'éditeur web). [test]
6. `reorder_doe_sections` exige exactement les ids des sections du DOE, chacun une fois. [test]
7. **Écriture concurrente** (contrôle dans le service, dans la transaction qui relit le DOE ; une fenêtre de quelques millisecondes reste entre relecture et écriture, acceptée) : toute écriture de l'assistant relit le DOE, le modifie et le réécrit en entier. Pour ne pas écraser une modification faite entre-temps dans l'éditeur web, `PUT /does/{id}` accepte `expected_updated_at` : s'il ne correspond plus, l'API répond 409 et l'outil relit puis réessaie une fois, sinon renvoie l'erreur. [test : API + serveur MCP]
8. Toutes les écritures passent par l'API existante : ses contrôles s'appliquent (`planning.manage`, 30 sections, 300 fichiers, `mcp_enabled`). [test e2e]

## Out of scope

- Suppression de l'objet dans le stockage quand un fichier est retiré d'un DOE (il reste stocké ; nettoyage à part, avec la table de propriété des fichiers).
- Documents typés (notices, garanties, PV), notes rédigées, plans enrichis : lot 2.
- Envoi de fichiers depuis l'app mobile ChatGPT (bug côté OpenAI).

## Design

- `libs/mcp-tools` : 9 variantes de `Tool`, toutes `Write`, domaine `doe`.
- `apps/mcp-server` : les 9 outils ; module `files` (téléchargement contrôlé, base64, détection du type par signature, dépôt) ; relecture + `expected_updated_at` + une nouvelle tentative sur 409 pour toutes les écritures DOE, y compris les 3 existantes.
- API (`libs/core` doe + `handlers-planning`) : `UpdateDoeRequest.expected_updated_at: Option<DateTime<Utc>>`, refus 409 « le DOE a été modifié entre-temps » en cas d'écart. Le webapp l'envoie aussi (même protection pour lui).
- `libs/handlers/src/delegated.rs` : ouvrir `POST /api/v1/files` aux tokens délégués.
- Cerbos : rien à changer (écritures DOE avec `mestier.doe.write`).

## Acceptance

1. `update_doe_supply` ne change que les champs donnés ; `supplier_name` vide retire le fournisseur.
2. `delete_doe_supply` retire la fourniture ; un `supply_id` inconnu donne une erreur sans écriture.
3. `add_doe_section` ajoute une section du bon genre à la position demandée ; au-delà de 30 sections, l'erreur de l'API remonte.
4. `delete_doe_section` refuse une section non vide.
5. `reorder_doe_sections` refuse une liste incomplète ou en double.
6. `add_doe_file` avec un paramètre fichier ChatGPT (serveur de test sur un hôte autorisé) dépose le fichier et l'ajoute à la section photos avec sa légende ; le DOE relu contient la clé.
7. `add_doe_file` refuse : hôte non autorisé, `http`, redirection, fichier trop gros, type non autorisé (un `.jpg` qui contient du HTML), photo au format PDF.
8. `add_doe_file` en base64 marche pour une fiche technique PDF de fourniture.
9. `update_doe_file` change une légende ; `delete_doe_file` retire la clé.
10. Écriture concurrente : un `PUT` avec un `expected_updated_at` périmé reçoit 409 ; l'outil relit et réussit à la seconde tentative.
11. E2E par la gateway : un token délégué peut `POST /api/v1/files` ; toujours aucun `DELETE` ouvert.
12. Recette ChatGPT : « Ajoute cette photo dans Photos générales du chantier avec la légende … » avec une photo jointe dans le web ; la photo apparaît dans l'éditeur et dans le PDF.

## Workstreams

| Id | Mission | Qui |
|---|---|---|
| L1-A | API : `expected_updated_at` sur le `PUT` du DOE (core, handler, webapp qui l'envoie), ouverture de `POST /api/v1/files` aux tokens délégués | moi |
| L1-B | Catalogue `mcp-tools` (9 outils) | moi, avant L1-C |
| L1-C | Serveur MCP : 9 outils, module fichiers, relecture + nouvelle tentative sur 409 pour toutes les écritures | sous-agent |
| L1-D | E2E par la gateway, puis recette ChatGPT avec une vraie photo | moi + toi |

L1-A et L1-B d'abord (contrats), puis L1-C. Une seule branche, `feat/mcp`, et une PR.

## Risks

| Risque | Parade |
|---|---|
| Les hôtes exacts des `download_url` d'OpenAI ne sont pas documentés | Liste configurable ; le serveur journalise l'hôte refusé ; on la règle pendant la recette L1-D |
| Le LLM appelle `add_doe_file` sans joindre de fichier | Erreur explicite « joins le fichier à ta demande » |
| Un DOE à 300 fichiers alourdit chaque relecture | Accepté : un DOE fait quelques dizaines de fichiers |

## Verify (exit)

```bash
cargo fmt --all --check
SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p handlers-planning --test http_e2e -- --ignored
python3 apps/mcp-gateway/e2e/local_stack.py
cd apps/webapp && pnpm check && pnpm test && pnpm build
```
