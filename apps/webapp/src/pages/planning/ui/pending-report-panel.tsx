import { AlertCircle, Loader2 } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Textarea } from '#/components/ui/textarea'
import type { AssignmentReport } from '#/hooks/use-assignment-reports'
import { reportedAtLabel } from '../lib/pending-reports'

export interface PendingReportPanelProps {
	reports: AssignmentReport[]
	/** Resolves `reported_by` to a display name — falls back to a
	 * placeholder while the roster hasn't loaded, same as
	 * `resolveAssigneeNames` elsewhere in this module. */
	memberName: (memberId: string) => string
	plannedLabel: string
	reportedLabel: (minutes: number) => string
	/** The one report currently armed for "Appliquer" — its prefill has
	 * already landed in the Détails tab's own fields; this only decides
	 * which report's panel shows the confirmation sentence. */
	applyingReportId: string | null
	onApply: (report: AssignmentReport) => void
	onCancelApply: () => void
	isResolving: boolean
	resolveError: string | null
	dismissingReportId: string | null
	dismissNote: string
	onStartDismiss: (report: AssignmentReport) => void
	onCancelDismiss: () => void
	onDismissNoteChange: (value: string) => void
	onConfirmDismiss: (report: AssignmentReport) => void
}

/**
 * The manager's half of the correction loop, on the task it corrects.
 *
 * Shown above the tabs, not inside "Détails": it needs to stay visible
 * whichever tab the manager is on when they decide to act, and "Appliquer"
 * has to reach into the Détails tab's own fields regardless.
 */
export function PendingReportPanel({
	reports,
	memberName,
	plannedLabel,
	reportedLabel,
	applyingReportId,
	onApply,
	onCancelApply,
	isResolving,
	resolveError,
	dismissingReportId,
	dismissNote,
	onStartDismiss,
	onCancelDismiss,
	onDismissNoteChange,
	onConfirmDismiss,
}: PendingReportPanelProps) {
	if (reports.length === 0) return null

	return (
		<div className="mb-4 space-y-3">
			{reports.map((report) => (
				<div
					key={report.id}
					className="rounded-lg border-2 border-amber-500 bg-amber-50 p-3 dark:bg-amber-950/30"
				>
					<p className="text-xs font-semibold uppercase tracking-wide text-amber-700 dark:text-amber-500">
						Écart signalé
					</p>
					<p className="mt-1 text-sm">
						Prévu : {plannedLabel} · Déclaré :{' '}
						{reportedLabel(report.reported_minutes)}
					</p>
					<p className="mt-1 text-xs text-muted-foreground">
						Par {memberName(report.reported_by)}, le{' '}
						{reportedAtLabel(report.created_at)}
					</p>
					{report.comment ? (
						<p className="mt-1 text-sm text-muted-foreground">
							« {report.comment} »
						</p>
					) : null}

					{resolveError ? (
						<p className="mt-2 flex items-start gap-1.5 text-sm text-destructive">
							<AlertCircle className="mt-0.5 size-4 shrink-0" />
							{resolveError}
						</p>
					) : null}

					{applyingReportId === report.id ? (
						<div className="mt-2 rounded-md bg-card p-2 text-sm">
							<p>
								La durée du projet ci-dessous a été mise à jour pour refléter
								cet écart. Appliquer déplace le planning et change la marge —
								enregistrez pour confirmer.
							</p>
							<Button
								type="button"
								size="sm"
								variant="ghost"
								className="mt-2"
								disabled={isResolving}
								onClick={onCancelApply}
							>
								Annuler
							</Button>
						</div>
					) : dismissingReportId === report.id ? (
						<div className="mt-2 space-y-2">
							<Textarea
								placeholder="Note pour le déclarant (facultatif)"
								rows={2}
								value={dismissNote}
								onChange={(event) => onDismissNoteChange(event.target.value)}
							/>
							<div className="flex gap-2">
								<Button
									type="button"
									size="sm"
									variant="outline"
									disabled={isResolving}
									onClick={onCancelDismiss}
								>
									Annuler
								</Button>
								<Button
									type="button"
									size="sm"
									disabled={isResolving}
									onClick={() => onConfirmDismiss(report)}
								>
									{isResolving ? <Loader2 className="animate-spin" /> : null}
									Confirmer le rejet
								</Button>
							</div>
						</div>
					) : (
						<div className="mt-2 flex gap-2">
							<Button type="button" size="sm" onClick={() => onApply(report)}>
								Appliquer
							</Button>
							<Button
								type="button"
								size="sm"
								variant="outline"
								onClick={() => onStartDismiss(report)}
							>
								Rejeter
							</Button>
						</div>
					)}
				</div>
			))}
		</div>
	)
}
