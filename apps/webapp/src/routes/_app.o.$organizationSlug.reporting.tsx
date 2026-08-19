import { createFileRoute } from '@tanstack/react-router'
import { ProfitabilityFeature } from '#/pages/reporting/feature/profitability-feature'

export const Route = createFileRoute('/_app/o/$organizationSlug/reporting')({
	component: ProfitabilityFeature,
})
