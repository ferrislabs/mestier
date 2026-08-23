import { createFileRoute } from '@tanstack/react-router'
import { QuoteHandoverFeature } from '#/pages/quotes/feature/quote-handover-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/crm/quotes/$quoteId/handover',
)({
	component: QuoteHandoverPage,
})

function QuoteHandoverPage() {
	const { quoteId } = Route.useParams()

	return <QuoteHandoverFeature quoteId={quoteId} />
}
