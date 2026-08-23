import { AlertCircle, Loader2, Paperclip, RotateCw } from 'lucide-react'
import type { RefObject } from 'react'
import { Skeleton } from '#/components/ui/skeleton'
import { cn } from '#/lib/utils'
import type { DisplayMessage } from '#/pages/chat/lib/messages-reducer'

export interface MessageListUIProps {
	messages: DisplayMessage[]
	currentUserId: string | null
	isLoadingInitial: boolean
	initialError: string | null
	isLoadingOlder: boolean
	hasMoreOlder: boolean
	editingMessageId: string | null
	editDraft: string
	scrollContainerRef: RefObject<HTMLDivElement | null>
	onScroll: (event: React.UIEvent<HTMLDivElement>) => void
	onRetry: (tempId: string) => void
	onStartEdit: (messageId: string, content: string) => void
	onChangeEditDraft: (value: string) => void
	onConfirmEdit: () => void
	onCancelEdit: () => void
	onDelete: (messageId: string) => void
	resolveAttachmentUrl: (storageKey: string) => string | undefined
}

export function MessageListUI({
	messages,
	currentUserId,
	isLoadingInitial,
	initialError,
	isLoadingOlder,
	hasMoreOlder,
	editingMessageId,
	editDraft,
	scrollContainerRef,
	onScroll,
	onRetry,
	onStartEdit,
	onChangeEditDraft,
	onConfirmEdit,
	onCancelEdit,
	onDelete,
	resolveAttachmentUrl,
}: MessageListUIProps) {
	if (isLoadingInitial) {
		return (
			<div className="flex flex-1 flex-col justify-end gap-3 p-4">
				<Skeleton className="h-10 w-2/3" />
				<Skeleton className="h-10 w-1/2 self-end" />
				<Skeleton className="h-10 w-3/5" />
			</div>
		)
	}

	if (initialError) {
		return (
			<div className="flex flex-1 flex-col items-center justify-center gap-2 text-sm text-destructive">
				<AlertCircle className="size-6" />
				<span>Impossible de charger les messages.</span>
			</div>
		)
	}

	if (messages.length === 0) {
		return (
			<div className="flex flex-1 items-center justify-center text-sm text-muted-foreground">
				Aucun message. Soyez le premier à écrire.
			</div>
		)
	}

	return (
		<div
			ref={scrollContainerRef}
			onScroll={onScroll}
			className="flex flex-1 flex-col gap-3 overflow-y-auto p-4"
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

			{messages.map((entry) => (
				<MessageRow
					key={entry.tempId ?? entry.message.id}
					entry={entry}
					isOwn={
						currentUserId !== null &&
						entry.message.author_user_id === currentUserId
					}
					isEditing={editingMessageId === entry.message.id}
					editDraft={editDraft}
					onRetry={onRetry}
					onStartEdit={onStartEdit}
					onChangeEditDraft={onChangeEditDraft}
					onConfirmEdit={onConfirmEdit}
					onCancelEdit={onCancelEdit}
					onDelete={onDelete}
					resolveAttachmentUrl={resolveAttachmentUrl}
				/>
			))}
		</div>
	)
}

interface MessageRowProps {
	entry: DisplayMessage
	isOwn: boolean
	isEditing: boolean
	editDraft: string
	onRetry: (tempId: string) => void
	onStartEdit: (messageId: string, content: string) => void
	onChangeEditDraft: (value: string) => void
	onConfirmEdit: () => void
	onCancelEdit: () => void
	onDelete: (messageId: string) => void
	resolveAttachmentUrl: (storageKey: string) => string | undefined
}

function MessageRow({
	entry,
	isOwn,
	isEditing,
	editDraft,
	onRetry,
	onStartEdit,
	onChangeEditDraft,
	onConfirmEdit,
	onCancelEdit,
	onDelete,
	resolveAttachmentUrl,
}: MessageRowProps) {
	const { message, status } = entry

	// No REST path maps a chat `user_id` to a display name (see the note in
	// `presence-summary-ui.tsx`) — "Vous" for the caller's own messages is
	// the one name this can say honestly; anyone else renders unlabeled.
	const authorLabel = isOwn ? 'Vous' : 'Membre'

	return (
		<div
			className={cn(
				'group flex max-w-[75%] flex-col gap-1 rounded-lg px-3 py-2 text-sm',
				isOwn
					? 'ml-auto items-end bg-primary text-primary-foreground'
					: 'items-start bg-muted text-foreground',
				status === 'failed' && 'opacity-60',
			)}
		>
			<div className="flex items-center gap-1.5 text-xs opacity-70">
				<span>{authorLabel}</span>
				<span>·</span>
				<time dateTime={message.created_at}>
					{new Date(message.created_at).toLocaleTimeString('fr-FR', {
						hour: '2-digit',
						minute: '2-digit',
					})}
				</time>
				{message.edited_at ? <span>(modifié)</span> : null}
			</div>

			{isEditing ? (
				<div className="flex w-full flex-col gap-1.5">
					<textarea
						value={editDraft}
						onChange={(event) => onChangeEditDraft(event.target.value)}
						className="w-full rounded-md border bg-background p-2 text-sm text-foreground"
						rows={2}
					/>
					<div className="flex justify-end gap-2">
						<button
							type="button"
							onClick={onCancelEdit}
							className="text-xs underline"
						>
							Annuler
						</button>
						<button
							type="button"
							onClick={onConfirmEdit}
							className="text-xs font-semibold underline"
						>
							Enregistrer
						</button>
					</div>
				</div>
			) : (
				<p className="whitespace-pre-wrap break-words">{message.content}</p>
			)}

			{message.attachments.length > 0 ? (
				<ul className="flex flex-col gap-1">
					{message.attachments.map((attachment) => {
						const url = resolveAttachmentUrl(attachment.storage_key)
						return (
							<li key={attachment.storage_key}>
								{url ? (
									<a
										href={url}
										target="_blank"
										rel="noreferrer"
										className="flex items-center gap-1 text-xs underline"
									>
										<Paperclip className="size-3" />
										{attachment.filename}
									</a>
								) : (
									<span className="flex items-center gap-1 text-xs opacity-70">
										<Paperclip className="size-3" />
										{attachment.filename}
									</span>
								)}
							</li>
						)
					})}
				</ul>
			) : null}

			{status === 'sending' ? (
				<span className="text-xs opacity-70">Envoi…</span>
			) : null}

			{status === 'failed' ? (
				<button
					type="button"
					onClick={() => entry.tempId && onRetry(entry.tempId)}
					className="flex items-center gap-1 text-xs font-medium underline"
				>
					<RotateCw className="size-3" />
					Échec de l’envoi — réessayer
				</button>
			) : null}

			{isOwn && status === 'sent' && !isEditing ? (
				<div className="hidden gap-2 text-xs opacity-70 group-hover:flex">
					<button
						type="button"
						onClick={() => onStartEdit(message.id, message.content)}
						className="underline"
					>
						Modifier
					</button>
					<button
						type="button"
						onClick={() => onDelete(message.id)}
						className="underline"
					>
						Supprimer
					</button>
				</div>
			) : null}
		</div>
	)
}
