import { createFileRoute, redirect } from '@tanstack/react-router'
import { moduleLandingPath } from '#/modules/landing'

export const Route = createFileRoute('/_app/crm/')({
	beforeLoad: () => {
		throw redirect({ to: moduleLandingPath('crm') })
	},
})
