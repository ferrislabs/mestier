import { AlertCircle } from 'lucide-react'
import { useEffect, useRef } from 'react'
import { Button } from '#/components/ui/button'
import type { SupportConversationUIProps } from '#/pages/support/types'
import { ThreadEntry } from '#/pages/support/ui/thread-entry'
import { TicketComposer } from '#/pages/support/ui/ticket-composer'

export function TicketConversationUI(props: SupportConversationUIProps) {
	const scrollRef = useRef<HTMLDivElement>(null)
	const latestEntryId = props.entries.at(-1)?.id
	const embedded = props.layout === 'embedded'

	useEffect(() => {
		const element = scrollRef.current
		if (embedded && latestEntryId && element)
			element.scrollTop = element.scrollHeight
	}, [embedded, latestEntryId])

	const isEmpty = !props.isLoading && !props.error && props.entries.length === 0

	const thread = (
		<>
			{props.error ? (
				<div className="mb-4 flex flex-col gap-3 rounded-md border border-destructive/30 bg-destructive-soft p-4 text-destructive sm:flex-row sm:items-center sm:justify-between">
					<div className="flex items-center gap-3">
						<AlertCircle className="size-5 shrink-0" />
						<p className="text-sm font-medium">{props.error}</p>
					</div>
					<Button onClick={props.onRetry} variant="outline" size="sm">
						Réessayer
					</Button>
				</div>
			) : null}
			{props.isLoading ? (
				<div className="flex items-center justify-center rounded-md border border-dashed bg-muted/25 p-6 text-center text-xs text-muted-foreground">
					Chargement…
				</div>
			) : isEmpty ? (
				<div className="flex items-center justify-center rounded-md border border-dashed bg-muted/25 p-6 text-center text-sm text-muted-foreground">
					Aucun message pour le moment.
				</div>
			) : (
				<ol aria-label="Fil de discussion" className="grid gap-4">
					{props.entries.map((entry) => (
						<ThreadEntry
							key={entry.id}
							entry={entry}
							reviews={props.reviewsByEntry.get(entry.id)}
							pending={props.pendingEntryId === entry.id}
							editing={entry.id === props.editingDraftId}
							onOpenAttachment={props.onOpenAttachment}
							draftActions={
								entry.kind === 'draft' && props.canManage
									? {
											onEdit: () => props.onEditDraft(entry.id),
											onSend: () => props.onSendDraft(entry.id),
											onDelete: () => props.onDeleteDraft(entry.id),
											onToggleApproval: () => props.onToggleApproval(entry.id),
											onComment: (body) => props.onCommentDraft(entry.id, body),
										}
									: undefined
							}
						/>
					))}
				</ol>
			)}
		</>
	)

	const composer = props.canManage ? (
		<TicketComposer
			mode={props.mode}
			onModeChange={props.onModeChange}
			text={props.text}
			onTextChange={props.onTextChange}
			nextStatus={props.nextStatus}
			onNextStatusChange={props.onNextStatusChange}
			editingDraft={props.editingDraftId !== null}
			onCancelDraft={props.onCancelDraft}
			isSubmitting={props.isSubmitting}
			composerError={props.composerError}
			rows={embedded ? 3 : 5}
			onSend={props.onSend}
			onSaveDraft={props.onSaveDraft}
			onAddNote={props.onAddNote}
		/>
	) : null

	if (!embedded)
		return (
			<div className="grid min-w-0 gap-4">
				<div>{thread}</div>
				{composer}
			</div>
		)

	return (
		<div className="flex min-h-0 min-w-0 flex-1 flex-col">
			<div ref={scrollRef} className="min-h-0 flex-1 overflow-y-auto px-5 py-4">
				{thread}
			</div>
			{composer ? (
				<div className="shrink-0 border-border border-t bg-background px-5 py-3">
					{composer}
				</div>
			) : null}
		</div>
	)
}
