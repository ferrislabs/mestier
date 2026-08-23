# Splitting 73 issues across worktrees

Written 2026-08-23, after the nine M7 to M11 epics were created. Read this
before opening a worktree, and fix it the moment it goes stale.

The limiting factor isn't the number of agents. It's three generated files,
one shared database, and how much review bandwidth one person has.

## Step 0: what has to be true before the first worktree

About an hour of work. That hour is the difference between five parallel
branches and five parallel conflicts.

### One database per worktree

`.env` points at a single database, `mestier`, on port 5433 (see
`docker-compose.yml:61`). Two worktrees running `sqlx migrate run` against it
will wreck each other.

```bash
psql "postgres://ferriskey:ferriskey@localhost:5433/postgres" \
  -c 'CREATE DATABASE mestier_wt_cost_history'

export DATABASE_URL=postgres://ferriskey:ferriskey@localhost:5433/mestier_wt_cost_history
sqlx migrate run --source migrations
```

One thing works in our favor: integration suites create their own throwaway
databases from `DATABASE_URL` (`scratch_pool` and `automation_pool` in
`libs/core/src/application/test_support.rs`), so they follow along
automatically. E2e suites listen on `127.0.0.1:0`, so they never fight over a
port either.

Redis stays shared. Tests use distinct organization IDs, so that holds up,
but it's the one shared resource left.

### A script to generate the API client

`typed-openapi` is a devDependency in `apps/webapp/package.json`, and no
script calls it. Every agent will invent their own command, and two
inventions produce two different clients even without a git conflict.

Adding a `gen:api` to `package.json` fixes that. It has to say where the
OpenAPI document comes from: a server built from the branch, not some dev
server left running. A stale server produces a client that's silently
missing the new routes, no error, and that's already happened once.

### A convention for the two generated files that are tracked

```
apps/webapp/src/api/api.client.ts     fully regenerated on every backend route
apps/webapp/src/routeTree.gen.ts      regenerated on every frontend route
```

Both live in git. Both get rewritten wholesale. So two branches adding a
route will conflict on a large generated file, and resolving that conflict
by hand produces a file that matches neither branch.

The rule: never resolve a conflict in these files by hand, regenerate
instead. `git checkout --ours`, then the generation command, then commit. A
`.gitattributes` entry marking them as generated makes the intent visible.

Checked on 2026-08-23: `routeTree.gen.ts` does not regenerate on its own.
Delete it and `pnpm check` still passes (biome never touches it), but
`pnpm test` fails on four files with
`Failed to resolve import "#/routeTree.gen"`. Vitest doesn't trigger the
TanStack Router plugin. CI (`.github/workflows/docker.yaml`) only runs
`pnpm check` and `pnpm test`, never `pnpm build`. Untracking it would break
CI immediately. It stays tracked. Settled, don't reopen this without also
changing CI.

### What, against expectations, isn't a problem

`.sqlx/` has 317 tracked files, and you'd expect carnage. It's fine: the
filenames are content hashes, so two branches adding different queries add
different files. Additive, not conflicting.

The actual trap is elsewhere: `cargo sqlx prepare` regenerates the whole
directory from whatever just recompiled, and wipes everything when nothing
did. So `cargo clean -p <crate>` first, always.

## The registry files

`CLAUDE.md` already names them, and `libs/handlers-planning/src/lib.rs`
states the rule in a comment:

> Adding an aggregate therefore touches two lines of this file. Those two
> lines are the only place where otherwise independent workstreams collide,
> so this file is owned by whoever integrates them: a workstream reports the
> lines it needs rather than editing them itself.

That rule was written for exactly this situation. The files in question:

- `libs/core/src/infrastructure/registry.rs`
- `libs/handlers-*/src/lib.rs`
- `apps/api/src/router.rs` and `apps/api/src/openapi.rs`
- `apps/webapp/src/modules/registry.ts`
- the root `Cargo.toml`

A two-line conflict resolves in ten seconds. The real problem is an agent
reorganizing the file along the way.

## Migration numbers

