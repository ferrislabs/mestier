import { createFileRoute } from '@tanstack/react-router'
import { InvoiceListFeature } from '#/pages/invoices/feature/invoice-list-feature'

export const Route = createFileRoute('/_app/o/$organizationSlug/crm/invoices/')(
	{
		component: InvoiceListFeature,
	},
)
