import { RotateCw } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Textarea } from '#/components/ui/textarea'
import { cn } from '#/lib/utils'
import { formatFileSize, formatTime, userHue } from '#/pages/chat/lib/format'
import type { DisplayMessage } from '#/pages/chat/lib/messages-reducer'
import { AttachmentCard } from '#/pages/chat/ui/attachment-card'
import { MessageText } from '#/pages/chat/ui/message-text'
import { MessageToolbar } from '#/pages/chat/ui/message-toolbar'
import { PersonAvatar } from '#/pages/chat/ui/person-avatar'
import { PhotoPreview } from '#/pages/chat/ui/photo-preview'
import { ReactionChips } from '#/pages/chat/ui/reaction-chips'
import { ThreadIndicator } from '#/pages/chat/ui/thread-indicator'

export interface MessageRowProps {
	entry: DisplayMessage
	isOwn: boolean
	continuation: boolean
	isEditing: boolean
	editDraft: string
	hasThread: boolean
	pinned: boolean
	saved: boolean
	currentUserId: string | null
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
	onCopyLink: (messageId: string) => void
	onTogglePin: (messageId: string) => void
	onToggleSaved: (messageId: string) => void
	resolveAttachmentUrl: (storageKey: string) => string | undefined
}

export function MessageRow({
	entry,
	isOwn,
	continuation,
	isEditing,
	editDraft,
	hasThread,
	pinned,
	saved,
	currentUserId,
	onRetry,
	onStartEdit,
	onChangeEditDraft,
	onConfirmEdit,
	onCancelEdit,
	onDelete,
	onToggleReaction,
	onOpenThread,
	onCopyLink,
	onTogglePin,
	onToggleSaved,
	resolveAttachmentUrl,
}: MessageRowProps) {
	const { message, status } = entry
	const sentAt = new Date(message.created_at)
	const time = formatTime(sentAt)
	const authorLabel = isOwn ? 'Vous' : 'Membre'
	const reactions = message.reactions.map((reaction) => ({
		emoji: reaction.emoji,
		count: reaction.count,
		reactedByMe:
			currentUserId !== null && reaction.user_ids.includes(currentUserId),
	}))

	function toggleEmoji(emoji: string) {
		const reacted = reactions.some(
			(reaction) => reaction.emoji === emoji && reaction.reactedByMe,
		)
		onToggleReaction(message.id, emoji, reacted)
	}

	return (
		<article
			aria-label={`Message de ${authorLabel} à ${time}`}
			className={cn(
				'group relative flex gap-3 px-4 py-0.5 transition-colors focus-within:bg-muted/60 hover:bg-muted/60',
				!continuation && 'mt-3 pt-1',
				status === 'failed' && 'opacity-60',
			)}
		>
			<div className="w-9 shrink-0">
				{continuation ? (
					<time
						dateTime={message.created_at}
						className="block pt-1 text-right text-[11px] text-muted-foreground opacity-0 transition-opacity group-focus-within:opacity-100 group-hover:opacity-100"
					>
						{time}
					</time>
				) : (
					<PersonAvatar
						name={authorLabel}
						hue={userHue(message.author_user_id)}
						size="md"
					/>
				)}
			</div>
			<div className="min-w-0 flex-1">
				{!continuation && (
					<header className="flex items-baseline gap-2">
						<span className="font-semibold text-sm">{authorLabel}</span>
						<time
							dateTime={message.created_at}
							className="text-muted-foreground text-xs"
						>
							{time}
						</time>
						{message.edited_at ? (
							<span className="text-muted-foreground text-xs">(modifié)</span>
						) : null}
					</header>
				)}

				{isEditing ? (
					<div className="flex w-full flex-col gap-1.5">
						<Textarea
							value={editDraft}
							onChange={(event) => onChangeEditDraft(event.target.value)}
							aria-label="Modifier le message"
							rows={2}
						/>
						<div className="flex justify-end gap-2">
							<Button
								type="button"
								variant="ghost"
								size="sm"
								onClick={onCancelEdit}
							>
								Annuler
							</Button>
							<Button type="button" size="sm" onClick={onConfirmEdit}>
								Enregistrer
							</Button>
						</div>
					</div>
				) : (
					<MessageText text={message.content} />
				)}

				{message.attachments.map((attachment) => {
					const url = resolveAttachmentUrl(attachment.storage_key)
					const sizeLabel = formatFileSize(attachment.size_bytes)
					return attachment.mime_type.startsWith('image/') && url ? (
						<PhotoPreview
							key={attachment.storage_key}
							name={attachment.filename}
							sizeLabel={sizeLabel}
							url={url}
						/>
					) : (
						<AttachmentCard
							key={attachment.storage_key}
							name={attachment.filename}
							typeLabel={attachment.mime_type}
							sizeLabel={sizeLabel}
							url={url}
						/>
					)
				})}

				{status === 'sending' ? (
					<p className="text-muted-foreground text-xs">Envoi…</p>
				) : null}

				{status === 'failed' ? (
					<button
						type="button"
						onClick={() => entry.tempId && onRetry(entry.tempId)}
						className="flex items-center gap-1 text-destructive text-xs font-medium underline"
					>
						<RotateCw className="size-3" />
						Échec de l’envoi — réessayer
					</button>
				) : null}

				{status === 'sent' ? (
					<ReactionChips
						reactions={reactions}
						onToggle={(emoji, reacted) =>
							onToggleReaction(message.id, emoji, reacted)
						}
					/>
				) : null}

				{status === 'sent' && hasThread ? (
					<ThreadIndicator onOpen={() => onOpenThread(message.id)} />
				) : null}
			</div>
			{status === 'sent' && !isEditing ? (
				<MessageToolbar
					hasThread={hasThread}
					isOwn={isOwn}
					pinned={pinned}
					saved={saved}
					onReact={toggleEmoji}
					onOpenThread={() => onOpenThread(message.id)}
					onCopyLink={() => onCopyLink(message.id)}
					onTogglePin={() => onTogglePin(message.id)}
					onToggleSaved={() => onToggleSaved(message.id)}
					onEdit={() => onStartEdit(message.id, message.content)}
					onDelete={() => onDelete(message.id)}
				/>
			) : null}
		</article>
	)
}
