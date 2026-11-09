import { createFileRoute, redirect } from '@tanstack/react-router'
import { buildOrgPath } from '#/modules/org-path'

export const Route = createFileRoute('/_app/o/$organizationSlug/cursus/')({
	beforeLoad: ({ params }) => {
		throw redirect({
			to: buildOrgPath(params.organizationSlug, '/cursus/promotions'),
		})
	},
})
