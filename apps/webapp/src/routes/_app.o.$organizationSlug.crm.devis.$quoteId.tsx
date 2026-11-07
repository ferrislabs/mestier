import { createFileRoute } from '@tanstack/react-router'
import { QuoteEditFeature } from '#/pages/quotes/feature/quote-edit-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/crm/devis/$quoteId',
)({
	component: QuoteEditPage,
})

function QuoteEditPage() {
	const { quoteId } = Route.useParams()
	return <QuoteEditFeature key={quoteId} quoteId={quoteId} />
}
