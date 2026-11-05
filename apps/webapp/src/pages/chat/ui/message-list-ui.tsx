import { AlertCircle, Loader2 } from 'lucide-react'
import type { RefObject } from 'react'
import { Skeleton } from '#/components/ui/skeleton'
import type { DisplayMessage } from '#/pages/chat/lib/messages-reducer'
import { buildTimeline } from '#/pages/chat/lib/timeline'
import { MessageRow } from '#/pages/chat/ui/message-row'
import { DaySeparator } from '#/pages/chat/ui/timeline-separators'

export interface MessageListUIProps {
	messages: DisplayMessage[]
	currentUserId: string | null
	isLoadingInitial: boolean
	initialError: string | null
	isLoadingOlder: boolean
	hasMoreOlder: boolean
	editingMessageId: string | null
	editDraft: string
	label?: string
	emptyLabel?: string
	pinnedMessageIds?: ReadonlySet<string>
	savedMessageIds?: ReadonlySet<string>
	threadedMessageIds: ReadonlySet<string>
	scrollContainerRef: RefObject<HTMLDivElement | null>
	onScroll: (event: React.UIEvent<HTMLDivElement>) => void
	onWheel: (event: React.UIEvent<HTMLDivElement>) => void
	onTouchMove: (event: React.UIEvent<HTMLDivElement>) => void
	onRetry: (tempId: string) => void
	onStartEdit: (messageId: string, content: string) => void
	onChangeEditDraft: (value: string) => void
	onConfirmEdit: () => void
	onCancelEdit: () => void
	onDelete: (messageId: string) => void
	onToggleReaction: (
		messageId: string,
		emoji: string,
		currentlyReacted: boolean,
	) => void
	onOpenThread: (messageId: string) => void
	onCopyLink?: (messageId: string) => void
	onTogglePin?: (messageId: string) => void
	onToggleSaved?: (messageId: string) => void
	resolveAttachmentUrl: (storageKey: string) => string | undefined
}

const NO_IDS: ReadonlySet<string> = new Set()

function ignore() {}

export function MessageListUI({
	messages,
	currentUserId,
	isLoadingInitial,
	initialError,
	isLoadingOlder,
	hasMoreOlder,
	editingMessageId,
	editDraft,
	label = 'Messages',
	emptyLabel = 'Aucun message. Soyez le premier à écrire.',
	pinnedMessageIds = NO_IDS,
	savedMessageIds = NO_IDS,
	threadedMessageIds,
	scrollContainerRef,
	onScroll,
	onWheel,
	onTouchMove,
	onRetry,
	onStartEdit,
	onChangeEditDraft,
	onConfirmEdit,
	onCancelEdit,
	onDelete,
	onToggleReaction,
	onOpenThread,
	onCopyLink = ignore,
	onTogglePin = ignore,
	onToggleSaved = ignore,
	resolveAttachmentUrl,
}: MessageListUIProps) {
	if (isLoadingInitial) {
		return (
			<div className="flex min-h-0 flex-1 flex-col justify-end gap-3 p-4">
				<Skeleton className="h-10 w-2/3" />
				<Skeleton className="h-10 w-1/2" />
				<Skeleton className="h-10 w-3/5" />
			</div>
		)
	}

	if (initialError) {
		return (
			<div className="flex min-h-0 flex-1 flex-col items-center justify-center gap-2 text-sm text-destructive">
				<AlertCircle className="size-6" />
				<span>Impossible de charger les messages.</span>
			</div>
		)
	}

	if (messages.length === 0) {
		return (
			<div className="flex min-h-0 flex-1 items-center justify-center px-6 text-center text-muted-foreground text-sm">
				{emptyLabel}
			</div>
		)
	}

	const timeline = buildTimeline(messages)

	return (
		<div
			ref={scrollContainerRef}
			role="log"
			aria-live="polite"
			aria-relevant="additions"
			aria-label={label}
			onScroll={onScroll}
			onWheel={onWheel}
			onTouchMove={onTouchMove}
			className="flex min-h-0 flex-1 flex-col overflow-y-auto pt-2 pb-4"
		>
			{hasMoreOlder ? (
				<div className="flex justify-center py-2">
					{isLoadingOlder ? (
						<Loader2 className="size-4 animate-spin text-muted-foreground" />
					) : (
						<span className="text-xs text-muted-foreground">
							Faites défiler vers le haut pour charger plus
						</span>
					)}
				</div>
			) : null}

			{timeline.map((item) => {
				if (item.kind === 'day')
					return <DaySeparator key={item.key} label={item.label} />
				const { entry } = item
				return (
					<MessageRow
						key={item.key}
						entry={entry}
						continuation={item.continuation}
						isOwn={
							currentUserId !== null &&
							entry.message.author_user_id === currentUserId
						}
						isEditing={editingMessageId === entry.message.id}
						editDraft={editDraft}
						hasThread={threadedMessageIds.has(entry.message.id)}
						pinned={pinnedMessageIds.has(entry.message.id)}
						saved={savedMessageIds.has(entry.message.id)}
						currentUserId={currentUserId}
						onRetry={onRetry}
						onStartEdit={onStartEdit}
						onChangeEditDraft={onChangeEditDraft}
						onConfirmEdit={onConfirmEdit}
						onCancelEdit={onCancelEdit}
						onDelete={onDelete}
						onToggleReaction={onToggleReaction}
						onOpenThread={onOpenThread}
						onCopyLink={onCopyLink}
						onTogglePin={onTogglePin}
						onToggleSaved={onToggleSaved}
						resolveAttachmentUrl={resolveAttachmentUrl}
					/>
				)
			})}
		</div>
	)
}