The seventeen migrations are already numbered, distinct, right there in the
issues (`20260822000001` through `20260822000017`). That's not a
coincidence, it's what lets ten migrations get written in parallel without a
name collision. No agent renumbers its own.

The catch: sqlx applies migrations in ascending version order. One that
shows up later with a lower number than one already applied is a problem on
whichever database already has the earlier one. Since each worktree has its
own database, that only bites at integration time, and it's the integrator
who renumbers if the merge order ends up different from the issue order.

## The one cross-epic collision I can actually name

#301 and #338 both rewrite
`libs/core/src/infrastructure/profitability/postgres/repository.rs`. #301
changes the rate join, #338 adds supplier cost. They can't run at the same
time, and #301 goes first because it fixes a bug.

Next to that: #341 amends #310, #312, and #316 (confirmed in the issue body:
"Depends on #310, #312 and #316"), not just #310 and #316 like I first
noted. #316 creates the `invoices` table that #341 modifies, and #316 lives
in WT-7, which itself waits on WT-3. So #341 can't run in wave 1 inside
WT-3, its branch wouldn't have the table it's amending yet. It travels with
WT-7 instead, in wave 2, right after #316.

## The waves

A worktree carries an entire vertical stack. A stack is serial by nature, a
worktree is serial by nature, the parallelism happens between worktrees.

### Wave 1, four worktrees

| Worktree | Branch | Issues | Why this one |
|---|---|---|---|
| WT-1 | `chantier/cost-history` | #300 → #301 → #302 ∥ #303 | Start here. A raise rewrites every past margin. It's the worst bug and the cheapest to fix. |
| WT-2 | `chantier/correction-loop` | #287 → #288 → #289 ∥ #290, #291 free | The loop ADR 0002 depends on, and that was never built. |
| WT-3 | `chantier/valid-documents` | #310 → #312 ∥ #313 → #314, then #311 and #315 | Unblocks the whole commercial side. |
| WT-4 | `chantier/ship-the-chat` | #323 → #324 ∥ #327 → #325 → #326 ∥ #328 → #329 | Zero Rust files. Never fights with the others, except on `routeTree.gen.ts`. |

Four, not six, on purpose. Each one produces a stack of PRs to review, and
review is what saturates first.

### Wave 2, after wave 1 merges

| Worktree | Branch | Issues | Waits on |
|---|---|---|---|
| WT-5 | `chantier/cheap-plan` | #292 → #294 ∥ #293 → #295 | nothing |
| WT-6 | `chantier/supplier-invoices` | #336 → #337 ∥ #338 → #339 → #340 | WT-1, for the profitability file |
| WT-7 | `chantier/invoicing` | #316 → #341 → #317 ∥ #318 ∥ #320 → #319 → #321 → #322 | WT-3, and #341 needs to go early in this wave (see above) |
| WT-8 | `chantier/templates` | #296 → #297, and #298 → #299 | nothing |
| WT-9 | `chantier/project-channels` | #345 → #346 ∥ #347 → #348 ∥ #349 | WT-4 for #348 and #349, #225 for #347 |

### Wave 3, alone

`chantier/business-permissions`, #304 → #305 ∥ #306 → #307 ∥ #308 → #309.

It touches every handlers crate. Run it in the middle of a fan-out and it
conflicts with everything. It runs alone, before or after, never during.

### The rest, filling gaps

#330 and #331 are small and frontend-only. #332 waits until nothing reads
`tasks.customer_id` anymore. #342, #343, and #344 wait on WT-7. #333 is
deliberately left unscheduled.

## What every worktree checks before it's done

```bash
cargo fmt --all --check
SQLX_OFFLINE=true cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p mestier-core --lib -- --ignored --test-threads=1
cd apps/webapp && pnpm check && pnpm test
```

And for any migration, the round trip, because a `down` nobody ever ran is a
`down` that loses data without telling you:

```bash
sqlx migrate run --source migrations \
  && sqlx migrate revert --source migrations \
  && sqlx migrate run --source migrations
```

Run an integration suite twice against a freshly created database. A suite
that secretly needs leftovers from an earlier test passes on a dirty
database, and that's already been reported green here when it wasn't true.
