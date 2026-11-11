# 0016. Documents store a Mestier block format, not the editor's JSON

Status: accepted
Date: 2026-11-11

## Context

Knowledge documents are edited block by block in the webapp with BlockNote (`docs/chantiers/documents.md`). The same documents are written and read in Markdown by the API and the MCP assistant, searched by the existing Postgres full-text trigger on `body`, and may reference Drive files, photos, table views, customers, projects and quotes, all scoped by `org_id`.

## Decision

A document's content is a versioned block format owned by Mestier (`{ "version": 1, "blocks": [...] }`), stored in `knowledge_articles.content JSONB`. The domain validates it (`DocumentContent::parse`) and renders it to Markdown, which the server writes to `body` on every save. Markdown sent by the API or the assistant is parsed into the same blocks with `pulldown-cmark`. The webapp translates between BlockNote's blocks and this format in `lib/blocks.ts`.

## Alternatives rejected

- Store BlockNote's JSON as is: the database format would follow a 0.x library's releases, and the Rust side could neither validate it nor render it without reimplementing BlockNote's schema anyway.
- Store Markdown only: callouts, image widths, mentions and table views have no faithful Markdown form, and every save from the editor would lose them.
- Store ProseMirror or Yjs documents: needed only for real-time co-editing, which is out of scope, and opaque to the domain.

## Consequences

- Gains: the domain is the single gate for every write path (webapp, Markdown API, MCP), search and the assistant read plain Markdown, and changing the editor later only touches the webapp translation.
- Costs: a translation layer in the webapp and a Markdown parser and renderer in the domain, kept in step by round-trip tests.
- What becomes harder: real-time co-editing later would need a CRDT next to this format, or a migration away from it.
