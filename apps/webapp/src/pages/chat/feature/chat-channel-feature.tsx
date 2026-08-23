import { useChannel } from '#/hooks/use-chat'
import { useTypingUsers } from '#/hooks/use-typing'
import { ChatChannelHeaderUI } from '#/pages/chat/ui/chat-channel-header-ui'
import { TypingIndicatorUI } from '#/pages/chat/ui/typing-indicator-ui'

export interface ChatChannelFeatureProps {
	channelId: string
}

/**
 * The active channel's own pane. For now this only renders the header and
 * the typing indicator — the message thread itself is #325's scope, stacked
 * on top of this route. `useSendTyping` (in `use-typing.ts`) is ready for
 * #325's composer to call; there is no text input here to drive it yet.
 */
export function ChatChannelFeature({ channelId }: ChatChannelFeatureProps) {
	const channel = useChannel(channelId)
	const typingUserIds = useTypingUsers(channelId)

	return (
		<div className="flex min-h-0 flex-1 flex-col">
			<ChatChannelHeaderUI
				name={channel.data?.name}
				topic={channel.data?.topic}
				isLoading={channel.isLoading}
				isError={channel.isError}
			/>
			<div className="flex-1" />
			<TypingIndicatorUI typingCount={typingUserIds.size} />
		</div>
	)
}
