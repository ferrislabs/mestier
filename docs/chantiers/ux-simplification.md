# Chantier L — Simplifier l'usage de Mestier

Origine : audit UX global (5 sous-agents en lecture seule, un par module) demandé après que la boucle MVP (devis→planification→pointage→rentabilité) soit devenue fonctionnellement complète. 27 points de friction relevés ; 4 workstreams retenus par le product owner (HR/Équipe laissé de côté pour l'instant).

## Décomposition

| Workstream | Branche | Base | Statut | Fichiers possédés |
|---|---|---|---|---|
| WS1 Planning | `feature/planning-ux-friction` | `main` | intégré, poussé | `apps/webapp/src/pages/planning/**`, `apps/webapp/src/hooks/use-tasks.ts` |
| WS2 Rentabilité | `feature/reporting-ux-friction` | `feature/profitability-recollected-time` (stack) | intégré, poussé | `apps/webapp/src/pages/reporting/**` |
| WS3 App terrain | `feature/field-day-state-and-errors` | `feature/profitability-recollected-time` (stack) | intégré, poussé | `apps/webapp/src/pages/field/**`, `apps/webapp/src/hooks/use-field.ts`, `libs/handlers-field/src/field/current.rs`, `libs/handlers-field/src/response.rs`, `libs/core/src/application/time_entry/mod.rs`, `libs/core/src/domain/time_entry/service.rs` |
| WS4 CRM/Devis | `feature/crm-quotes-ux-friction` | `main` | intégré, poussé | `apps/webapp/src/pages/customers/**`, `apps/webapp/src/pages/quotes/**`, `apps/webapp/src/pages/catalog/**` |

WS2 et WS3 sont empilées sur `feature/profitability-recollected-time` (PR #du chantier M) : les deux modifient les mêmes fichiers déjà touchés par ce chantier (`reporting/types.ts`, `api.client.ts` généré) et ont été développées par-dessus. Leurs PR ciblent cette branche, pas `main`.

Aucun recoupement de fichiers entre workstreams → dispatch parallèle, pas de point de convergence orchestrateur nécessaire (aucun registre, aucune migration).
Seul WS3 touche le backend ; c'est le seul à régénérer `apps/webapp/src/api/api.client.ts`/`api.tanstack.ts` (généré, jamais édité à la main).

## Décisions produit gelées

- **Bulk-assign (WS1)** : passe en additif. Le backend `POST /tasks/bulk-assign` reste un remplacement complet (contrat inchangé) ; le frontend n'appelle plus cette route pour l'ajout, il fait une `PATCH /tasks/{id}` par tâche sélectionnée avec l'union des `member_ids` existants + la personne choisie.
- **Pipeline WON/LOST (WS4)** : deviennent des états terminaux, atteignables uniquement via une action explicite ("Marquer gagné" / "Marquer perdu"), retirés des flèches avancer/reculer séquentielles.
- **HT/TVA (WS4)** : pas de calcul de TVA (hors scope MVP, prévu en Phase 2 facturation) — seulement libeller explicitement "Total HT" partout où un total de devis brut est affiché aujourd'hui.
- **Contrat WS3** : nouvelle forme de réponse pour `GET /field/current` :
  ```rust
  pub struct FieldCurrentResponse {
      pub running: Option<TimeEntryResponse>,
      pub day_ended_at: Option<DateTime<Utc>>,
  }
  ```
  Source : `TimeEntryService::day_log_for_today(employee_id, now, timezone)` (nouvelle méthode passe-plat symétrique à `running_for`, réutilise le `local_date` privé déjà existant) + `resolve_timezone` (déjà utilisé par `profitability_report`). Le frontend dérive l'état "journée terminée" de ce champ serveur, plus jamais d'un state React local seul.

## Hors scope (noté, pas implémenté cette passe)

- WS1 : risque de doublon si l'affectation échoue après création de tâche · pickers client/devis plafonnés à 100 sans recherche · libellé menu "congé"/"absence"
- WS2 : titre de chantier non qualifié par le client · pas de préréglages de période · détail main-d'œuvre/matériel caché · `formatCents` dupliqué
- WS3 : confirmation/undo sur la clôture destructive · valeur par défaut du recover-time codée en dur
- WS4 : création de devis bloquée sans adresse client (correctif inline plus lourd, à faire dans un chantier dédié)
- WS5 (HR/Équipe) entier : pas retenu pour cette passe

## Journal de décisions

- Rejeté : forcer `bulk-assign` additif côté backend (changerait un contrat partagé par d'autres appelants potentiels) → additif fait côté frontend via des PATCH individuels.
- Rejeté : mapper les messages de conflit anglais → français par un nouveau champ de code d'erreur fin sur `CoreError` (changerait le contrat `ApiError`/`ErrorBody` partagé par tout le backend) → mapping par correspondance exacte des messages anglais connus, côté frontend uniquement, dans le scope de WS3.
