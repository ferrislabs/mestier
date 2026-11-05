import { createFileRoute } from '@tanstack/react-router'
import { StripeFeature } from '#/pages/stripe/feature/stripe-feature'
import {
	cleanStripeSearch,
	parseStripeSearch,
	updateStripeSearch,
} from '#/pages/stripe/lib/stripe-model'

export const Route = createFileRoute('/_app/o/$organizationSlug/crm/stripe')({
	validateSearch: parseStripeSearch,
	component: StripePage,
})

function StripePage() {
	const search = Route.useSearch()
	const navigate = Route.useNavigate()
	return (
		<StripeFeature
			search={search}
			onSearchChange={(patch) =>
				navigate({
					replace: true,
					search: cleanStripeSearch(updateStripeSearch(search, patch)),
				})
			}
		/>
	)
}
