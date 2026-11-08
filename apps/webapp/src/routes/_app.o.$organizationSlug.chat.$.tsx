import { createFileRoute, redirect } from '@tanstack/react-router'
import { buildOrgPath } from '#/modules/org-path'

export const Route = createFileRoute('/_app/o/$organizationSlug/chat/$')({
	beforeLoad: ({ params }) => {
		const rest = params._splat ? `/${params._splat}` : ''
		throw redirect({
			to: buildOrgPath(params.organizationSlug, `/discussions${rest}`),
		})
	},
})
