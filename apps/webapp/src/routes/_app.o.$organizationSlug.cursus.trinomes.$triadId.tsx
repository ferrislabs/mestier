import { createFileRoute } from '@tanstack/react-router'
import { TriadDetailFeature } from '#/pages/cursus/triads/feature/triad-detail-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/cursus/trinomes/$triadId',
)({
	component: TriadDetailRoute,
})

function TriadDetailRoute() {
	const { organizationSlug, triadId } = Route.useParams()
	const navigate = Route.useNavigate()

	return (
		<TriadDetailFeature
			triadId={triadId}
			onBack={() =>
				void navigate({
					to: '/o/$organizationSlug/cursus/trinomes',
					params: { organizationSlug },
				})
			}
			onOpenPromotion={(promotionId) =>
				void navigate({
					to: '/o/$organizationSlug/cursus/promotions/$promotionId',
					params: { organizationSlug, promotionId },
				})
			}
		/>
	)
}
