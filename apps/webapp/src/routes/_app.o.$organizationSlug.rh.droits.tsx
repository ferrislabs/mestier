import { createFileRoute } from '@tanstack/react-router'
import { RightsFeature } from '#/pages/hr/feature/rights-feature'

interface RightsSearch {
	compte?: string
}

function validateSearch(raw: Record<string, unknown>): RightsSearch {
	return typeof raw.compte === 'string' && raw.compte !== ''
		? { compte: raw.compte }
		: {}
}

export const Route = createFileRoute('/_app/o/$organizationSlug/rh/droits')({
	validateSearch,
	component: RightsRoute,
})

function RightsRoute() {
	const search = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<RightsFeature
			accountId={search.compte ?? null}
			onAccountChange={(accountId) =>
				navigate({
					replace: true,
					search: { compte: accountId },
				})
			}
		/>
	)
}
