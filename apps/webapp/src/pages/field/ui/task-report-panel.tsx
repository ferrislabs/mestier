import { AlertCircle, Loader2 } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import { Textarea } from '#/components/ui/textarea'
import type { AssignmentReport, FieldTask } from '#/hooks/use-field'
import { plannedMinutesLabel, reportedMinutesLabel } from '../types'

interface TaskReportPanelProps {
	task: FieldTask
	/** The one report that matters for this assignment — pending if there is
	 * one, else the most recently resolved one, else `null` when nothing has
	 * ever been filed. Resolved by `reportForAssignment` in the feature. */
	report: AssignmentReport | null
	/** Whether this task's form is the one currently open — at most one at a
	 * time across the whole screen, matching `staleEntry`'s own single-form
	 * convention on this page. */
	isEditing: boolean
	draftMinutes: string
	draftComment: string
	isSubmitting: boolean
	isWithdrawing: boolean
	error: string | null
	onOpen: () => void
	onCancel: () => void
	onDraftMinutesChange: (value: string) => void
	onDraftCommentChange: (value: string) => void
	onSubmit: () => void
	onWithdraw: () => void
}

/**
 * The correction loop's field half, on one task row: what was planned, what
 * was reported (if anything), and a way to say what really happened.
 *
 * Never shows a rate, a cost, or anybody's salary — minutes only. See
 * `field-day-feature.tsx`'s "minutes, not money" rule.
 */
export function TaskReportPanel({
	task,
	report,
	isEditing,
	draftMinutes,
	draftComment,
	isSubmitting,
	isWithdrawing,
	error,
	onOpen,
	onCancel,
	onDraftMinutesChange,
	onDraftCommentChange,
	onSubmit,
	onWithdraw,
}: TaskReportPanelProps) {
	const planned = plannedMinutesLabel(task)

	if (isEditing) {
		const minutesId = `report-minutes-${task.task_assignment_id}`
		const commentId = `report-comment-${task.task_assignment_id}`
		const parsed = Number(draftMinutes)
		const isZero = draftMinutes.trim() !== '' && parsed === 0

		return (
			<div className="mt-3 rounded-lg border bg-muted/40 p-3">
				<p className="text-xs text-muted-foreground">Prévu : {planned}</p>

				<div className="mt-2">
					<Label htmlFor={minutesId} className="text-sm font-semibold">
						Durée réelle (minutes)
					</Label>
					<Input
						id={minutesId}
						type="number"
						inputMode="numeric"
						min={0}
						step={1}
						className="mt-1 h-12 text-base"
						value={draftMinutes}
						onChange={(event) => onDraftMinutesChange(event.target.value)}
					/>
					{isZero ? (
						<p className="mt-1 text-xs text-muted-foreground">
							Vous déclarez que ce projet n'a pas eu lieu.
						</p>
					) : null}
				</div>

				<div className="mt-2">
					<Label htmlFor={commentId} className="text-sm font-semibold">
						Commentaire (facultatif)
					</Label>
					<Textarea
						id={commentId}
						className="mt-1"
						rows={2}
						value={draftComment}
						onChange={(event) => onDraftCommentChange(event.target.value)}
					/>
				</div>

				{error ? (
					<p className="mt-2 flex items-start gap-1.5 text-sm text-destructive">
						<AlertCircle className="mt-0.5 size-4 shrink-0" />
						{error}
					</p>
				) : null}

				<div className="mt-3 flex gap-2">
					<Button
						type="button"
						variant="outline"
						className="h-11 flex-1"
						disabled={isSubmitting}
						onClick={onCancel}
					>
						Annuler
					</Button>
					<Button
						type="button"
						className="h-11 flex-1"
						disabled={isSubmitting || draftMinutes.trim() === ''}
						onClick={onSubmit}
					>
						{isSubmitting ? <Loader2 className="animate-spin" /> : null}
						{report?.resolution === 'PENDING' ? 'Enregistrer' : 'Déclarer'}
					</Button>
				</div>
			</div>
		)
	}

	if (report?.resolution === 'PENDING') {
		return (
			<div className="mt-3 rounded-lg border bg-muted/40 p-3">
				<p className="text-xs text-muted-foreground">Prévu : {planned}</p>
				<p className="mt-1 text-sm font-medium">
					Déclaré : {reportedMinutesLabel(report.reported_minutes)} — en attente
					de validation
				</p>
				{report.comment ? (
					<p className="mt-1 text-sm text-muted-foreground">{report.comment}</p>
				) : null}
				<div className="mt-2 flex gap-2">
					<Button type="button" size="sm" variant="outline" onClick={onOpen}>
						Modifier
					</Button>
					<Button
						type="button"
						size="sm"
						variant="outline"
						disabled={isWithdrawing}
						onClick={onWithdraw}
					>
						{isWithdrawing ? <Loader2 className="animate-spin" /> : null}
						Retirer
					</Button>
				</div>
			</div>
		)
	}

	if (report) {
		const decisionLabel =
			report.resolution === 'APPLIED'
				? 'Écart appliqué au planning'
				: 'Écart non retenu'

		return (
			<div className="mt-3 rounded-lg border bg-muted/40 p-3">
				<p className="text-xs text-muted-foreground">Prévu : {planned}</p>
				<p className="mt-1 text-sm font-medium">
					Déclaré : {reportedMinutesLabel(report.reported_minutes)}
				</p>
				<p className="mt-1 text-sm text-muted-foreground">{decisionLabel}</p>
				{report.resolution_note ? (
					<p className="mt-1 text-sm text-muted-foreground">
						« {report.resolution_note} »
					</p>
				) : null}
				<Button
					type="button"
					size="sm"
					variant="outline"
					className="mt-2"
					onClick={onOpen}
				>
					Signaler un nouvel écart
				</Button>
			</div>
		)
	}

	return (
		<div className="mt-3 flex items-center justify-between gap-2 rounded-lg border bg-muted/40 p-3">
			<p className="text-xs text-muted-foreground">Prévu : {planned}</p>
			<Button type="button" size="sm" variant="outline" onClick={onOpen}>
				Signaler un écart
			</Button>
		</div>
	)
}
