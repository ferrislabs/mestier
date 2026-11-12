import { useNavigate } from '@tanstack/react-router'
import { useAdminOrganizations } from '#/hooks/use-admin'
import { mutationErrorMessage } from '#/lib/api-error'
import { useAdminList } from '#/pages/admin-layout/feature/use-admin-list'
import { AdminOrganizations } from '#/pages/admin-organizations/ui/admin-organizations'

export function AdminOrganizationsFeature() {
	const navigate = useNavigate()
	const list = useAdminList()
	const organizations = useAdminOrganizations(list.params)

	return (
		<AdminOrganizations
			organizations={organizations.data?.data ?? []}
			pagination={organizations.data?.pagination}
			search={list.input}
			loading={organizations.isPending}
			error={mutationErrorMessage(organizations.error)}
			onSearch={list.setInput}
			onPage={list.setPage}
			onSelect={(organization) =>
				void navigate({
					to: '/admin/organisations/$organizationId',
					params: { organizationId: organization.id },
				})
			}
		/>
	)
}
