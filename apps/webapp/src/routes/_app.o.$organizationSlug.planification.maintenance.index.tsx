import { createFileRoute, redirect } from '@tanstack/react-router'
import { buildOrgPath } from '#/modules/org-path'

/**
 * The "Entretien" entry points here. Until the grid lands, it is the recap
 * that opens: the grid replaces this redirect with its own screen.
 */
export const Route = createFileRoute(
	'/_app/o/$organizationSlug/planification/maintenance/',
)({
	beforeLoad: ({ params }) => {
		throw redirect({
			to: buildOrgPath(
				params.organizationSlug,
				'/planification/maintenance/recap',
			),
		})
	},
})
