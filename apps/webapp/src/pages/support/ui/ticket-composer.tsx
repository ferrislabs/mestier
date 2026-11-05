import { Lock, Send } from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { ToggleGroup, ToggleGroupItem } from '#/components/ui/toggle-group'
import { cn } from '#/lib/utils'
import { SUPPORT_COLUMNS } from '#/pages/support/lib/columns'
import type {
	SupportComposerMode,
	SupportConversationUIProps,
	SupportStatus,
} from '#/pages/support/types'
import { ConfirmSendButton } from '#/pages/support/ui/confirm-send-button'
import { useConfirmation } from '#/pages/support/ui/use-confirmation'

export type TicketComposerProps = Pick<
	SupportConversationUIProps,
	| 'mode'
	| 'onModeChange'
	| 'text'
	| 'onTextChange'
	| 'nextStatus'
	| 'onNextStatusChange'
	| 'onCancelDraft'
	| 'onSend'
	| 'onSaveDraft'
	| 'onAddNote'
	| 'isSubmitting'
	| 'composerError'
> & {
	editingDraft: boolean
	rows?: number
}

export function TicketComposer({
	mode,
	onModeChange,
	text,
	onTextChange,
	nextStatus,
	onNextStatusChange,
	editingDraft,
	onCancelDraft,
	onSend,
	onSaveDraft,
	onAddNote,
	isSubmitting,
	composerError,
	rows = 5,
}: TicketComposerProps) {
	const empty = text.trim() === ''
	const blocked = empty || isSubmitting
	const sendConfirmation = useConfirmation()
	const submit = () => {
		if (blocked) return
		if (mode === 'note') onAddNote()
		else sendConfirmation.press(onSend)
	}
	return (
		<section
			aria-label="Écrire"
			className={cn(
				'overflow-hidden rounded-lg border bg-card shadow-xs',
				mode === 'note' ? 'border-state-today/50' : 'border-border',
			)}
		>
			<div className="flex items-center gap-2 border-border border-b px-3 py-2">
				{editingDraft ? (
					<p className="flex-1 text-sm">
						Modification d’un brouillon{' '}
						<span className="text-muted-foreground">
							: les approbations seront remises à zéro.
						</span>
					</p>
				) : (
					<ToggleGroup
						type="single"
						size="sm"
						value={mode}
						onValueChange={(value) => {
							if (value !== 'reply' && value !== 'note') return
							sendConfirmation.cancel()
							onModeChange(value as SupportComposerMode)
						}}
						aria-label="Type de message"
					>
						<ToggleGroupItem value="reply" className="px-3">
							<Send />
							Réponse au client
						</ToggleGroupItem>
						<ToggleGroupItem value="note" className="px-3">
							<Lock />
							Note interne
						</ToggleGroupItem>
					</ToggleGroup>
				)}
				{editingDraft && (
					<Button
						type="button"
						variant="ghost"
						size="sm"
						onClick={onCancelDraft}
					>
						Annuler
					</Button>
				)}
			</div>
			<textarea
				value={text}
				onChange={(event) => {
					sendConfirmation.cancel()
					onTextChange(event.target.value)
				}}
				onKeyDown={(event) => {
					if (event.key === 'Enter' && (event.metaKey || event.ctrlKey)) {
						event.preventDefault()
						submit()
					}
				}}
				rows={rows}
				placeholder={
					mode === 'note'
						? 'Une information pour l’équipe : le client ne la verra pas.'
						: 'Votre réponse au client…'
				}
				aria-label={mode === 'note' ? 'Note interne' : 'Réponse au client'}
				className={cn(
					'block w-full resize-y bg-transparent px-4 py-3 text-sm outline-none placeholder:text-muted-foreground',
					mode === 'note' && 'bg-state-today/5',
				)}
			/>
			{composerError ? (
				<p className="px-4 pb-2 text-destructive text-sm">{composerError}</p>
			) : null}
			<div className="flex flex-wrap items-center gap-2 border-border border-t px-3 py-2">
				{mode === 'reply' ? (
					<>
						<span className="text-muted-foreground text-xs">
							Après l’envoi :
						</span>
						<Select
							value={nextStatus}
							onValueChange={(value) =>
								onNextStatusChange(value as SupportStatus)
							}
						>
							<SelectTrigger
								size="sm"
								aria-label="Statut après l’envoi"
								className="min-w-[190px]"
							>
								<SelectValue />
							</SelectTrigger>
							<SelectContent>
								{SUPPORT_COLUMNS.map((status) => (
									<SelectItem key={status.status} value={status.status}>
										{status.label}
									</SelectItem>
								))}
							</SelectContent>
						</Select>
						<span className="ml-auto flex gap-2">
							<Button
								type="button"
								variant="outline"
								size="sm"
								disabled={blocked}
								onClick={onSaveDraft}
							>
								{editingDraft
									? 'Mettre à jour le brouillon'
									: 'Enregistrer en brouillon'}
							</Button>
							<ConfirmSendButton
								confirmation={sendConfirmation}
								onSend={onSend}
								disabled={blocked}
							/>
						</span>
					</>
				) : (
					<>
						<span className="text-muted-foreground text-xs">
							Visible uniquement par l’équipe support.
						</span>
						<Button
							type="button"
							size="sm"
							className="ml-auto"
							disabled={blocked}
							onClick={onAddNote}
						>
							Ajouter la note
						</Button>
					</>
				)}
			</div>
		</section>
	)
}
