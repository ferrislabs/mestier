import { createFileRoute } from '@tanstack/react-router'
import { PipelineFeature } from '#/pages/deals/feature/pipeline-feature'
import {
	filtersFromSearch,
	parseSearch,
	searchFromFilters,
} from '#/pages/deals/lib/pipeline-filters'

export const Route = createFileRoute('/_app/o/$organizationSlug/crm/pipeline')({
	validateSearch: parseSearch,
	component: DealPipelinePage,
})

function DealPipelinePage() {
	const search = Route.useSearch()
	const navigate = Route.useNavigate()
	return (
		<PipelineFeature
			filters={filtersFromSearch(search)}
			onFiltersChange={(filters) =>
				navigate({ replace: true, search: searchFromFilters(filters) })
			}
		/>
	)
}
