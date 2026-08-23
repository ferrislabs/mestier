import { useChannel } from '#/hooks/use-chat'
import { ChatChannelHeaderUI } from '#/pages/chat/ui/chat-channel-header-ui'

export interface ChatChannelFeatureProps {
	channelId: string
}

/**
 * The active channel's own pane. For now this only renders the header — the
 * message thread itself is #325's scope, stacked on top of this route.
 */
export function ChatChannelFeature({ channelId }: ChatChannelFeatureProps) {
	const channel = useChannel(channelId)

	return (
		<div className="flex min-h-0 flex-1 flex-col">
			<ChatChannelHeaderUI
				name={channel.data?.name}
				topic={channel.data?.topic}
				isLoading={channel.isLoading}
				isError={channel.isError}
			/>
			<div className="flex-1" />
		</div>
	)
}
