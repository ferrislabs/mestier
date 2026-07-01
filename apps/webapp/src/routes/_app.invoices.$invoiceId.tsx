import { createFileRoute } from '@tanstack/react-router'
import { InvoiceEditFeature } from '#/pages/invoices/feature/invoice-edit-feature'

export const Route = createFileRoute('/_app/invoices/$invoiceId')({
	component: InvoiceEditPage,
})

function InvoiceEditPage() {
	const { invoiceId } = Route.useParams()
	return <InvoiceEditFeature invoiceId={invoiceId} />
}
