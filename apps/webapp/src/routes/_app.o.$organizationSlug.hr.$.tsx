import { createFileRoute, redirect } from '@tanstack/react-router'
import { legacyHrPath, legacyHrSearch } from '#/modules/legacy-hr-path'
import { buildOrgPath } from '#/modules/org-path'

export const Route = createFileRoute('/_app/o/$organizationSlug/hr/$')({
	validateSearch: (search: Record<string, unknown>) => search,
	beforeLoad: ({ params, search }) => {
		throw redirect({
			to: buildOrgPath(
				params.organizationSlug,
				legacyHrPath(params._splat ?? ''),
			),
			search: legacyHrSearch(search),
		})
	},
})
