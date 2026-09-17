import { createFileRoute } from '@tanstack/react-router'
import { BoardFeature } from '#/pages/planification/feature/board-feature'
import { boardSearchSchema } from '#/pages/planification/lib/board-filters'

/**
 * The board (#466): five columns of cards, drag one to advance it, schedule
 * one without leaving the screen. The route was created by #468 ahead of the
 * screen; WS5 filled it in and registered the section in `modules/registry.ts`.
 *
 * #467 put the filter state in the search params. `validateSearch` never
 * throws — `boardSearchSchema` catches every field — so a stale or
 * hand-edited link lands on a board rather than on a router error page, and
 * the feature below stays router-free: it is handed the filters and a way to
 * write them back, nothing more.
 */
export const Route = createFileRoute(
	'/_app/o/$organizationSlug/planification/board',
)({
	validateSearch: (search) => boardSearchSchema.parse(search),
	component: BoardPage,
})

function BoardPage() {
	const filters = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<BoardFeature
			filters={filters}
			// The whole filter set is written at once, rather than merged into
			// whatever the address already held: a key the feature left out is
			// a filter it cleared, and `undefined` is how the router is told to
			// drop it from the URL.
			onFiltersChange={(next) => void navigate({ search: () => next })}
		/>
	)
}
