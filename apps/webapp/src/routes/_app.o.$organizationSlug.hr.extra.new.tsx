import { createFileRoute } from '@tanstack/react-router'
import { ExtraFeature } from '#/pages/hr/feature/extra-feature'
import { isIsoDate } from '#/pages/hr/lib/rh/dates'

interface NewExtraSearch {
	member?: string
	date?: string
}

function validateSearch(raw: Record<string, unknown>): NewExtraSearch {
	return {
		...(typeof raw.member === 'string' && raw.member !== ''
			? { member: raw.member }
			: {}),
		...(isIsoDate(raw.date) ? { date: raw.date } : {}),
	}
}

export const Route = createFileRoute('/_app/o/$organizationSlug/hr/extra/new')({
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
					to: '/o/$organizationSlug/hr/extra',
					search: tab === 'majorations' ? { tab: 'majorations' } : {},
				})
			}
			status="all"
			memberId="all"
			onFiltersChange={(patch) =>
				navigate({
					to: '/o/$organizationSlug/hr/extra',
					search: {
						...(patch.status === 'all' ? {} : { status: patch.status }),
						...(patch.memberId === 'all' ? {} : { member: patch.memberId }),
					},
				})
			}
			creating
			initial={{ memberId: search.member, date: search.date }}
			onCreatingChange={(open) => {
				if (!open) navigate({ to: '/o/$organizationSlug/hr/extra', search: {} })
			}}
		/>
	)
}
