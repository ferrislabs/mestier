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
	const { archived } = Route.useSearch()
	const navigate = Route.useNavigate()

	return (
		<HostCompaniesListFeature
			includeArchived={archived === true}
			onIncludeArchivedChange={(includeArchived) =>
				void navigate({
					replace: true,
					search: () => (includeArchived ? { archived: true } : {}),
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
