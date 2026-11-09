import { createFileRoute } from '@tanstack/react-router'
import { ConsoleMyTriadFeature } from '#/pages/cursus/me/feature/console-my-triads-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/cursus/mes-trinomes/$triadId',
)({
	component: MyTriadRoute,
})

function MyTriadRoute() {
	const { organizationSlug, triadId } = Route.useParams()
	const navigate = Route.useNavigate()

	return (
		<ConsoleMyTriadFeature
			triadId={triadId}
			onBack={() =>
				void navigate({
					to: '/o/$organizationSlug/cursus/mes-trinomes',
					params: { organizationSlug },
				})
			}
		/>
	)
}
