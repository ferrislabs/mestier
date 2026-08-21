import { createFileRoute } from '@tanstack/react-router'
import { TeamListFeature } from '#/pages/hr/feature/team-list-feature'

/**
 * `index`, not `hr.team.tsx`. A bare `hr.team.tsx` becomes the *parent* of
 * `hr.team.$memberId.work-time` under file-based routing, and since it renders
 * the list rather than an `<Outlet/>`, the child route matched and never
 * appeared: the work-time page showed the team list instead. Same shape the CRM
 * already uses for `crm.customers.index` beside `crm.customers.$customerId`.
 */
export const Route = createFileRoute('/_app/o/$organizationSlug/hr/team/')({
	component: TeamListFeature,
})
