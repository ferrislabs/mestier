import { createFileRoute } from '@tanstack/react-router'
import { CustomerEditFeature } from '#/pages/customers/feature/customer-edit-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/crm/customers/$customerId',
)({
	component: CustomerEditPage,
})

function CustomerEditPage() {
	const { customerId } = Route.useParams()
	return <CustomerEditFeature customerId={customerId} />
}
