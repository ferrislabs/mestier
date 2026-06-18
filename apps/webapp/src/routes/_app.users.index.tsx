import { createFileRoute } from '@tanstack/react-router'
import { UserListFeature } from '#/pages/users/feature/user-list-feature'

export const Route = createFileRoute('/_app/users/')({
	component: UserListFeature,
})
