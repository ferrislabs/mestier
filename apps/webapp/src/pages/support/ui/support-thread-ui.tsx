import { AlertCircle, Paperclip } from 'lucide-react'
import { useState } from 'react'
import { Button } from '#/components/ui/button'
import { StatusBadge } from '#/components/ui/surface'
import { Textarea } from '#/components/ui/textarea'
import type {
	SupportBadgeTone,
	SupportEntryKind,
	SupportEntryVM,
	SupportThreadUIProps,
} from '#/pages/support/types'

const KIND_TONES: Record<SupportEntryKind, SupportBadgeTone> = {
	client_message: 'neutral',
	reply: 'brand',
	internal_note: 'warning',
	draft: 'neutral',
}

const KIND_SURFACES: Record<SupportEntryKind, string> = {
	client_message: 'bg-card',
	reply: 'bg-card',
	internal_note: 'border-warning/30 bg-warning-soft',
	draft: 'border-dashed bg-muted/25',
}

type SupportEntryHandlers = Pick<
	SupportThreadUIProps,
	| 'canManage'
	| 'editingEntryId'
	| 'pendingEntryId'
	| 'onStartEdit'
	| 'onCancelEdit'
	| 'onSaveEdit'
	| 'onSendDraft'
	| 'onDeleteDraft'
	| 'onOpenAttachment'
>

export function SupportThreadUI({
	entries,
	isLoading,
	error,
	onRetry,
	...handlers
}: SupportThreadUIProps) {
	const isEmpty = !isLoading && !error && entries.length === 0

	return (
		<div className="flex flex-col gap-4 p-5">
			<h2 className="font-semibold">Fil de discussion</h2>

			{error ? (
				<div className="flex flex-col gap-3 rounded-md border border-destructive/30 bg-destructive-soft p-4 text-destructive sm:flex-row sm:items-center sm:justify-between">
					<div className="flex items-center gap-3">
						<AlertCircle className="size-5 shrink-0" />
						<p className="text-sm font-medium">{error}</p>
					</div>
					<Button onClick={onRetry} variant="outline" size="sm">
						Réessayer
					</Button>
				</div>
			) : null}

			{isLoading ? (
				<div className="flex items-center justify-center rounded-md border border-dashed bg-muted/25 p-6 text-center text-xs text-muted-foreground">
					Chargement…
				</div>
			) : isEmpty ? (
				<div className="flex items-center justify-center rounded-md border border-dashed bg-muted/25 p-6 text-center text-sm text-muted-foreground">
					Aucun message pour le moment.
				</div>
			) : (
				<div className="flex flex-col gap-3">
					{entries.map((entry) => (
						<SupportEntry key={entry.id} entry={entry} {...handlers} />
					))}
				</div>
			)}
		</div>
	)
}

interface SupportEntryProps extends SupportEntryHandlers {
	entry: SupportEntryVM
}

function SupportEntry({
	entry,
	canManage,
	editingEntryId,
	pendingEntryId,
	onStartEdit,
	onCancelEdit,
	onSaveEdit,
	onSendDraft,
	onDeleteDraft,
	onOpenAttachment,
}: SupportEntryProps) {
	const isDraft = entry.kind === 'draft'
	const isManaged = canManage && isDraft
	const isEditing = isManaged && editingEntryId === entry.id
	const isPending = pendingEntryId === entry.id

	return (
		<article
			aria-label={`${entry.kindLabel}, ${entry.authorLabel}`}
			className={`flex flex-col gap-2 rounded-md border p-4 ${KIND_SURFACES[entry.kind]}`}
		>
			<div className="flex flex-wrap items-center gap-x-2 gap-y-1">
				<StatusBadge tone={KIND_TONES[entry.kind]}>
					{entry.kindLabel}
				</StatusBadge>
				<span className="text-sm font-medium">{entry.authorLabel}</span>
				<span className="text-xs text-muted-foreground">
					{entry.createdAtLabel}
				</span>
			</div>
			<p className="text-xs text-muted-foreground">{entry.visibilityLabel}</p>

			{isEditing ? (
				<DraftEditor
					initialBody={entry.body}
					isPending={isPending}
					onCancel={onCancelEdit}
					onSave={(body) => onSaveEdit(entry.id, body)}
				/>
			) : (
				<p className="text-sm whitespace-pre-wrap break-words">{entry.body}</p>
			)}

			{entry.attachments.length > 0 ? (
				<div className="flex flex-wrap gap-2">
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
			) : null}

			{isManaged && !isEditing ? (
				<div className="flex flex-wrap gap-2">
					<Button
						type="button"
						variant="outline"
						size="sm"
						disabled={isPending}
						onClick={() => onStartEdit(entry.id)}
					>
						Modifier
					</Button>
					<Button
						type="button"
						size="sm"
						disabled={isPending}
						onClick={() => onSendDraft(entry.id)}
					>
						Envoyer
					</Button>
					<Button
						type="button"
						variant="ghost"
						size="sm"
						disabled={isPending}
						onClick={() => onDeleteDraft(entry.id)}
					>
						Supprimer
					</Button>
				</div>
			) : null}
		</article>
	)
}

interface DraftEditorProps {
	initialBody: string
	isPending: boolean
	onCancel: () => void
	onSave: (body: string) => void
}

function DraftEditor({
	initialBody,
	isPending,
	onCancel,
	onSave,
}: DraftEditorProps) {
	const [text, setText] = useState(initialBody)
	const isBlank = text.trim() === ''

	return (
		<div className="flex flex-col gap-2">
			<Textarea
				aria-label="Modifier le brouillon"
				value={text}
				onChange={(event) => setText(event.target.value)}
			/>
			<div className="flex flex-wrap gap-2">
				<Button
					type="button"
					size="sm"
					disabled={isBlank || isPending}
					onClick={() => onSave(text)}
				>
					Enregistrer
				</Button>
				<Button
					type="button"
					variant="ghost"
					size="sm"
					disabled={isPending}
					onClick={onCancel}
				>
					Annuler
				</Button>
			</div>
		</div>
	)
}
