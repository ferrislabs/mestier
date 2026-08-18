import { createFileRoute } from '@tanstack/react-router'
import { QuoteNewFeature } from '#/pages/quotes/feature/quote-new-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/crm/quotes/new',
)({
	component: QuoteNewFeature,
})
