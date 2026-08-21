# ADR 0002 : le coût vient du planning, plus du pointage

- Status: Accepted
- Date: 2026-08-21
- Related issue: #260
- Supersedes: la formule de coût de [ADR 0001](0001-mvp-data-model.md)

## Contexte

L'ADR 0001 fixait la rentabilité sur le pointage : `coût = Σ(temps pointé ×
€/h)`. Le code livré applique cette règle à la lettre. `ProfitabilityFacts` ne
contient que des `ClockedTime`, et l'adapter ne fait entrer un chantier dans le
rapport que s'il existe une ligne `time_entries` dans la fenêtre :

```sql
AND EXISTS (SELECT 1 FROM time_entries te ... )
AND r.customer_id IS NOT NULL
```

Deux conséquences, toutes deux vérifiées sur le code :

1. Une tâche planifiée, assignée et faite, mais jamais pointée, coûte **zéro**.
2. Un travail sans client n'apparaît nulle part. Une réunion de 2 h à trois
   personnes coûte six heures-personne et n'est comptabilisée sur rien. Une
   tâche « envoyer le rapport à Brigitte, 30 min » non plus.

Sur le terrain, personne ne pointe. Un responsable — ou un workflow, ou un
agent — pose la tâche sur le planning : Bernard, mission à Clermont, 9h-12h,
déplacement inclus. Bernard ne s'enregistre pas. C'est un salarié, la tâche est
son temps, son temps est un coût. S'il a mis moins de temps que prévu, c'est à
lui de le dire et le plan est corrigé. Cette correction est un acte de
management, pas un pointage manquant.

## Décision

**La tâche planifiée est la source du coût.** `time_entries` et l'application
terrain restent en base et continuent de fonctionner, mais plus aucun calcul
monétaire ne les lit.

**`projects` devient une entité.** Un « chantier » n'en était pas une : c'était
une tâche racine portant un `customer_id`, et la rentabilité regroupait sur
`COALESCE(parent_task_id, id)`, ce qui ne marche que grâce au plafond de deux
niveaux de la hiérarchie. Un projet existe avec ou sans client, et n'importe
quelle tâche s'y rattache quelle que soit sa profondeur. C'est ce qui rend un
centre de coût interne exprimable.

**Le client et le devis remontent sur le projet.** La marge est le coût du
projet face au total du devis : il faut un dénominateur qui couvre le sujet
entier.

**Les frais sont un montant libre sur la tâche**, avec un libellé obligatoire
dès que le montant est non nul.

**Les tâches « journée entière » sont coûtées sur les créneaux de travail du
membre.** `expand_work_slots` rend une liste d'intervalles et non une
amplitude, donc une journée 9h-12h / 13h-17h coûte 420 minutes et la pause de
midi n'est jamais facturée.

**Le double comptage est exposé, pas réparé en silence.** Deux tâches qui se
chevauchent pour la même personne facturent deux fois la même heure, et
`detect_conflicts` avertit sans jamais refuser. Le rapport porte un champ
`overlapping_minutes` à côté de `employees_without_rate` déjà existant.

## Alternatives rejetées

- **Planifié et réel côte à côte.** Deux colonnes, prévu et pointé, avec
  l'écart. Rejeté : ça double le modèle et l'interface pour exprimer une
  correction qu'un responsable peut simplement faire sur la tâche.
- **Garder `quote_id` sur la tâche racine.** Rejeté : un projet à deux tâches
  racines n'aurait aucun devis unique auquel se comparer.
- **Barème kilométrique pour les frais.** Plus juste, mais suppose que
  quelqu'un saisisse des distances. Reporté, pas écarté.
- **7 heures en dur pour une journée entière.** Faux pour tout contrat qui
  n'est pas un temps plein standard.
- **Dédupliquer les chevauchements automatiquement.** Rejeté : le coût
  cesserait d'être vérifiable à la main, et ce module a pour principe
  d'afficher l'incomplétude plutôt que de la masquer.

## Conséquences

- `GET /reporting/worked-hours` ne fait aucune requête propre : il réutilise le
  read de la rentabilité. Les heures de paie deviennent donc des heures
  planifiées. Laisser cet écran sur `time_entries` afficherait une page vide
  puisque plus personne ne pointe.
- `open_entries`, `recollected_minutes` et `closed_after_the_fact` quittent le
  module rentabilité. Un effet de bord favorable : le pointage jamais clôturé
  était la première cause de marge masquée, et il n'existe plus. Seul un taux
  horaire réellement absent bloque encore une marge.
- `tasks.customer_id` et `tasks.quote_id` survivent à la migration `projects`.
  Les supprimer est un suivi, une fois que plus aucun lecteur ne les utilise.
- La rentabilité devient sensible à la qualité du planning. C'est le but : le
  planning était déjà la source de vérité de qui fait quoi, il devient la
  source de vérité de ce que ça coûte.
