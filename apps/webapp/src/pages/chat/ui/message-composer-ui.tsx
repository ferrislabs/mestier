import { Loader2, Paperclip, Send, X } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { cn } from '#/lib/utils'

export interface PendingAttachment {
	id: string
	filename: string
	status: 'uploading' | 'ready' | 'failed'
}

export interface MessageComposerUIProps {
	value: string
	onChange: (value: string) => void
	onSend: () => void
	onTyping: () => void
	pendingAttachments: PendingAttachment[]
	onAttachFiles: (files: FileList) => void
	onRemoveAttachment: (id: string) => void
}

export function MessageComposerUI({
	value,
	onChange,
	onSend,
	onTyping,
	pendingAttachments,
	onAttachFiles,
	onRemoveAttachment,
}: MessageComposerUIProps) {
	const hasBlockingUpload = pendingAttachments.some(
		(attachment) => attachment.status === 'uploading',
	)
	const canSend =
		(value.trim().length > 0 || pendingAttachments.length > 0) &&
		!hasBlockingUpload

	function submit() {
		if (!canSend) return
		onSend()
	}

	return (
		<div className="flex flex-col gap-2 border-t p-3">
			{pendingAttachments.length > 0 ? (
				<ul className="flex flex-wrap gap-2">
					{pendingAttachments.map((attachment) => (
						<li
							key={attachment.id}
							className={cn(
								'flex items-center gap-1.5 rounded-md border px-2 py-1 text-xs',
								attachment.status === 'failed' &&
									'border-destructive text-destructive',
							)}
						>
							{attachment.status === 'uploading' ? (
								<Loader2 className="size-3 animate-spin" />
							) : (
								<Paperclip className="size-3" />
							)}
							<span className="max-w-40 truncate">{attachment.filename}</span>
							<button
								type="button"
								onClick={() => onRemoveAttachment(attachment.id)}
								aria-label={`Retirer ${attachment.filename}`}
							>
								<X className="size-3" />
							</button>
						</li>
					))}
				</ul>
			) : null}

			<div className="flex items-end gap-2">
				<label className="flex size-9 shrink-0 cursor-pointer items-center justify-center rounded-md border text-muted-foreground hover:bg-accent">
					<Paperclip className="size-4" />
					<input
						type="file"
						multiple
						className="hidden"
						onChange={(event) => {
							if (event.target.files && event.target.files.length > 0) {
								onAttachFiles(event.target.files)
							}
							event.target.value = ''
						}}
					/>
				</label>

				<textarea
					value={value}
					onChange={(event) => {
						onChange(event.target.value)
						onTyping()
					}}
					onKeyDown={(event) => {
						if (event.key === 'Enter' && !event.shiftKey) {
							event.preventDefault()
							submit()
						}
					}}
					placeholder="Écrire un message…"
					rows={1}
					className="max-h-40 flex-1 resize-none rounded-md border bg-background p-2 text-sm"
				/>

				<Button
					type="button"
					size="icon"
					disabled={!canSend}
					onClick={submit}
					aria-label="Envoyer"
				>
					<Send className="size-4" />
				</Button>
			</div>
		</div>
	)
}
