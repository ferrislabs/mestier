import { createFileRoute } from '@tanstack/react-router'
import { QuoteCreateFeature } from '#/pages/quotes/feature/quote-create-feature'

export const Route = createFileRoute('/_app/crm/quotes/')({
	component: QuoteCreateFeature,
})
