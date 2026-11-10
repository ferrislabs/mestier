# News : annonces produit

État : **spec, 2026-11-10** (branche `feat/news`).

## Goal

L'équipe Mestier publie des news (nouveautés, annonces) depuis un back-office ; chaque utilisateur les voit une fois, dans une fenêtre en carrousel à l'ouverture de l'app, et peut les relire depuis son menu.

## Context

- Demande du 2026-11-10, avec une référence de design : la fenêtre « What's new » de Mistral Studio (image d'en-tête, titre, liste de points icône + titre + phrase, bouton en pied).
- Les rôles du royaume FerrisKey arrivent dans le jeton et dans le `Subject` (`iam.roles`). `LocalPolicyEngine` laisse passer `mestier:admin` sur toute action ; une action absente de `default_authorizer` est refusée à tous les autres.
- Il n'existe ni back-office ni données au-dessus des organisations.
- Réponses du 2026-11-10 : news pour tous les utilisateurs de Mestier ; back-office ouvert par `mestier:admin` pour l'instant ; contenu structuré comme l'exemple ; vue dès qu'affichée ; brouillon / publiée avec date de publication facultative, sans ciblage ; entrée « Nouveautés » dans le menu utilisateur.

## Décisions

1. **Données de plateforme, pas d'organisation** : `news` et `news_views` n'ont pas d'`org_id`. Exception voulue à la règle « toute requête est bornée par `org_id` » : une news est la même pour tous. `news_views` est borné par `user_id`.
2. **Contenu structuré** : `title` (1 à 120 caractères), `image_key` facultatif (stockage de fichiers, même envoi que les autres images), `items` (1 à 6 : `icon`, `title` 1 à 60, `text` 1 à 240), `cta` facultatif (`label` 1 à 40, `url` en `https://` ou `http://`). Icônes : une liste fermée (`NewsIcon`), traduite en icônes lucide côté webapp.
3. **Publication portée par une date** : `published_at: Option<DateTime>`. Absente = brouillon ; passée = publiée ; future = programmée. Pas de champ d'état à part.
4. **Back-office réservé à `mestier:admin`, lu à part** (amendé le 2026-11-10, réponse a) : Mestier ne lisait aucun rôle du jeton. `Identity` porte désormais les rôles du royaume dans un champ à part (`realm_roles`), jamais donné au moteur de droits : le passe-droit `mestier:admin` de `LocalPolicyEngine` reste inactif. Le handler en tire un `NewsEditor` (preuve de type) que chaque use case du back-office exige. Un jeton délégué (assistant MCP) ou un client n'a aucun rôle du royaume. Remplacer par un rôle dédié plus tard = changer `NewsEditor::ROLE`.
5. **Vue = affichée** : le webapp envoie `POST /news/seen` avec l'id d'une news dès qu'elle est à l'écran dans le carrousel. Idempotent. La vue est retenue côté serveur, par utilisateur, tous appareils confondus.
6. **Ordre** : la plus récemment publiée d'abord.
7. **Webapp** : la fenêtre s'ouvre après connexion s'il reste des news non vues ; « Nouveautés » dans le menu utilisateur la rouvre sur toutes les news publiées ; l'entrée « Back-office news » n'apparaît qu'à qui peut gérer.

## Invariants

1. Une news en brouillon ou programmée n'est jamais servie aux utilisateurs. [test]
2. Une news vue n'est plus proposée à cet utilisateur, et reste proposée aux autres. [test]
3. Un contenu invalide (titre vide, 0 ou 7 points, URL `javascript:`) est refusé à l'enregistrement. [type : constructeurs]
4. Seul `mestier:admin`, sur son propre jeton, lit, crée, modifie, publie et supprime dans le back-office ; ce rôle ne donne rien d'autre. [type : `NewsEditor` ; test : `Identity::roles` reste vide]
5. Supprimer une news supprime ses vues. [migration : `ON DELETE CASCADE`]
6. Migration réversible. [test : up, down, up]

## Out of scope

- Ciblage (par organisation, module, rôle), langues autres que le français.
- Rôle FerrisKey dédié (`mestier:news`) : plus tard.
- Statistiques de lecture dans le back-office.
- Notification hors app (e-mail, push).
- Texte riche dans les points.

## Design

