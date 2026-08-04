import { createFileRoute } from '@tanstack/react-router'
import { QuoteEditFeature } from '#/pages/quotes/feature/quote-edit-feature'

export const Route = createFileRoute('/_app/crm/quotes/$quoteId')({
	component: QuoteEditPage,
})

function QuoteEditPage() {
	const { quoteId } = Route.useParams()
	return <QuoteEditFeature quoteId={quoteId} />
}
