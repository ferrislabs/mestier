import { createFileRoute } from '@tanstack/react-router'
import { InvoiceDetailFeature } from '#/pages/invoices/feature/invoice-detail-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/crm/invoices/$invoiceId',
)({
	component: InvoiceDetailPage,
})

function InvoiceDetailPage() {
	const { invoiceId } = Route.useParams()
	return <InvoiceDetailFeature invoiceId={invoiceId} />
}
