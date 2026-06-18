import { useNavigate } from '@tanstack/react-router'
import {
	type User,
	useCreateUser,
	useDeleteUser,
	useUsers,
} from '#/hooks/use-users'
import { UserListUI } from '#/pages/users/ui/user-list-ui'

export function UserListFeature() {
	const navigate = useNavigate()
	const users = useUsers()
	const createUser = useCreateUser()
	const deleteUser = useDeleteUser()

	const handleEdit = (user: User) => {
		void navigate({
			to: '/users/$userId',
			params: { userId: user.id },
		})
	}

	return (
		<UserListUI
			users={users.data?.data ?? []}
			error={
				users.error?.message ??
				createUser.error?.message ??
				deleteUser.error?.message ??
				null
			}
			isLoading={users.isLoading}
			isCreating={createUser.isPending}
			deletingId={
				deleteUser.variables?.path.id && deleteUser.isPending
					? deleteUser.variables.path.id
					: null
			}
			onAdd={(payload) => createUser.mutateAsync({ body: payload })}
			onEdit={handleEdit}
			onDelete={(user) => deleteUser.mutate({ path: { id: user.id } })}
			onRetry={() => void users.refetch()}
		/>
	)
}
