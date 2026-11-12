import { createFileRoute } from '@tanstack/react-router'
import { PromotionsListFeature } from '#/pages/cursus/promotions/feature/promotions-list-feature'
import { parseArchiveSearch } from '#/pages/cursus/shared/lib/list-filters'

interface PromotionsSearch {
	archived?: 'archived' | 'all'
	training?: string
	q?: string
}

function validateSearch(raw: Record<string, unknown>): PromotionsSearch {
	const archived = parseArchiveSearch(raw.archived)
	return {
		...(archived ? { archived } : {}),
		...(typeof raw.training === 'string' && raw.training !== ''
			? { training: raw.training }
			: {}),
		...(typeof raw.q === 'string' && raw.q !== '' ? { q: raw.q } : {}),
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
			archived={search.archived ?? 'active'}
			trainingId={search.training}
			q={search.q}
			onFiltersChange={(filters) =>
				void navigate({
					replace: true,
					search: {
						...(filters.archived === 'active'
							? {}
							: { archived: filters.archived }),
						...(filters.trainingId ? { training: filters.trainingId } : {}),
						...(filters.q ? { q: filters.q } : {}),
					},
				})
			}
			onManageTrainings={() =>
				void navigate({
					to: '/o/$organizationSlug/cursus/formations',
					params: (previous) => previous,
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
