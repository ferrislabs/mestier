import { createFileRoute } from '@tanstack/react-router'
import { QuoteListFeature } from '#/pages/quotes/feature/quote-list-feature'

export const Route = createFileRoute('/_app/o/$organizationSlug/crm/quotes/')({
	component: QuoteListFeature,
})
