import { createFileRoute } from '@tanstack/react-router'
import { ChatShellFeature } from '#/pages/chat/feature/chat-shell-feature'

export const Route = createFileRoute('/_app/o/$organizationSlug/chat')({
	component: ChatShellFeature,
})
