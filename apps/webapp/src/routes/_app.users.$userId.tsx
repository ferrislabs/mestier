import { createFileRoute } from '@tanstack/react-router'
import { UserEditFeature } from '#/pages/users/feature/user-edit-feature'

export const Route = createFileRoute('/_app/users/$userId')({
	component: UserEditPage,
})

function UserEditPage() {
	const { userId } = Route.useParams()
	return <UserEditFeature userId={userId} />
}
