import { CalendarIcon } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Calendar } from '#/components/ui/calendar'
import { Label } from '#/components/ui/label'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import {
	calendarSelectionToDateRange,
	dateRangeToCalendarSelection,
	formatDateRangeFr,
	hasOwnDates,
	shiftEndTimeForNewStartTime,
	type TaskFormValues,
	timeOptionsWith,
} from '#/pages/planning/lib/task-form'

export interface TaskWindowFieldsProps {
	values: Pick<
		TaskFormValues,
		'startDate' | 'endDate' | 'startTime' | 'endTime' | 'allDay'
	>
	/** Whether this draft is (or will become) a subtask — governs whether the range picker offers "hériter du parent" instead of requiring its own dates. */
	isSubtask: boolean
	/** The parent's window, formatted — shown as the range trigger's label while a subtask has no dates of its own. `null` for a root task. */
	windowPlaceholder: string | null
	onChange: (
		patch: Partial<
			Pick<TaskFormValues, 'startDate' | 'endDate' | 'startTime' | 'endTime'>
		>,
	) => void
}

/**
 * The task form's date-range + time-of-day pickers, styled after Google
 * Calendar's own event editor: one range field (a single popover calendar
 * in `range` mode) instead of two separate date inputs, and a scrollable
 * half-hour list for each time instead of free-typed text. Pure
 * presentation, like every sibling in `ui/` — `feature/task-sheet-feature.tsx`
 * never reads this file directly, only through `TaskFormFields`.
 */
export function TaskWindowFields({
	values,
	isSubtask,
	windowPlaceholder,
	onChange,
}: TaskWindowFieldsProps) {
	const hasDates = hasOwnDates(values)
	const rangeLabel = hasDates
		? formatDateRangeFr(values.startDate, values.endDate)
		: (windowPlaceholder ?? 'Sélectionner une période')

	return (
		<div className="flex flex-col gap-4">
			<div className="flex flex-col gap-2">
				<Label>Période</Label>
				<div className="flex items-center gap-2">
					<Popover>
						<PopoverTrigger asChild>
							<Button
								type="button"
								variant="outline"
								className="w-full justify-start font-normal"
							>
								<CalendarIcon />
								{rangeLabel}
							</Button>
						</PopoverTrigger>
						<PopoverContent className="w-auto p-0" align="start">
							<Calendar
								mode="range"
								captionLayout="dropdown"
								selected={dateRangeToCalendarSelection(
									values.startDate,
									values.endDate,
								)}
								onSelect={(selection) => {
									const next = calendarSelectionToDateRange(selection)
									if (next) onChange(next)
								}}
							/>
						</PopoverContent>
					</Popover>
					{isSubtask && hasDates ? (
						<Button
							type="button"
							variant="ghost"
							size="sm"
							className="shrink-0"
							onClick={() => onChange({ startDate: '', endDate: '' })}
						>
							Hériter du parent
						</Button>
					) : null}
				</div>
				{isSubtask && windowPlaceholder && !hasDates ? (
					<p className="text-xs text-muted-foreground">
						Laissez vide pour hériter de la fenêtre du parent.
					</p>
				) : null}
			</div>

			{!values.allDay ? (
				<div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
					<TimeField
						id="task-start-time"
						label="Heure de début"
						value={values.startTime}
						onChange={(newStartTime) =>
							onChange({
								startTime: newStartTime,
								endTime: shiftEndTimeForNewStartTime(values, newStartTime),
							})
						}
					/>
					<TimeField
						id="task-end-time"
						label="Heure de fin"
						value={values.endTime}
						onChange={(endTime) => onChange({ endTime })}
					/>
				</div>
			) : null}
		</div>
	)
}

interface TimeFieldProps {
	id: string
	label: string
	value: string
	onChange: (value: string) => void
}

function TimeField({ id, label, value, onChange }: TimeFieldProps) {
	return (
		<div className="flex flex-col gap-2">
			<Label htmlFor={id}>{label}</Label>
			<Select value={value} onValueChange={onChange}>
				<SelectTrigger id={id} aria-label={label} className="w-full">
					<SelectValue />
				</SelectTrigger>
				{/* Capped rather than left to fill the available viewport space
				 * (`SelectContent`'s own default): 48 half-hour slots would
				 * otherwise render as one very tall list. `max-h-64` keeps
				 * roughly 8 rows in view, like Google Calendar's own time
				 * dropdown, and scrolls to the rest. */}
				<SelectContent className="max-h-64">
					{timeOptionsWith(value).map((time) => (
						<SelectItem key={time} value={time}>
							{time}
						</SelectItem>
					))}
				</SelectContent>
			</Select>
		</div>
	)
}
