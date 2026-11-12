# Cursus UI coherence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give Cursus a grouped sidebar with counts, like CRM, and put every Cursus console page in `PageShell`.

**Architecture:** A `pages/cursus/nav/` folder mirrors `pages/crm-nav/`: a pure path helper, a pure sidebar UI built on `pages/projet/ui/sidebar-parts.tsx`, and a feature that derives counts from the module's existing list hooks. The registry plugs it in with `sidebar:`. Entreprises and Participants pages swap their ad-hoc wrappers for `PageShell` + `PageHeader`.

**Tech Stack:** React 19, TanStack Router/Query, Tailwind v4, vitest + Testing Library, Biome.

**Spec:** `docs/superpowers/specs/2026-11-12-cursus-ui-coherence-design.md`

## Global Constraints
- Frontend only. No API change, no `pnpm gen:api`.
- Imports use the `#/` alias. Files kebab-case. Biome is the formatter.
- UI components take props only (no hooks except what `sidebar-parts` already does).
- User-facing copy in French: group titles exactly `Annuaire` and `Référentiel`.
- jest-dom matchers are not installed: use `toBeTruthy()` / `toBeNull()`.
- Branch `feat/cursus-7-ui-coherence` on top of `feat/cursus-6-company-contract-fields`. Commits in English, no AI mention, never stage `Cargo.lock`.

## Review Focus
- A member without `VIEW_CURSUS` but who is a participant sees only "Mes trinômes" (and no empty group titles). Test in Task 2.
- A group whose entries are all hidden renders no title. Test in Task 2.
- Counts use `pagination.total`, not page length: Participants lists 25 per page. Test in Task 3 via `countOf`.
- A detail URL (`/cursus/entreprises/<id>`) keeps "Entreprises" active. Test in Task 1.
- A count stays hidden while its query is loading (`undefined`), never shows `0`. Test in Task 2.

---

### Task 1: Active-entry path helper

**Files:**
- Create: `apps/webapp/src/pages/cursus/nav/lib/cursus-path.ts`
- Test: `apps/webapp/src/pages/cursus/nav/lib/cursus-path.test.ts`

**Interfaces:**
- Produces: `CURSUS_ENTRY` (const object: `promotions`, `triads`, `myTriads`, `participants`, `companies`, `trainings`, `establishments`), `type CursusEntry`, `activeCursusEntry(modulePath: string): CursusEntry | null`.

- [ ] **Step 1: Write the failing tests**
  - `activeCursusEntry('/cursus/promotions')` → `'promotions'`
  - `activeCursusEntry('/cursus/entreprises/abc')` → `'companies'`
  - `activeCursusEntry('/cursus/mes-trinomes/x')` → `'myTriads'` (must not match `trinomes`)
  - `activeCursusEntry('/cursus/trinomes')` → `'triads'`
  - `activeCursusEntry('/cursus')` → `null`
- [ ] **Step 2: Run** `pnpm vitest run src/pages/cursus/nav/lib` → FAIL (module missing)
- [ ] **Step 3: Implement** with a prefix table and the same `isUnder` rule as `pages/crm-nav/lib/crm-path.ts` (exact match or `prefix/`). Paths: `/cursus/promotions`, `/cursus/trinomes`, `/cursus/mes-trinomes`, `/cursus/participants`, `/cursus/entreprises`, `/cursus/formations`, `/cursus/etablissements`.
- [ ] **Step 4: Run** same command → PASS
- [ ] **Step 5: Commit** `feat(cursus): resolve the active sidebar entry from the path`

### Task 2: Sidebar UI

**Files:**
- Create: `apps/webapp/src/pages/cursus/nav/ui/cursus-sidebar.tsx`
- Test: `apps/webapp/src/pages/cursus/nav/ui/cursus-sidebar.test.tsx` (mirror the router setup of `pages/projet/ui/projet-sidebar.test.tsx`)

**Interfaces:**
- Consumes: `CURSUS_ENTRY`, `CursusEntry` (Task 1); `SidebarSection`, `SidebarEntryContent`, `sidebarItemClassName` from `#/pages/projet/ui/sidebar-parts`; `buildOrgPath` from `#/modules/org-path`.
- Produces:
  ```ts
  export interface CursusSidebarCounts { promotions?: number; triads?: number; participants?: number; companies?: number; trainings?: number; establishments?: number }
  export interface CursusSidebarProps {
    organizationSlug: string
    active: CursusEntry | null
    counts: CursusSidebarCounts
    canView: boolean          // VIEW_CURSUS
    isParticipant: boolean    // shows "Mes trinômes"
    isCollapsed: (groupId: string) => boolean
    onToggleGroup: (groupId: string) => void
  }
  export function CursusSidebar(props: CursusSidebarProps): JSX.Element
  ```
- Groups: `main` (untitled: Promotions/Layers, Trinômes/Users, Mes trinômes/BookUser), `directory` = `Annuaire` (Participants/UserRoundCog, Entreprises/Building2), `reference` = `Référentiel` (Formations/BookOpen, Établissements/School). Icons as in `modules/registry.ts`. "Mes trinômes" has no count. Same structure as `pages/crm-nav/ui/crm-sidebar.tsx`.

