# Événements émis

Référence des 38 événements que Mestier émet, et de leur disponibilité dans le déclencheur « Événements » d'un workflow (`TriggerKind::Events`).

Source de vérité : le descripteur de chaque module (`libs/core/src/domain/<module>/events.rs`), assemblé par `event_catalogue()` dans `libs/core/src/domain/automation/catalogue.rs`. Tous sont en version 1.

## Les 38 événements

| Événement | Libellé | Sujet | Émis par |
|---|---|---|---|
| `quote.created` | Devis créé | `quote` | `domain/quote/service.rs:231`, `:542` |
| `quote.updated` | Devis modifié | `quote` | `domain/quote/service.rs:331` |
| `quote.sent` | Devis envoyé | `quote` | `domain/quote/service.rs:557` |
| `quote.accepted` | Devis accepté | `quote` | `domain/quote/service.rs:557` |
| `quote.declined` | Devis refusé | `quote` | `domain/quote/service.rs:557` |
| `quote.cancelled` | Devis annulé | `quote` | `domain/quote/service.rs:557` |
| `quote.deleted` | Devis supprimé | `quote` | `domain/quote/service.rs:576` |
| `invoice.created` | Facture créée | `invoice` | `domain/invoice/service.rs:330`, `:898`, `:1030`, `:1205` |
| `invoice.updated` | Facture modifiée | `invoice` | `domain/invoice/service.rs:405` |
| `invoice.issued` | Facture émise | `invoice` | `domain/invoice/service.rs:1533` |
| `invoice.partially_paid` | Facture partiellement payée | `invoice` | `domain/invoice/service.rs:1533` |
| `invoice.paid` | Facture payée | `invoice` | `domain/invoice/service.rs:1533` |
| `invoice.cancelled` | Facture annulée | `invoice` | `domain/invoice/service.rs:1533` |
| `invoice.deleted` | Facture supprimée | `invoice` | `domain/invoice/service.rs:486` |
| `invoice_payment.recorded` | Paiement enregistré | `invoice_payment` | `domain/invoice/service.rs:542` |
| `invoice_payment.deleted` | Paiement supprimé | `invoice_payment` | `domain/invoice/service.rs:569` |
| `supplier_invoice.received` | Facture fournisseur reçue | `supplier_invoice` | `domain/supplier_invoice/service.rs:86` |
| `supplier_invoice.confirmed` | Facture fournisseur confirmée | `supplier_invoice` | `domain/supplier_invoice/service.rs:401` |
| `supplier_invoice.rejected` | Facture fournisseur rejetée | `supplier_invoice` | `domain/supplier_invoice/service.rs:401` |
| `supplier_invoice_line_allocation.recorded` | Coût fournisseur attribué à un projet | `supplier_invoice_line_allocation` | `domain/supplier_invoice/service.rs:213`, `:357` |
| `ticket.created` | Ticket de support reçu | `ticket` | `domain/support/ticket_service.rs:92` |
| `ticket.transitioned` | Statut d'un ticket de support modifié | `ticket` | `domain/support/ticket_service.rs:140`, `thread_service.rs:277` |
| `maintenance_intervention.proposed` | Intervention d'entretien déclarée | `maintenance_intervention` | `domain/maintenance/visits.rs:165` |
| `maintenance_intervention.validated` | Intervention d'entretien validée | `maintenance_intervention` | `domain/maintenance/visits.rs:114`, `:182` |
| `maintenance_intervention.rejected` | Intervention d'entretien rejetée | `maintenance_intervention` | `domain/maintenance/visits.rs:205` |
| `time_entry.started` | Pointage démarré | `time_entry` | `domain/time_entry/service.rs:78` |
| `time_entry.stopped` | Pointage clôturé | `time_entry` | `domain/time_entry/service.rs:105`, `:147`, `:320` |
| `time_entry.declared` | Temps déclaré après coup | `time_entry` | `domain/time_entry/service.rs:212` |
| `day.ended` | Journée terminée | `day_log` | `domain/time_entry/service.rs:340` |
| `assignment_report.reported` | Écart déclaré | `assignment_report` | `domain/assignment_report/service.rs:93` |
| `assignment_report.applied` | Écart appliqué au planning | `assignment_report` | `domain/assignment_report/service.rs:227` |
| `assignment_report.dismissed` | Écart rejeté | `assignment_report` | `domain/assignment_report/service.rs:235` |
| `member.invited` | Invitation envoyée | `invitation` | `domain/invitation/service.rs:133` |
| `member.joined` | Membre arrivé | `member` | `domain/invitation/service.rs:285` |
| `member.removed` | Membre parti | `member` | `application/member/mod.rs:116` |
| `customer.created` | Client créé | `customer` | `domain/customer/service.rs` (`create_customer`) |
| `deal.created` | Opportunité créée | `deal` | `domain/deal/service.rs` (`create_deal`) |
| `deal.stage_changed` | Opportunité déplacée dans le pipeline | `deal` | `domain/deal/service.rs` (`move_deal`), `domain/customer/service.rs` (archivage d'un client) |

`deal.stage_changed` porte `from` et `to` (les noms d'étapes `NEW`, `VISIT`, `QUOTE_SENT`, `NEGOTIATION`, `WON`, `LOST`) et le deal dans son état final, dont `lost_reason`. Il part aussi pour chaque deal ouvert que l'archivage d'un client perd, et ne part pas quand le deal est déjà à l'étape demandée. Gagner un deal convertit un prospect en client sans émettre d'event : `deal.stage_changed` à `WON` est le signal.

Les transitions n'ont pas toutes un nom : un devis ou une facture qui repasse en `DRAFT`, un devis `SUPERSEDED`, une facture fournisseur `RECEIVED` n'émettent rien (`event_name` renvoie `None`).

## Disponibilité dans le nœud événement

Chaîne vérifiée, de l'émission au déclencheur :

1. **Descripteur** : chaque événement a un `EventDescriptor` (nom, version, libellé, sujet, exemple de payload) dans le module qui le possède.
2. **Catalogue** : `event_catalogue()` les assemble, et le test `the_catalogue_describes_exactly_what_the_modules_emit` échoue si un événement émis n'a pas de descripteur, ou l'inverse.
3. **API** : `GET /api/v1/organizations/{organization_id}/automation/events` renvoie tout le catalogue (`libs/handlers-automation/src/catalogue.rs`), sans filtre.
4. **Validation** : un workflow dont le déclencheur nomme un événement hors catalogue est refusé (`GraphError::UnknownEventName`, `domain/automation/workflow/validation.rs`).
5. **Abonnement** : à la publication, les noms du déclencheur sont écrits dans `automation.subscription` (`infrastructure/automation/postgres/subscription.rs`).
6. **Éditeur** : le sélecteur lit l'API. `apps/webapp/src/pages/automation/lib/event-search.ts` range les 38 noms par domaine, et un événement inconnu tomberait dans « Autres » plutôt que de disparaître. `event-contracts.ts` décrit les 38 payloads.

## Ce qui n'émet pas d'événement

Aucun événement n'existe pour : la modification d'un client, ses contacts, la modification ou la suppression d'un deal, projets, tâches et commentaires, produits, fournisseurs, dépenses, catégories de dépenses, contrats d'entretien (seules les interventions en ont), chat.

Ajouter un événement : un `EventDescriptor` dans le module, une ligne dans `emitted_events()`, le nom dans `EVENT_DOMAINS` du webapp et son contrat dans `event-contracts.ts`.
