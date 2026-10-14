import {
	AlertCircle,
	Check,
	ClipboardCheck,
	Loader2,
	RefreshCw,
	X,
} from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Label } from '#/components/ui/label'
import {
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import { Textarea } from '#/components/ui/textarea'
import type { QueueRow } from '#/pages/maintenance/lib/validation-lines'

export interface ValidationUIProps {
	organizationName: string
	rows: QueueRow[]
	/** Visits in the queue, which can exceed what one page shows. */
	totalCount: number
	isLoading: boolean
	/** The queue could not be read. */
	loadError: string | null
	onRetry: () => void
	/** The last validate or reject that failed. */
	actionError: string | null
	/** The visit a request is in flight for. */
	pendingId: string | null
	/** The visit whose rejection form is open: at most one. */
	rejectingId: string | null
	rejectNote: string
	onValidate: (id: string) => void
	onStartReject: (id: string) => void
	onCancelReject: () => void
	onRejectNoteChange: (note: string) => void
	onConfirmReject: () => void
}

/**
 * The manager's queue of visits the field declared done. Each row is
 * validated in one click or rejected with a note; a validated visit moves the
 * line's due date, so nothing here is applied until someone looks at it.
 * Nothing fetches here.
 */
export function ValidationUI({
	organizationName,
	rows,
	totalCount,
	isLoading,
	loadError,
	onRetry,
	actionError,
	pendingId,
	rejectingId,
	rejectNote,
	onValidate,
	onStartReject,
	onCancelReject,
	onRejectNoteChange,
	onConfirmReject,
}: ValidationUIProps) {
	return (
		<PageShell>
			<PageHeader
				eyebrow={organizationName}
				title="À valider"
				description="Les interventions que le terrain a déclarées faites. Une fois validée, une intervention décale l’échéance de la prestation."
				actions={
					<Button type="button" variant="outline" onClick={onRetry}>
						<RefreshCw />
						Actualiser
					</Button>
				}
			/>

			{loadError ? (
				<SectionCard className="flex flex-col gap-3 border-destructive/30 bg-destructive-soft p-5 text-destructive sm:flex-row sm:items-center sm:justify-between">
					<div className="flex items-center gap-3">
						<AlertCircle className="size-5 shrink-0" />
						<p className="text-sm font-medium">{loadError}</p>
					</div>
					<Button onClick={onRetry} variant="outline" size="sm">
						Réessayer
					</Button>
				</SectionCard>
			) : null}

			{actionError ? (
				<SectionCard
					role="alert"
					className="flex items-center gap-3 border-destructive/30 bg-destructive-soft p-5 text-destructive"
				>
					<AlertCircle className="size-5 shrink-0" />
					<p className="text-sm font-medium">{actionError}</p>
				</SectionCard>
			) : null}

			<SectionCard>
				<SectionHeader
					title={`En attente (${totalCount})`}
					description={
						totalCount > rows.length
							? `Les ${rows.length} plus anciennes sont affichées ; les autres suivent une fois celles-ci traitées.`
							: 'Les plus anciennes en premier.'
					}
				/>

				{isLoading ? (
					<p className="p-5 text-sm text-muted-foreground">Chargement…</p>
				) : rows.length === 0 && !loadError ? (
					<div className="flex min-h-52 flex-col items-center justify-center gap-3 p-8 text-center">
						<div className="flex size-12 items-center justify-center rounded-lg bg-muted text-muted-foreground">
							<ClipboardCheck className="size-5" />
						</div>
						<p className="font-medium">Aucune visite à valider.</p>
					</div>
				) : (
					<ul className="divide-y">
						{rows.map((row) => (
							<li key={row.id} className="px-5 py-4">
								<div className="flex flex-wrap items-start justify-between gap-3">
									<div className="min-w-0">
										<div className="flex flex-wrap items-center gap-2">
											<p className="text-sm font-medium">{row.title}</p>
											<StatusBadge tone="warning">En attente</StatusBadge>
										</div>
										<p className="mt-0.5 text-sm text-muted-foreground">
											{row.subtitle}
										</p>
										<p className="mt-1 text-xs text-muted-foreground">
											Faite le {row.doneOn}, déclarée par {row.proposedBy}
										</p>
									</div>

									{rejectingId === row.id ? null : (
										<div className="flex shrink-0 gap-2">
											<Button
												type="button"
												size="sm"
												variant="outline"
												disabled={pendingId !== null}
												onClick={() => onStartReject(row.id)}
											>
												<X />
												Rejeter
											</Button>
											<Button
												type="button"
												size="sm"
												disabled={pendingId !== null}
												onClick={() => onValidate(row.id)}
											>
												{pendingId === row.id ? (
													<Loader2 className="animate-spin" />
												) : (
													<Check />
												)}
												Valider
											</Button>
										</div>
									)}
								</div>

								{rejectingId === row.id ? (
									<div className="mt-3 rounded-lg border bg-muted/40 p-3">
										<Label
											htmlFor={`reject-note-${row.id}`}
											className="text-sm font-semibold"
										>
											Motif du rejet (facultatif)
										</Label>
										<Textarea
											id={`reject-note-${row.id}`}
											className="mt-1"
											rows={2}
											value={rejectNote}
											onChange={(event) =>
												onRejectNoteChange(event.target.value)
											}
										/>
										<div className="mt-3 flex justify-end gap-2">
											<Button
												type="button"
												size="sm"
												variant="outline"
												disabled={pendingId !== null}
												onClick={onCancelReject}
											>
												Annuler
											</Button>
											<Button
												type="button"
												size="sm"
												variant="destructive"
												disabled={pendingId !== null}
												onClick={onConfirmReject}
											>
												{pendingId === row.id ? (
													<Loader2 className="animate-spin" />
												) : null}
												Confirmer le rejet
											</Button>
										</div>
									</div>
								) : null}
							</li>
						))}
					</ul>
				)}
			</SectionCard>
		</PageShell>
	)
}
