import { createFileRoute } from '@tanstack/react-router'
import { CustomerEditFeature } from '#/pages/customers/feature/customer-edit-feature'
import {
	DEFAULT_TAB,
	validateClientSearch,
} from '#/pages/customers/lib/rubriques'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/crm/customers/$customerId',
)({
	validateSearch: validateClientSearch,
	component: CustomerEditPage,
})

function CustomerEditPage() {
	const { customerId } = Route.useParams()
	const search = Route.useSearch()
	return (
		<CustomerEditFeature
			customerId={customerId}
			tab={search.onglet ?? DEFAULT_TAB}
		/>
	)
}
