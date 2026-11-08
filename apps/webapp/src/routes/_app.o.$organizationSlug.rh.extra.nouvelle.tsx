import { createFileRoute } from '@tanstack/react-router'
import { ExtraFeature } from '#/pages/hr/feature/extra-feature'
import { isIsoDate } from '#/pages/hr/lib/rh/dates'

interface NewExtraSearch {
	membre?: string
	date?: string
}

function validateSearch(raw: Record<string, unknown>): NewExtraSearch {
	return {
		...(typeof raw.membre === 'string' && raw.membre !== ''
			? { membre: raw.membre }
			: {}),
		...(isIsoDate(raw.date) ? { date: raw.date } : {}),
	}
}

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/rh/extra/nouvelle',
)({
	validateSearch,
	component: NewExtraRoute,
})

function NewExtraRoute() {
	const search = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<ExtraFeature
			tab="declarations"
			onTabChange={(tab) =>
				navigate({
					to: '/o/$organizationSlug/rh/extra',
					search: tab === 'majorations' ? { onglet: 'majorations' } : {},
				})
			}
			status="all"
			memberId="all"
			onFiltersChange={(patch) =>
				navigate({
					to: '/o/$organizationSlug/rh/extra',
					search: {
						...(patch.status === 'all' ? {} : { statut: patch.status }),
						...(patch.memberId === 'all' ? {} : { membre: patch.memberId }),
					},
				})
			}
			creating
			initial={{ memberId: search.membre, date: search.date }}
			onCreatingChange={(open) => {
				if (!open) navigate({ to: '/o/$organizationSlug/rh/extra', search: {} })
			}}
		/>
	)
}
