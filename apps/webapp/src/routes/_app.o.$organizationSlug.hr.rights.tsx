import { createFileRoute } from '@tanstack/react-router'
import { RightsFeature } from '#/pages/hr/feature/rights-feature'
import { DEFAULT_ACCOUNT_ID } from '#/pages/hr/lib/rh/balances'

interface RightsSearch {
	account?: string
}

function validateSearch(raw: Record<string, unknown>): RightsSearch {
	return typeof raw.account === 'string' && raw.account !== ''
		? { account: raw.account }
		: {}
}

export const Route = createFileRoute('/_app/o/$organizationSlug/hr/rights')({
	validateSearch,
	component: RightsRoute,
})

function RightsRoute() {
	const search = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<RightsFeature
			accountId={search.account ?? DEFAULT_ACCOUNT_ID}
			onAccountChange={(accountId) =>
				navigate({
					replace: true,
					search:
						accountId === DEFAULT_ACCOUNT_ID ? {} : { account: accountId },
				})
			}
		/>
	)
}
