import {
	Check,
	Lock,
	MessageSquarePlus,
	Paperclip,
	Pencil,
	Trash2,
} from 'lucide-react'
import { useState } from 'react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { cn } from '#/lib/utils'
import { formatDateTime, formatRelative } from '#/pages/support/lib/format'
import { entryKindLabel } from '#/pages/support/lib/thread'
import type {
	SupportDraftReviewsVM,
	SupportEntryVM,
} from '#/pages/support/types'
import { ConfirmSendButton } from '#/pages/support/ui/confirm-send-button'
import { PersonAvatar } from '#/pages/support/ui/member-avatar'
import { useConfirmation } from '#/pages/support/ui/use-confirmation'

export interface DraftActions {
	onEdit: () => void
	onSend: () => void
	onDelete: () => void
	onToggleApproval: () => void
	onComment: (body: string) => void
}

function Time({ iso }: { iso: string }) {
	return (
		<time
			dateTime={iso}
			title={formatDateTime(iso)}
			className="text-muted-foreground text-xs"
		>
			{formatRelative(iso)}
		</time>
	)
}

function DraftReviews({
	reviews,
	pending,
	actions,
}: {
	reviews: SupportDraftReviewsVM
	pending: boolean
	actions: DraftActions
}) {
	const [commenting, setCommenting] = useState(false)
	const sendConfirmation = useConfirmation()
	const [comment, setComment] = useState('')

	return (
		<div className="grid gap-2 border-border border-t border-dashed px-4 py-3">
			<p className="text-muted-foreground text-xs">
				{reviews.approvedBy.length > 0
					? `Approuvé par ${reviews.approvedBy.join(', ')}`
					: 'Pas encore d’approbation'}
			</p>
			{reviews.comments.map((review) => (
				<div key={review.id} className="flex items-start gap-2 text-sm">
					<PersonAvatar
						name={review.memberName}
						hue={review.memberHue}
						size="sm"
					/>
					<p className="min-w-0">
						<span className="font-medium">{review.memberName}</span>{' '}
						<span className="text-muted-foreground">: {review.body}</span>
					</p>
				</div>
			))}
			{commenting && (
				<form
					className="flex gap-2"
					onSubmit={(event) => {
						event.preventDefault()
						if (comment.trim() === '') return
						actions.onComment(comment)
						setComment('')
						setCommenting(false)
					}}
				>
					<Input
						value={comment}
						onChange={(event) => setComment(event.target.value)}
						placeholder="Votre avis sur ce brouillon…"
						aria-label="Avis sur le brouillon"
						autoFocus
					/>
					<Button type="submit" size="sm" disabled={comment.trim() === ''}>
						Publier
					</Button>
				</form>
			)}
			<div className="flex flex-wrap items-center gap-1.5">
				<Button
					type="button"
					size="sm"
					variant={reviews.approvedByMe ? 'secondary' : 'outline'}
					onClick={actions.onToggleApproval}
					aria-pressed={reviews.approvedByMe}
				>
					<Check />
					{reviews.approvedByMe ? 'Approuvé' : 'Approuver'}
				</Button>
				<Button
					type="button"
					size="sm"
					variant="ghost"
					onClick={() => setCommenting((value) => !value)}
				>
					<MessageSquarePlus />
					Donner un avis
				</Button>
				<span className="ml-auto flex gap-1.5">
					<Button
						type="button"
						size="sm"
						variant="ghost"
						disabled={pending}
						onClick={actions.onDelete}
						aria-label="Supprimer le brouillon"
					>
						<Trash2 />
					</Button>
					<Button
						type="button"
						size="sm"
						variant="outline"
						disabled={pending}
						onClick={actions.onEdit}
					>
						<Pencil />
						Modifier
					</Button>
					<ConfirmSendButton
						confirmation={sendConfirmation}
						onSend={actions.onSend}
						disabled={pending}
					/>
				</span>
			</div>
		</div>
	)
}

const EMPTY_REVIEWS: SupportDraftReviewsVM = {
	approvedBy: [],
	approvedByMe: false,
	comments: [],
}

export function ThreadEntry({
	entry,
	reviews = EMPTY_REVIEWS,
	pending = false,
	draftActions,
	editing,
	onOpenAttachment,
}: {
	entry: SupportEntryVM
	reviews?: SupportDraftReviewsVM
	pending?: boolean
	draftActions?: DraftActions
	editing?: boolean
	onOpenAttachment: (attachmentId: string) => void
}) {
	const isClient = entry.kind === 'client_message'
	const label = entryKindLabel(entry.kind)

	return (
		<li className={cn('flex gap-3', !isClient && 'flex-row-reverse')}>
			<PersonAvatar name={entry.authorLabel} hue={entry.authorHue} size="lg" />
			<article
				aria-label={`${label} de ${entry.authorLabel}`}
				className={cn(
					'min-w-0 max-w-2/3 overflow-hidden rounded-lg border',
					isClient ? 'rounded-tl-sm' : 'rounded-tr-sm',
					isClient && 'border-border bg-card',
					entry.kind === 'reply' && 'border-primary/25 bg-primary/5',
					entry.kind === 'internal_note' &&
						'border-state-today/40 bg-state-today/8',
					entry.kind === 'draft' &&
						'border-muted-foreground/40 border-dashed bg-card',
					editing && 'ring-2 ring-ring/40',
				)}
			>
				<header className="flex flex-wrap items-center gap-x-2 gap-y-1 px-4 pt-3">
					<span className="font-medium text-sm">{entry.authorLabel}</span>
					<span
						className={cn(
							'inline-flex items-center gap-1 rounded-sm px-1.5 py-px font-medium text-[11px]',
							isClient && 'bg-muted text-muted-foreground',
							entry.kind === 'reply' && 'bg-primary/10 text-primary',
							entry.kind === 'internal_note' &&
								'bg-state-today/15 text-state-today',
							entry.kind === 'draft' &&
								'border border-muted-foreground/40 border-dashed text-muted-foreground',
						)}
					>
						{entry.kind === 'internal_note' && (
							<Lock className="size-3" aria-hidden />
						)}
						{label}
						{entry.kind === 'internal_note' && ' · visible par l’équipe'}
						{entry.kind === 'draft' && ' · pas encore envoyé'}
					</span>
					<span className="ml-auto">
						<Time iso={entry.createdAt} />
					</span>
				</header>
				<p className="whitespace-pre-wrap break-words px-4 pt-1.5 pb-3 text-sm leading-relaxed">
					{entry.body}
				</p>
				{entry.attachments.length > 0 && (
					<div className="flex flex-wrap gap-2 px-4 pb-3">
						{entry.attachments.map((attachment) => (
							<Button
								key={attachment.id}
								type="button"
								variant="outline"
								size="xs"
								onClick={() => onOpenAttachment(attachment.id)}
							>
								<Paperclip aria-hidden="true" />
								{`${attachment.fileName} (${attachment.sizeLabel})`}
							</Button>
						))}
					</div>
				)}
				{entry.kind === 'draft' && draftActions && (
					<DraftReviews
						reviews={reviews}
						pending={pending}
						actions={draftActions}
					/>
				)}
			</article>
		</li>
	)
}
