import { MessageComposerUI } from '#/pages/chat/ui/message-composer-ui'
import { MessageListUI } from '#/pages/chat/ui/message-list-ui'
import { ThreadPanelHeaderUI } from '#/pages/chat/ui/thread-panel-header-ui'
import { TypingIndicatorUI } from '#/pages/chat/ui/typing-indicator-ui'
import { useMessageThreadUi } from './use-message-thread-ui'

export interface ChatThreadPanelFeatureProps {
	threadChannelId: string
	onClose: () => void
}

/**
 * A thread is a channel (see `use-message-thread-ui.ts`'s doc comment), so
 * this panel reuses the exact same message list and composer as the parent
 * channel, pointed at the thread's own id. Threads do not nest — the thread
 * action inside a thread's own messages is a no-op, matching how the server
 * models it (a thread's `parent_id` is a `TEXT` channel, never another
 * thread).
 */
export function ChatThreadPanelFeature({
	threadChannelId,
	onClose,
}: ChatThreadPanelFeatureProps) {
	const { messageListProps, composerProps, typingCount } =
		useMessageThreadUi(threadChannelId)

	return (
		<div className="flex w-80 shrink-0 flex-col border-l">
			<ThreadPanelHeaderUI
				replyCount={messageListProps.messages.length}
				onClose={onClose}
			/>
			<MessageListUI
				{...messageListProps}
				threadedMessageIds={new Set()}
				onOpenThread={() => {}}
			/>
			<TypingIndicatorUI typingCount={typingCount} />
			<MessageComposerUI {...composerProps} />
		</div>
	)
}
