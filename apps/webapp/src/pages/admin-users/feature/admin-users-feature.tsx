import { useAdminUsers } from '#/hooks/use-admin'
import { mutationErrorMessage } from '#/lib/api-error'
import { useAdminList } from '#/pages/admin-layout/feature/use-admin-list'
import { AdminUsers } from '#/pages/admin-users/ui/admin-users'

export function AdminUsersFeature() {
	const list = useAdminList()
	const users = useAdminUsers(list.params)

	return (
		<AdminUsers
			users={users.data?.data ?? []}
			pagination={users.data?.pagination}
			search={list.input}
			loading={users.isPending}
			error={mutationErrorMessage(users.error)}
			onSearch={list.setInput}
			onPage={list.setPage}
		/>
	)
}
