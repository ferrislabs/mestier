import { useMemo, useState } from 'react'
import {
	useChannel,
	useCreateThread,
	useThreads,
	useThreadsGatewaySync,
} from '#/hooks/use-chat'
import { ChatChannelHeaderUI } from '#/pages/chat/ui/chat-channel-header-ui'
import { MessageComposerUI } from '#/pages/chat/ui/message-composer-ui'
import { MessageListUI } from '#/pages/chat/ui/message-list-ui'
import { TypingIndicatorUI } from '#/pages/chat/ui/typing-indicator-ui'
import { ChatThreadPanelFeature } from './chat-thread-panel-feature'
import { useMessageThreadUi } from './use-message-thread-ui'

export interface ChatChannelFeatureProps {
	channelId: string
}

export function ChatChannelFeature({ channelId }: ChatChannelFeatureProps) {
	const channel = useChannel(channelId)
	const { messageListProps, composerProps, typingCount } =
		useMessageThreadUi(channelId)

	const threads = useThreads(channelId)
	useThreadsGatewaySync(channelId)
	const createThread = useCreateThread()
	const [openThreadChannelId, setOpenThreadChannelId] = useState<string | null>(
		null,
	)

	const threadByOriginMessageId = useMemo(() => {
		const map = new Map<string, string>()
		for (const thread of threads.data ?? []) {
			if (thread.origin_message_id) {
				map.set(thread.origin_message_id, thread.id)
			}
		}
		return map
	}, [threads.data])

	function handleOpenThread(messageId: string) {
		const existing = threadByOriginMessageId.get(messageId)
		if (existing) {
			setOpenThreadChannelId(existing)
			return
		}

		createThread.mutate(
			{
				path: { channel_id: channelId },
				body: { name: 'Fil de discussion', origin_message_id: messageId },
			} as never,
			{
				onSuccess: (response) => {
					setOpenThreadChannelId((response as { data: { id: string } }).data.id)
				},
			},
		)
	}

	return (
		<div className="flex min-h-0 flex-1">
			<div className="flex min-h-0 flex-1 flex-col">
				<ChatChannelHeaderUI
					name={channel.data?.name}
					topic={channel.data?.topic}
					isLoading={channel.isLoading}
					isError={channel.isError}
				/>
				<MessageListUI
					{...messageListProps}
					threadedMessageIds={new Set(threadByOriginMessageId.keys())}
					onOpenThread={handleOpenThread}
				/>
				<TypingIndicatorUI typingCount={typingCount} />
				<MessageComposerUI {...composerProps} />
			</div>
			{openThreadChannelId ? (
				<ChatThreadPanelFeature
					key={openThreadChannelId}
					threadChannelId={openThreadChannelId}
					onClose={() => setOpenThreadChannelId(null)}
				/>
			) : null}
		</div>
	)
}
