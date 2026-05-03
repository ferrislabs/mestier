import { createFileRoute } from '@tanstack/react-router'
import { CustomerListFeature } from '#/pages/customers/feature/customer-list-feature'

export const Route = createFileRoute('/_app/customers/')({
	component: CustomerListFeature,
})
