import { createFileRoute } from '@tanstack/react-router'
import { TriadsListFeature } from '#/pages/cursus/triads/feature/triads-list-feature'

interface TriadsSearch {
	closed?: boolean
	promotion?: string
}

function validateSearch(raw: Record<string, unknown>): TriadsSearch {
	const closed =
		raw.closed === true || raw.closed === 'true'
			? true
			: raw.closed === false || raw.closed === 'false'
				? false
				: undefined
	return {
		...(closed === undefined ? {} : { closed }),
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
			closed={search.closed}
			onFiltersChange={(filters) =>
				void navigate({
					replace: true,
					search: {
						...(filters.closed === undefined ? {} : { closed: filters.closed }),
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
