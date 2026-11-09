import { createFileRoute } from '@tanstack/react-router'
import { PromotionsListFeature } from '#/pages/cursus/promotions/feature/promotions-list-feature'

interface PromotionsSearch {
	archived?: true
	training?: string
}

function validateSearch(raw: Record<string, unknown>): PromotionsSearch {
	return {
		...(raw.archived === true || raw.archived === 'true'
			? { archived: true as const }
			: {}),
		...(typeof raw.training === 'string' && raw.training !== ''
			? { training: raw.training }
			: {}),
	}
}

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/cursus/promotions/',
)({
	validateSearch,
	component: PromotionsRoute,
})

function PromotionsRoute() {
	const search = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<PromotionsListFeature
			archived={search.archived === true}
			trainingId={search.training}
			onFiltersChange={(filters) =>
				void navigate({
					replace: true,
					search: {
						...(filters.archived ? { archived: true as const } : {}),
						...(filters.trainingId ? { training: filters.trainingId } : {}),
					},
				})
			}
			onManageTrainings={() =>
				void navigate({
					to: '/o/$organizationSlug/settings',
					params: (previous) => previous,
					hash: 'cursus',
				})
			}
			onOpenPromotion={(promotionId) =>
				void navigate({
					to: '/o/$organizationSlug/cursus/promotions/$promotionId',
					params: (previous) => ({ ...previous, promotionId }),
				})
			}
		/>
	)
}
