# 0018. Surveys have their own schema and store their definition as one JSONB graph

Status: proposed
Date: 2026-11-14

## Context

The questionnaire module (`survey` in code, `docs/chantiers/questionnaires.md`) collects answers from members and, for public questionnaires, from anonymous respondents. A questionnaire is a graph of questions with branching. The graph is frozen when the questionnaire opens, and every read, whether to render it or to validate an answer, needs the whole graph.

## Decision

Questionnaires, campaigns and responses live in a dedicated Postgres schema, `survey`, with the tables `survey.campaign`, `survey.survey` and `survey.response`. The definition of a questionnaire is a single JSONB column, `survey.survey.graph`, written and read whole and never normalized, as `automation.workflow_version` does for workflows. Answers are one JSONB object per response. Every table carries `org_id`.

## Alternatives rejected

- The tables of the Connaissances module: they persist content written by an LLM or a human, and have no notion of a response, a respondent or a completion state.
- Nodes and edges normalized into tables: the graph is frozen at opening and always read in full, so the joins buy nothing, and a partial write could leave a graph the validator would refuse.

## Consequences

- Gains: one row read yields the whole definition; the schema is isolated and dropped with a single statement; the graph shape can evolve without a migration.
- Costs: the database cannot enforce the graph's integrity, so the domain must validate it before opening. JSONB contents are not covered by foreign keys.
- What becomes harder: querying across questions in SQL, for example statistics per question, needs JSONB operators or a read model.
- One completed response per person is enforced by the database. `survey.response.unique_respondent` copies `NOT allow_multiple_responses` when the response completes; the setting cannot change once the questionnaire is open, so the copy never goes stale. Two partial unique indexes state the rule: `(survey_id, member_id)` and `(survey_id, identifier_value)`, each restricted to completed responses whose `unique_respondent` is true and whose key is not null.
- Tenant isolation holds in the schema too: `survey.survey` references its campaign through `(org_id, campaign_id)`, a response references its questionnaire through `(org_id, survey_id)` and its member through `(member_id, org_id)`, so no row can point at another organization's row.
