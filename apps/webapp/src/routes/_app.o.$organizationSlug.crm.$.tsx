import { createFileRoute, notFound, redirect } from '@tanstack/react-router'
import { legacyCrmPath } from '#/modules/legacy-crm-path'
import { buildOrgPath } from '#/modules/org-path'

export const Route = createFileRoute('/_app/o/$organizationSlug/crm/$')({
	validateSearch: (search: Record<string, unknown>) => search,
	beforeLoad: ({ params, search }) => {
		const current = legacyCrmPath(params._splat ?? '')
		if (current === null) throw notFound()
		throw redirect({
			to: buildOrgPath(params.organizationSlug, current),
			search,
		})
	},
})
