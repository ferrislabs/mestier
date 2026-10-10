# ADR 0004 : une gateway MCP fait PEP et échange le token

- Status: Proposed
- Date: 2026-10-09
- Related: [chantier MCP](../chantiers/mcp.md)

## Contexte

Les clients de Mestier veulent utiliser l'IA de leur choix (ChatGPT web d'abord). Le protocole commun est MCP. Le client LLM obtient un token OAuth pour le serveur MCP ; la spec MCP interdit de repasser ce token à une API en aval, et l'API Mestier doit voir l'utilisateur, pas un compte technique.

## Décision

Trois services, deux dans le dépôt Mestier :

- **Gateway** (`apps/mcp-gateway`, Rust) : seule exposée. Ressource OAuth (RFC 9728, 401 + `WWW-Authenticate`), valide le token client, identifie l'outil, interroge le PDP en AuthZEN 1.0, puis échange le token client contre un token délégué (RFC 8693, audience = API Mestier, `act` = la gateway) et transmet au serveur MCP.
- **Serveur MCP** (`apps/mcp-server`, Rust, `rmcp`) : privé, expose les outils, appelle l'API Mestier avec le token délégué.
- **PDP** externe en AuthZEN : Cerbos d'abord (ADR 0005), FerrisKey à terme.

FerrisKey reste le seul serveur d'autorisation : il gagne CIMD et DCR pour que l'URL suffise au client LLM. L'API Mestier vérifie l'audience des tokens, sans quoi un token client contournerait la gateway.

## Alternatives rejetées

- **Le serveur MCP fait tout** (ressource OAuth, PEP, échange) : pas de point d'application unique, et la politique se disperserait dans le code des outils.
- **La gateway imite un serveur d'autorisation devant FerrisKey** pour offrir DCR sans toucher FerrisKey : deuxième IAM à maintenir, et deux émetteurs de tokens pour un même utilisateur.
- **Compte de service pour appeler l'API** : l'API ne verrait plus l'utilisateur, ses droits et son journal deviendraient faux.

## Conséquences

- Deux appels de plus par requête (PDP, échange) : l'échange est mis en cache jusqu'à expiration par couple (utilisateur, client LLM).
- La gateway ne connaît rien du métier de Mestier : elle pourra servir à d'autres produits ferrislabs et sortir du dépôt plus tard.
- Activer l'audience côté API demande de vérifier les tokens du webapp avant.
