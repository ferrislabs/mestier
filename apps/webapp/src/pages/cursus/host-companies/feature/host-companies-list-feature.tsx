import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { useHasPermission } from '#/hooks/use-permissions'
import { mutationErrorMessage } from '#/lib/api-error'
import {
	useCreateHostCompany,
	useHostCompanies,
} from '#/pages/cursus/host-companies/feature/use-host-companies'
import { toHostCompanyRequest } from '#/pages/cursus/host-companies/lib/host-companies'
import {
	HostCompaniesListUI,
	type HostCompanyRowVM,
} from '#/pages/cursus/host-companies/ui/host-companies-list-ui'

export interface HostCompaniesListFeatureProps {
	includeArchived: boolean
	onIncludeArchivedChange: (includeArchived: boolean) => void
	onOpenHostCompany: (hostCompanyId: string) => void
}

export function HostCompaniesListFeature({
	includeArchived,
	onIncludeArchivedChange,
	onOpenHostCompany,
}: HostCompaniesListFeatureProps) {
	const { activeOrganizationId } = useActiveOrganization()
	const canManage = useHasPermission('MANAGE_CURSUS_TRIADS')
	const [createOpen, setCreateOpen] = useState(false)
	const list = useHostCompanies(activeOrganizationId, {
		archived: includeArchived,
		per_page: 100,
	})
	const createCompany = useCreateHostCompany()

	const rows: HostCompanyRowVM[] = (list.data?.data ?? []).map((company) => ({
		id: company.id,
		name: company.name,
		siret: company.siret ?? '—',
		nafCode: company.naf_code ?? '—',
		employeeCount:
			company.employee_count == null ? '—' : String(company.employee_count),
		archived: Boolean(company.archived_at),
	}))

	return (
		<HostCompaniesListUI
			rows={rows}
			isLoading={list.isLoading}
			error={mutationErrorMessage(list.error)}
			includeArchived={includeArchived}
			canManage={canManage}
			createOpen={createOpen}
			isCreating={createCompany.isPending}
			createError={mutationErrorMessage(createCompany.error)}
			onIncludeArchivedChange={onIncludeArchivedChange}
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