- **Migration** `20261118000001_create_news` : `news (id, title, image_key, items JSONB, cta JSONB NULL, published_at NULL, created_at, updated_at)`, index sur `published_at` ; `news_views (news_id → news ON DELETE CASCADE, user_id → users ON DELETE CASCADE, seen_at, PRIMARY KEY (news_id, user_id))`.
- **Core** `domain/news` : `News`, `NewsId`, `NewsTitle`, `NewsItem`, `NewsIcon`, `NewsCta` (URL validée), `NewsContent` (points bornés) ; port `NewsRepository` (`insert`, `update`, `delete`, `find_by_id`, `list_all`, `list_published(now)`, `list_unseen(user, now)`, `mark_seen(user, ids)`). `application/news` : use cases `list_news_for_admin`, `create_news`, `update_news`, `delete_news` (chacun exige un `NewsEditor`), `list_published_news`, `list_unseen_news`, `mark_news_seen`.
- **Infrastructure** `infrastructure/news/postgres` + marqueur dans `registry.rs`.
- **API** crate `handlers-platform` (zone : ce qui est au-dessus des organisations), tag `news` :
  - utilisateur : `GET /api/v1/news` (publiées), `GET /api/v1/news/unseen`, `POST /api/v1/news/seen` `{ ids }`, `GET /api/v1/news/access` → `{ can_manage }` ;
  - back-office : `GET /api/v1/admin/news`, `POST /api/v1/admin/news`, `PUT /api/v1/admin/news/{id}`, `DELETE /api/v1/admin/news/{id}`.
- **Webapp** :
  - `hooks/use-news.ts` ;
  - `pages/news/ui/news-dialog.tsx` : la fenêtre de l'exemple (en-tête image ou dégradé, titre, points, bouton), carrousel (flèches, points de position, ← →), pure UI ;
  - `pages/news/feature/news-gate-feature.tsx` monté dans `_app.tsx` : ouvre la fenêtre sur les non vues, marque vue chaque news affichée ;
  - menu utilisateur : « Nouveautés » et, avec `can_manage`, « Back-office news » ;
  - `pages/news-admin/` + route `/admin/news` : liste (titre, état brouillon / programmée / publiée, date), éditeur (titre, image, points avec choix d'icône, bouton), aperçu en direct avec le même `news-dialog`, publier maintenant, programmer, repasser en brouillon, supprimer.

## Acceptance

1. Un admin crée une news en brouillon : `GET /news/unseen` ne la renvoie à personne.
2. Il la publie : un utilisateur la reçoit dans `/news/unseen`, puis ne la reçoit plus après `POST /news/seen` ; un autre utilisateur la reçoit toujours.
3. Une news programmée demain n'est ni dans `/news` ni dans `/news/unseen`.
4. Un utilisateur sans `mestier:admin` : 403 sur chaque route `/admin/news`, `can_manage = false`.
5. Contenu invalide (titre vide, 7 points, URL `javascript:alert(1)`) : 422, rien d'enregistré.
6. Supprimer une news : elle disparaît de `/news`, ses vues aussi.
7. Webapp : à la connexion, la fenêtre montre 2 news non vues en carrousel ; on la ferme ; au rechargement elle ne revient pas ; « Nouveautés » la rouvre.
8. Back-office : créer, voir l'aperçu, publier ; la news apparaît chez un autre utilisateur.

## Workstreams

| Id | Mission | Qui |
|---|---|---|
| N-A | Migration, domaine, dépôt, use cases, `handlers-platform`, OpenAPI, tests | moi |
| N-B | Webapp : fenêtre carrousel, garde, menu, back-office | sous-agent, après `pnpm gen:api` |

Une branche `feat/news`, une PR.

## Risks

| Risque | Parade |
|---|---|
| La fenêtre gêne à chaque connexion | Une news vue ne revient jamais ; aucune requête si rien de nouveau |
| Une URL de bouton malveillante | Seuls `http(s)://` acceptés, lien ouvert avec `rel="noopener noreferrer"` |
| `mestier:admin` donne bien plus que les news | Assumé pour l'instant (réponse du 2026-11-10), rôle dédié hors périmètre |

## Écarts constatés à l'implémentation

- **Les rôles du jeton n'étaient pas lus** (`Identity::from(Claims)` mettait `roles: Vec::new()`) : corrigé par la décision 4 amendée, sans réveiller le passe-droit global.
- `GET /news/access` renvoie `{ "data": { "can_manage": bool } }` : enveloppé comme toute réponse JSON, pas `{ "can_manage": bool }` nu.
- Le contrôle du rôle passe avant la validation du contenu : un non-admin qui envoie un contenu invalide reçoit 403, pas 422.
- `NewsTitle` n'existe pas comme type à part : le titre est validé dans `NewsContent::try_from(NewsContentInput)`, avec `NewsItem::new` et `NewsCta::new` pour les points et le bouton.
- La crate `url` est ajoutée aux dépendances de `mestier-core` pour valider le lien du bouton.
- Webapp : l'éditeur du back-office utilise un état contrôlé et la validation pure de `lib/`, partagée avec l'aperçu, pas TanStack Form (même choix que l'éditeur de style DOE).
- Webapp : sans bouton, le pied de la fenêtre affiche « Compris », qui la ferme.
- Webapp : « Publier maintenant » sur une news déjà publiée garde sa date de publication ; une date programmée doit être dans le futur.
- Webapp : un `seen` en échec n'est pas renvoyé pendant la session ; la news sera reproposée à la prochaine ouverture.

## Verify (exit)

```bash
cargo fmt --all --check
SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p mestier-core -- --ignored
cargo test -p handlers-platform --test http_e2e -- --ignored
cd apps/webapp && pnpm check && pnpm test && pnpm build
```
