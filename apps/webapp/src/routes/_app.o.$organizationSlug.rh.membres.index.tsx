import { createFileRoute } from '@tanstack/react-router'
import { TeamListFeature } from '#/pages/hr/feature/team-list-feature'

/**
 * `index`, not `rh.membres.tsx`. A bare `rh.membres.tsx` becomes the *parent*
 * of `rh.membres.$memberId.temps-de-travail` under file-based routing, and
 * since it renders the list rather than an `<Outlet/>`, the child route matched
 * and never appeared: the work-time page showed the member list instead. Same
 * shape the CRM uses for `crm.clients.index` beside `crm.clients.$customerId`.
 */
export const Route = createFileRoute('/_app/o/$organizationSlug/rh/membres/')({
	component: TeamListFeature,
})
