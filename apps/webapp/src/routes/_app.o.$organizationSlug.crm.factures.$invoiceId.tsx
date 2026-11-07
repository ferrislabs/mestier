import { createFileRoute } from '@tanstack/react-router'
import { InvoiceDetailFeature } from '#/pages/invoices/feature/invoice-detail-feature'

interface InvoiceSearch {
	vue?: 'apercu'
}

function validateSearch(raw: Record<string, unknown>): InvoiceSearch {
	return raw.vue === 'apercu' ? { vue: 'apercu' } : {}
}

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/crm/factures/$invoiceId',
)({
	validateSearch,
	component: InvoiceDetailPage,
})

function InvoiceDetailPage() {
	const { invoiceId } = Route.useParams()
	const search = Route.useSearch()
	const navigate = Route.useNavigate()
	return (
		<InvoiceDetailFeature
			key={invoiceId}
			invoiceId={invoiceId}
			view={search.vue === 'apercu' ? 'apercu' : 'edition'}
			onViewChange={(view) =>
				navigate({
					replace: true,
					search: view === 'apercu' ? { vue: 'apercu' } : {},
				})
			}
		/>
	)
}
