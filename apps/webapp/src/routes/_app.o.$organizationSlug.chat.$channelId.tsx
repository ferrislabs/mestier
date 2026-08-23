import { createFileRoute } from '@tanstack/react-router'
import { ChatChannelFeature } from '#/pages/chat/feature/chat-channel-feature'

export const Route = createFileRoute(
	'/_app/o/$organizationSlug/chat/$channelId',
)({
	component: ChatChannelPage,
})

function ChatChannelPage() {
	const { channelId } = Route.useParams()
	// Keyed by channelId: switching channels must start the message thread
	// fresh (new reducer state, new scroll position) rather than carrying
	// the previous channel's history and edit/composer state over.
	return <ChatChannelFeature key={channelId} channelId={channelId} />
}
