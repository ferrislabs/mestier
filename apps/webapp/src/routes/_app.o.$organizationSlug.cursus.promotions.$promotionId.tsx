import { createFileRoute } from '@tanstack/react-router'
import { PromotionDetailFeature } from '#/pages/cursus/promotions/feature/promotion-detail-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/cursus/promotions/$promotionId',
)({
	component: PromotionDetailRoute,
})

function PromotionDetailRoute() {
	const { organizationSlug, promotionId } = Route.useParams()
	const navigate = Route.useNavigate()

	return (
		<PromotionDetailFeature
			promotionId={promotionId}
			onBack={() =>
				void navigate({
					to: '/o/$organizationSlug/cursus/promotions',
					params: { organizationSlug },
				})
			}
			onOpenTriad={(triadId) =>
				void navigate({
					to: '/o/$organizationSlug/cursus/trinomes/$triadId',
					params: { organizationSlug, triadId },
				})
			}
		/>
	)
}
