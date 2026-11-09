import { createFileRoute } from '@tanstack/react-router'
import { ConsoleMyTriadsFeature } from '#/pages/cursus/me/feature/console-my-triads-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/cursus/mes-trinomes/',
)({
	component: MyTriadsRoute,
})

function MyTriadsRoute() {
	const navigate = Route.useNavigate()

	return (
		<ConsoleMyTriadsFeature
			onOpen={(triadId) =>
				void navigate({
					to: '/o/$organizationSlug/cursus/mes-trinomes/$triadId',
					params: (previous) => ({ ...previous, triadId }),
				})
			}
		/>
	)
}
