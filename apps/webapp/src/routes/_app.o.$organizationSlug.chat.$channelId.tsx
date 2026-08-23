import { createFileRoute } from '@tanstack/react-router'
import { ChatChannelFeature } from '#/pages/chat/feature/chat-channel-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/chat/$channelId',
)({
	component: ChatChannelPage,
})

function ChatChannelPage() {
	const { channelId } = Route.useParams()
	return <ChatChannelFeature channelId={channelId} />
}
