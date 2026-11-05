import { createFileRoute } from '@tanstack/react-router'
import { CustomerPipelineFeature } from '#/pages/customers/feature/customer-pipeline-feature'
import {
	filtersFromSearch,
	parseSearch,
	searchFromFilters,
} from '#/pages/customers/lib/pipeline-filters'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/crm/customers/pipeline',
)({
	validateSearch: parseSearch,
	component: CustomerPipelinePage,
})

function CustomerPipelinePage() {
	const search = Route.useSearch()
	const navigate = Route.useNavigate()
	return (
		<CustomerPipelineFeature
			filters={filtersFromSearch(search)}
			onFiltersChange={(filters) =>
				navigate({ replace: true, search: searchFromFilters(filters) })
			}
		/>
	)
}
