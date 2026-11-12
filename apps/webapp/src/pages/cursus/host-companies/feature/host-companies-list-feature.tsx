import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import { mutationErrorMessage } from '#/lib/api-error'
import {
	useCreateHostCompany,
	useHostCompanies,
} from '#/pages/cursus/host-companies/feature/use-host-companies'
import {
	mainAddress,
	toHostCompanyRequest,
} from '#/pages/cursus/host-companies/lib/host-companies'
import {
	HostCompaniesListUI,
	type HostCompanyRowVM,
} from '#/pages/cursus/host-companies/ui/host-companies-list-ui'
import { countOf } from '#/pages/cursus/nav/lib/count-of'
import { useDebouncedSearch } from '#/pages/cursus/shared/feature/use-debounced-search'
import {
	type ArchiveFilter,
	archiveQuery,
	searchQuery,
} from '#/pages/cursus/shared/lib/list-filters'

export interface HostCompanyFiltersState {
	archived: ArchiveFilter
	q: string | undefined
}

export interface HostCompaniesListFeatureProps extends HostCompanyFiltersState {
	onFiltersChange: (filters: HostCompanyFiltersState) => void
	onOpenHostCompany: (hostCompanyId: string) => void
}

export function HostCompaniesListFeature({
	archived,
	q,
	onFiltersChange,
	onOpenHostCompany,
}: HostCompaniesListFeatureProps) {
	const { activeOrganizationId } = useActiveOrganization()
	const canManage = useHasPermission('MANAGE_CURSUS_TRIADS')
	const [createOpen, setCreateOpen] = useState(false)
	const search = useDebouncedSearch(q ?? '', (next) =>
		onFiltersChange({ archived, q: next || undefined }),
	)
	const list = useHostCompanies(activeOrganizationId, {
		...archiveQuery(archived),
		...searchQuery(search.query),
		per_page: 100,
	})
	const createCompany = useCreateHostCompany()

	const rows: HostCompanyRowVM[] = (list.data?.data ?? []).map((company) => ({
		id: company.id,
		name: company.name,
		siret: company.siret ?? '—',
		nafCode: company.naf_code ?? '—',
		city: mainAddress(company.addresses)?.city ?? '—',
		employeeCount:
			company.employee_count == null ? '—' : String(company.employee_count),
		archived: Boolean(company.archived_at),
	}))

	return (
		<HostCompaniesListUI
			rows={rows}
			total={countOf(list.data)}
			search={search.text}
			isLoading={list.isLoading}
			error={mutationErrorMessage(list.error)}
			archived={archived}
			canManage={canManage}
			createOpen={createOpen}
			isCreating={createCompany.isPending}
			createError={mutationErrorMessage(createCompany.error)}
			onSearchChange={search.setText}
			onArchivedChange={(next) => onFiltersChange({ archived: next, q })}
			onResetFilters={() => {
				search.reset()
				onFiltersChange({ archived: 'active', q: undefined })
			}}
			onCreateOpenChange={setCreateOpen}
			onCreate={(values) =>
				createCompany.mutate(
					{
						path: { organization_id: activeOrganizationId },
						body: toHostCompanyRequest(values),
					},
					{
						onSuccess: (response) => {
							setCreateOpen(false)
							onOpenHostCompany(response.data.id)
						},
					},
				)
			}
			onOpen={onOpenHostCompany}
		/>
	)
}
