import { createFileRoute, redirect } from '@tanstack/react-router'
import { moduleLandingPath } from '#/modules/landing'

export const Route = createFileRoute('/_app/planning/')({
	beforeLoad: () => {
		throw redirect({ to: moduleLandingPath('planning') })
	},
})
