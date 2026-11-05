import { createFileRoute } from '@tanstack/react-router'
import { LedgerFeature } from '#/pages/treasury/feature/ledger-feature'
import {
	cleanLedgerSearch,
	parseLedgerSearch,
	updateLedgerSearch,
} from '#/pages/treasury/lib/ledger-model'

export const Route = createFileRoute('/_app/o/$organizationSlug/crm/treasury')({
	validateSearch: parseLedgerSearch,
	component: LedgerPage,
})

function LedgerPage() {
	const search = Route.useSearch()
	const navigate = Route.useNavigate()
	return (
		<LedgerFeature
			search={search}
			onSearchChange={(patch) =>
				navigate({
					replace: true,
					search: cleanLedgerSearch(updateLedgerSearch(search, patch)),
				})
			}
		/>
	)
}
