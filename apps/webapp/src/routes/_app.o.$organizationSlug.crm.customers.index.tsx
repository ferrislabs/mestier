import { createFileRoute } from '@tanstack/react-router'
import {
	filtersOfSearch,
	searchOfFilters,
	validateClientsSearch,
} from '#/pages/customers/customer-list-url-state'
import { CustomerListFeature } from '#/pages/customers/feature/customer-list-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/crm/customers/',
)({
	validateSearch: validateClientsSearch,
	component: CustomerListPage,
})

function CustomerListPage() {
	const search = Route.useSearch()
	const navigate = Route.useNavigate()
	return (
		<CustomerListFeature
			filters={filtersOfSearch(search)}
			onFiltersChange={(filters) =>
				navigate({ replace: true, search: searchOfFilters(filters) })
			}
		/>
	)
}
