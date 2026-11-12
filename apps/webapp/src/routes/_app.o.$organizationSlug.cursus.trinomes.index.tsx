import { createFileRoute } from '@tanstack/react-router'
import { TriadsListFeature } from '#/pages/cursus/triads/feature/triads-list-feature'

interface TriadsSearch {
	status?: 'closed' | 'all'
	promotion?: string
}

function validateSearch(raw: Record<string, unknown>): TriadsSearch {
	return {
		...(raw.status === 'closed' || raw.status === 'all'
			? { status: raw.status }
			: {}),
		...(typeof raw.promotion === 'string' && raw.promotion !== ''
			? { promotion: raw.promotion }
			: {}),
	}
}

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/cursus/trinomes/',
)({
	validateSearch,
	component: TriadsRoute,
})

function TriadsRoute() {
	const search = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<TriadsListFeature
			promotionId={search.promotion}
			status={search.status ?? 'open'}
			onFiltersChange={(filters) =>
				void navigate({
					replace: true,
					search: {
						...(filters.status === 'open' ? {} : { status: filters.status }),
						...(filters.promotionId ? { promotion: filters.promotionId } : {}),
					},
				})
			}
			onOpenTriad={(triadId) =>
				void navigate({
					to: '/o/$organizationSlug/cursus/trinomes/$triadId',
					params: (previous) => ({ ...previous, triadId }),
				})
			}
		/>
	)
}