- [ ] **Step 1: Write the failing tests**
  - all entries and both titles render with `canView` and `isParticipant`; link "Entreprises 21" shows `21` when `counts.companies = 21`
  - `canView=false, isParticipant=true` → only "Mes trinômes"; `queryByText('Annuaire')` and `queryByText('Référentiel')` are `null`
  - `isParticipant=false` → no "Mes trinômes"
  - `counts.trainings` undefined → no number beside "Formations"
  - clicking the `Annuaire` toggle calls `onToggleGroup('directory')`
  - the active entry link has `aria-current="page"` (check what `crm-sidebar.tsx` sets; match it)
- [ ] **Step 2: Run** `pnpm vitest run src/pages/cursus/nav/ui` → FAIL
- [ ] **Step 3: Implement** `CursusSidebar`.
- [ ] **Step 4: Run** → PASS
- [ ] **Step 5: Commit** `feat(cursus): add a grouped sidebar with counts`

### Task 3: Sidebar feature and registry wiring

**Files:**
- Create: `apps/webapp/src/pages/cursus/nav/feature/cursus-sidebar-feature.tsx`
- Create: `apps/webapp/src/pages/cursus/nav/lib/count-of.ts` + `count-of.test.ts`
- Modify: `apps/webapp/src/modules/registry.ts` (cursus module: add `sidebar: CursusSidebarFeature`, keep `sections` as they are)

**Interfaces:**
- Consumes: Task 1 and 2; `useSidebarGroups` (`#/pages/projet/feature/use-sidebar-groups`, key `mestier:cursus:sidebar-collapsed`); `usePermissions`; `useIsCursusParticipant` (`#/pages/cursus/me/feature/use-cursus-participation`); `useActiveOrganization`; `splitOrgPath`.
- Count sources (each enabled only when `canView`):
  - promotions: `usePromotions(org, { archived: false, trainingId: undefined }, canView)`
  - triads: `useTriads(org, { promotionId: undefined, closed: false }, canView)`
  - participants: `useParticipants(org, { per_page: 1 })` (total of all statuses)
  - companies: `useHostCompanies(org, { per_page: 1 })` from `host-companies/feature/use-host-companies`
  - trainings: `useTrainings(org)` from `catalog/feature/use-cursus-catalog`
  - establishments: `useEstablishments(org)` from the same file
  - `useParticipants`, `useHostCompanies`, `useTrainings`, `useEstablishments` have no `enabled` flag: add an optional trailing `enabled = true` parameter to each, forwarded to `useQuery`, without changing existing callers.
- Produces: `countOf(response: { data: unknown[]; pagination?: { total: number } | null } | undefined): number | undefined` — `pagination.total` when present, else `data.length`, `undefined` when the response is.

- [ ] **Step 1: Write the failing tests for `countOf`**: `undefined` → `undefined`; `{data:[a], pagination:{total:58}}` → `58`; `{data:[a,b]}` → `2`.
- [ ] **Step 2: Run** `pnpm vitest run src/pages/cursus/nav/lib` → FAIL
- [ ] **Step 3: Implement `countOf`, the `enabled` parameters, the feature, and the registry line.**
- [ ] **Step 4: Run** `pnpm vitest run src/pages/cursus src/modules` → PASS; `pnpm tsc --noEmit` shows no new error (baseline 38).
- [ ] **Step 5: Commit** `feat(cursus): show the grouped sidebar in the module`

### Task 4: PageShell on Entreprises and Participants

**Files:**
- Modify: `apps/webapp/src/pages/cursus/host-companies/ui/host-companies-list-ui.tsx:110-115`
- Modify: `apps/webapp/src/pages/cursus/host-companies/ui/host-company-detail-ui.tsx:117-135`
- Modify: `apps/webapp/src/pages/cursus/participants/ui/participants-list-ui.tsx:120-125`
- Modify: `apps/webapp/src/pages/cursus/participants/ui/participant-detail-ui.tsx:69-90`

**Interfaces:**
- Consumes: `PageShell`, `PageHeader` from `#/components/ui/surface`, used as in `pages/cursus/catalog/ui/catalog-block.tsx`.
- Lists: outer wrapper → `PageShell`; title row → `PageHeader` with `title`, `description`, `actions` (the existing create button).
  - Entreprises: description `Les entreprises qui accueillent vos apprentis.`
  - Participants: description `Les apprentis et tuteurs d’entreprise suivis par l’organisation.`
- Details: outer wrapper → `PageShell` (drop `max-w-3xl p-6`), title row → `PageHeader` with the existing name as `title`, existing badges in `leading` or next to it, existing buttons as `actions`. Loading/not-found branches also in `PageShell`.

- [ ] **Step 1: Run the existing tests** `pnpm vitest run src/pages/cursus/host-companies src/pages/cursus/participants` → note baseline PASS
- [ ] **Step 2: Apply the wrapper changes**
- [ ] **Step 3: Run** same command → PASS (update a heading query only if the role changed); `pnpm check` clean on touched files
- [ ] **Step 4: Commit** `feat(cursus): lay out companies and participants pages like the rest of the module`

### Task 5: Finish
- [ ] `pnpm test` all green, `pnpm tsc --noEmit` at baseline, `pnpm check` clean.
- [ ] Commit the spec and plan: `docs(cursus): design and plan for UI coherence`.
- [ ] Push, open PR based on `feat/cursus-6-company-contract-fields`, link it.
