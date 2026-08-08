import { createFileRoute, redirect } from '@tanstack/react-router'
import { moduleLandingPath } from '#/modules/landing'
import { buildOrgPath } from '#/modules/org-path'

export const Route = createFileRoute('/_app/o/$organizationSlug/crm/')({
	beforeLoad: ({ params }) => {
		throw redirect({
			to: buildOrgPath(params.organizationSlug, moduleLandingPath('crm')),
		})
	},
})
