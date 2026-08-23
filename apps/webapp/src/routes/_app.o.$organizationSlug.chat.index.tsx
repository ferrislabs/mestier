import { createFileRoute } from '@tanstack/react-router'
import { ChatIndexFeature } from '#/pages/chat/feature/chat-index-feature'

export const Route = createFileRoute('/_app/o/$organizationSlug/chat/')({
	component: ChatIndexFeature,
})
