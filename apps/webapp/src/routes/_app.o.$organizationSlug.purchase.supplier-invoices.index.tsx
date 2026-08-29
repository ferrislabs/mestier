import { createFileRoute } from '@tanstack/react-router'
import { SupplierInvoiceInboxFeature } from '#/pages/purchase/feature/supplier-invoice-inbox-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/purchase/supplier-invoices/',
)({
	component: SupplierInvoiceInboxFeature,
})
