# 0011. Deals are their own aggregate, not a stage on the customer

Status: proposed
Date: 2026-11-07

## Context
The pipeline is carried by `customers.pipeline_stage`: one stage per customer. The rework prototype tracks opportunities (title, amount, expected close date, owner, source, lost reason) and a customer can have several at once. Quotes need to point at the opportunity they answer. A won opportunity converts a prospect into a customer.

## Decision
Add a `deal` aggregate scoped by `org_id`, with six stages (new, visit, quote_sent, negotiation, won, lost), a lost reason required to enter `lost`, and a nullable `deal_id` on quotes. A migration creates one deal per customer whose `pipeline_stage` is set, mapping the old stage onto the new ones. The API stops exposing `pipeline_stage`; the column stays in the table until a later migration removes it.

## Alternatives rejected
- Keep the stage on the customer and add the opportunity fields to it: one opportunity per customer, no history of lost deals, and the lost reason has nowhere to live.
- Drop `pipeline_stage` in the same migration: irreversible data loss if the down migration runs after deals were edited, and the automation connectors and MCP tools still read it.

## Consequences
- Gains: several opportunities per customer, lost reasons, a link from quote to opportunity.
- Costs: a data migration, and every consumer of `pipeline_stage` (connectors, MCP tools, webapp) moves to deals.
- What becomes harder: the customer list no longer carries a stage; a customer's state is derived from its deals.
