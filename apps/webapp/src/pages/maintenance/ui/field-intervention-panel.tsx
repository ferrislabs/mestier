import { AlertCircle, CheckCircle2, Loader2, Wrench } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'

interface FieldInterventionPanelProps {
	/** Used to tie the label to the date field, one per task on the screen. */
	taskId: string
	isOpen: boolean
	/** `YYYY-MM-DD`, what the field currently holds. */
	date: string
	/** `YYYY-MM-DD`, the last day that can be picked: today. */
	maxDate: string
	isSubmitting: boolean
	error: string | null
	/** Set once the visit was sent: already worded, e.g. `14 oct.`. */
	sentOn: string | null
	onOpen: () => void
	onCancel: () => void
	onDateChange: (date: string) => void
	onSubmit: () => void
}

/**
 * The field half of a maintenance visit, on a task row: one button to say it
 * was done, a date that starts on today, and what happens next.
 *
 * Written like `TaskReportPanel` for a gloved hand: tall controls, one column.
 */
export function FieldInterventionPanel({
	taskId,
	isOpen,
	date,
	maxDate,
	isSubmitting,
	error,
	sentOn,
	onOpen,
	onCancel,
	onDateChange,
	onSubmit,
}: FieldInterventionPanelProps) {
	if (sentOn) {
		return (
			<p className="mt-3 flex items-start gap-2 rounded-lg border bg-muted/40 p-3 text-sm font-medium text-primary">
				<CheckCircle2 className="mt-0.5 size-4 shrink-0" />
				Intervention du {sentOn} envoyée : le conducteur doit la valider.
			</p>
		)
	}

	if (!isOpen) {
		return (
			<Button
				type="button"
				size="lg"
				variant="outline"
				className="mt-3 h-12 w-full"
				onClick={onOpen}
			>
				<Wrench />
				Intervention faite
			</Button>
		)
	}

	const dateId = `intervention-date-${taskId}`

	return (
		<div className="mt-3 rounded-lg border bg-muted/40 p-3">
			<Label htmlFor={dateId} className="text-sm font-semibold">
				Date de l’intervention
			</Label>
			<Input
				id={dateId}
				type="date"
				className="mt-1 h-12 text-base"
				value={date}
				max={maxDate}
				onChange={(event) => onDateChange(event.target.value)}
			/>
			<p className="mt-1 text-xs text-muted-foreground">
				Le conducteur la validera avant qu’elle compte.
			</p>

			{error ? (
				<p
					role="alert"
					className="mt-2 flex items-start gap-1.5 text-sm text-destructive"
				>
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
					disabled={isSubmitting || date === ''}
					onClick={onSubmit}
				>
					{isSubmitting ? <Loader2 className="animate-spin" /> : null}
					Envoyer
				</Button>
			</div>
		</div>
	)
}
