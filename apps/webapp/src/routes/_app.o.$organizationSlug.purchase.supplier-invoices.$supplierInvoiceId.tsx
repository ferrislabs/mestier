import { createFileRoute } from '@tanstack/react-router'
import { SupplierInvoiceDetailFeature } from '#/pages/purchase/feature/supplier-invoice-detail-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/purchase/supplier-invoices/$supplierInvoiceId',
)({
	component: SupplierInvoiceDetailPage,
})

function SupplierInvoiceDetailPage() {
	const { supplierInvoiceId } = Route.useParams()
	return <SupplierInvoiceDetailFeature supplierInvoiceId={supplierInvoiceId} />
}
