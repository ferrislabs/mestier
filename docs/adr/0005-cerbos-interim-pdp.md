# ADR 0005 : Cerbos comme PDP provisoire

- Status: Proposed
- Date: 2026-10-09
- Related: [ADR 0004](0004-mcp-gateway-pep-token-exchange.md)

## Contexte

La gateway MCP a besoin d'un PDP dès la v1. À terme ce sera FerrisKey ; d'ici là, il faut un moteur simple, et changer de PDP ne doit pas toucher la gateway.

## Décision

Cerbos, lancé en conteneur, interrogé par la gateway via l'API **AuthZEN 1.0** que Cerbos implémente depuis la 0.48. Les politiques vivent dans `deploy/cerbos/policies/` avec leurs tests.

La politique v1 décide sur l'outil, son type d'accès (lecture, écriture) et les scopes OAuth. Elle ne recopie pas les rôles Mestier : l'API reste juge des droits métier.

## Alternatives rejetées

- **OPA** : Rego à apprendre, et pas d'API AuthZEN native (il faut Topaz ou une couche maison).
- **Le moteur local de `libs/authz`** : il vit dans le processus de l'API ; la gateway aurait besoin de son propre exemplaire et la séparation PEP/PDP disparaîtrait.
- **OpenFGA** : orienté relations, inutile tant que la décision ne porte que sur l'outil et les scopes.

## Conséquences

- Remplacer Cerbos par FerrisKey se fera en changeant l'URL AuthZEN de la gateway et en portant les politiques.
- Un service de plus à faire tourner. La gateway refuse tout quand il ne répond pas.
