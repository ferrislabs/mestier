import { createFileRoute } from '@tanstack/react-router'
import { InvoiceCreateFeature } from '#/pages/invoices/feature/invoice-create-feature'

export const Route = createFileRoute('/_app/invoices/')({
	component: InvoiceCreateFeature,
})
