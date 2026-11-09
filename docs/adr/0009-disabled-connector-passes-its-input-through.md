# 0009. A disabled connector passes the previous output through

Status: accepted
Date: 2026-11-06

## Context
The prototype's "disable node" lets data flow past it without running or failing. The engine ran disabled nodes while the card said "Désactivé".

## Decision
A disabled connector records a succeeded step without calling the connector, its output is the output of its first incoming edge (or the trigger payload), and its outgoing edges are followed as after a success. A disabled condition takes `Then`, a disabled loop iterates zero items.

## Alternatives rejected
- Mark the step skipped: downstream nodes would read it as a branch not taken.
- Fail the run: contradicts the prototype.

## Consequences
- Gains: disabling a node is safe for a live workflow.
- Costs: a disabled branching node needs a fixed choice, documented here.
- What becomes harder: lot 8 must redefine the passthrough on lists of items.
