import { createFileRoute } from '@tanstack/react-router'
import { InvoiceNewFeature } from '#/pages/invoices/feature/invoice-new-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/crm/invoices/new',
)({
	component: InvoiceNewFeature,
})
