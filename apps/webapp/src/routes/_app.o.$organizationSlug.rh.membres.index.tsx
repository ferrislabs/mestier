import { createFileRoute } from '@tanstack/react-router'
import { MembersFeature } from '#/pages/hr/feature/members-feature'
import { CONTRACT_LABELS } from '#/pages/hr/lib/rh/contracts'
import type { ContractType } from '#/pages/hr/lib/rh/types'

interface MembersSearch {
	statut?: 'archived' | 'all'
	contrat?: ContractType
}

function isContract(value: unknown): value is ContractType {
	return typeof value === 'string' && value in CONTRACT_LABELS
}

function validateSearch(raw: Record<string, unknown>): MembersSearch {
	return {
		...(raw.statut === 'archived' || raw.statut === 'all'
			? { statut: raw.statut }
			: {}),
		...(isContract(raw.contrat) ? { contrat: raw.contrat } : {}),
	}
}

export const Route = createFileRoute('/_app/o/$organizationSlug/rh/membres/')({
	validateSearch,
	component: MembersRoute,
})

function MembersRoute() {
	const search = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<MembersFeature
			status={search.statut ?? 'active'}
			contract={search.contrat ?? 'all'}
			onFilterChange={(patch) =>
				navigate({
					replace: true,
					search: {
						...(patch.status === 'active' ? {} : { statut: patch.status }),
						...(patch.contract === 'all' ? {} : { contrat: patch.contract }),
					},
				})
			}
			onMemberCreated={(memberId) =>
				navigate({
					to: '/o/$organizationSlug/rh/membres/$memberId',
					params: (previous) => ({ ...previous, memberId }),
				})
			}
		/>
	)
}
