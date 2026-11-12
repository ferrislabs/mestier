# Cursus UI coherence — design

## Intent
The Cursus module looks unfinished: its sidebar is a bare list of links, and page bodies switch between a centred column and full width. Bring it in line with the CRM and RH modules.

## Goals
- A Cursus sidebar built like the CRM one: grouped entries, a count beside each entry, collapsible groups.
- Every Cursus console page uses `PageShell` (centred, `max-w-6xl`).

## Non-goals
- No backend change, no new endpoint.
- No change to page content (columns, forms, dialogs) beyond the layout wrapper.
- The participant portal (`/portail`) keeps its own layout.

## Approaches
1. Custom sidebar with groups and counts, as in CRM. **Picked by the user.**
2. Group titles only, through the registry, no counts. Rejected: the sidebar stays thin.

## Design
- `pages/cursus/nav/` (new), mirroring `pages/crm-nav/`:
  - `ui/cursus-sidebar.tsx`: pure, reuses `SidebarSection`, `SidebarEntryContent` and `sidebarItemClassName` from `pages/projet/ui/sidebar-parts.tsx`.
  - `feature/cursus-sidebar-feature.tsx`: computes counts from the module's existing list hooks, like `CrmSidebarFeature`; collapsed state via `useSidebarGroups`.
  - Registered with `sidebar: CursusSidebarFeature` on the `cursus` module in `modules/registry.ts`.
- Groups and counts (my call, the user did not object):
  - untitled: Promotions (active), Trinômes (active), Mes trinômes (no count, only for a participant)
  - **Annuaire**: Participants, Entreprises (active)
  - **Référentiel**: Formations (active), Établissements
- Entries keep their permission gates (`VIEW_CURSUS`, `cursus-participant`). A count is hidden while its query loads.
- Entreprises and Participants list and detail pages wrap their body in `PageShell` + `PageHeader`, like Formations.

## Left to the builder
- Exact icons (keep the registry's ones).
- Whether counts reuse list queries as-is or pass a filter already supported by the API.
