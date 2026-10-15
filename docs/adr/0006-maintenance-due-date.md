# ADR 0006 : l'échéance d'entretien est calculée depuis la dernière intervention validée

- Status: Proposed
- Date: 2026-10-11
- Related: [chantier Entretien](../chantiers/entretien.md)

## Contexte

Le conducteur de travaux de l'entretien doit savoir quand repasser sur chaque site. La règle métier est glissante : « on a tondu le 10, on repasse le 24 ». Une intervention faite en retard décale toutes les suivantes.

Mestier a déjà `task_recurrence`. Ses dates sont fixées sur le calendrier à partir de `starts_on` (`expand_occurrences`), matérialisées en tâches jusqu'à un horizon de 60 jours, et le service ne lit jamais le statut d'une tâche. Il ne connaît que `Daily`, `Weekly` et `Monthly`, sans intervalle.

## Décision

Un nouveau contexte `maintenance` porte le contrat, ses prestations et leurs interventions. **L'échéance n'est pas stockée.** Une fonction pure la calcule depuis la cadence, la saison, les interventions validées, les tâches planifiées et une date `today` passée en paramètre.

Les tâches ne sont créées que quand le conducteur planifie, depuis le récap. Elles portent `maintenance_line_id` et appartiennent au projet du contrat, ce qui donne la rentabilité par l'ADR 0002.

## Alternatives rejetées

- **Étendre `task_recurrence`** avec un intervalle et un ancrage « depuis la dernière tâche faite ». Il faudrait alors dématérialiser et rematérialiser les occurrences à chaque validation, et lire le statut des tâches dans un service qui n'en dépend pas. On mélangerait aussi deux règles opposées (calendrier fixe et glissement) dans un même type.
- **Stocker la prochaine échéance** sur la prestation et la mettre à jour à chaque validation. C'est plus simple à requêter, mais il faut la recalculer à chaque rejet, suppression, correction de date ou changement de cadence, et l'oubli d'un seul chemin la rend fausse sans bruit. Le volume (quelques centaines de prestations par organisation) rend le calcul à la lecture gratuit.

## Conséquences

- La grille et le récap appellent la même fonction : ils ne divergent jamais.
- Le temps est un paramètre : les bascules de semaine et d'année se testent sans horloge.
- Trier ou filtrer par échéance en SQL est impossible. Le filtrage se fait en mémoire après chargement de la saison. Si une organisation dépasse quelques milliers de prestations, on ajoutera une projection, par un nouvel ADR.
