import { createFileRoute } from '@tanstack/react-router'
import { HostCompaniesListFeature } from '#/pages/cursus/host-companies/feature/host-companies-list-feature'
import { hostCompaniesSearchSchema } from '#/pages/cursus/host-companies/lib/host-companies'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/cursus/entreprises/',
)({
	validateSearch: (search) => hostCompaniesSearchSchema.parse(search),
	component: HostCompaniesPage,
})

function HostCompaniesPage() {
	const { archived, q } = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<HostCompaniesListFeature
			archived={archived ?? 'active'}
			q={q}
			onFiltersChange={(filters) =>
				void navigate({
					replace: true,
					search: () => ({
						...(filters.archived === 'active'
							? {}
							: { archived: filters.archived }),
						...(filters.q ? { q: filters.q } : {}),
					}),
				})
			}
			onOpenHostCompany={(hostCompanyId) =>
				void navigate({
					to: '/o/$organizationSlug/cursus/entreprises/$hostCompanyId',
					params: (previous) => ({ ...previous, hostCompanyId }),
					search: {},
				})
			}
		/>
	)
}
