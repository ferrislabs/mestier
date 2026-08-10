import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { Switch } from '#/components/ui/switch'
import { Textarea } from '#/components/ui/textarea'
import { cn } from '#/lib/utils'
import type { AbsenceFormValues } from '#/pages/hr/lib/absences'
import { ABSENCE_LABELS } from '#/pages/planning/lib/entries'
import type { TaskFormValues } from '#/pages/planning/lib/task-form'

export interface EventAssigneeOption {
	/** `<kind>:<uuid>`, the shape `PlanningResourceResponse.resource_id` carries. */
	resourceId: string
	displayName: string
}

/**
 * The draft being edited in the panel. The feature owns it: the panel only
 * reflects it and reports intents back.
 */
export type EventEditState =
	| {
			kind: 'task'
			entryId: string
			values: TaskFormValues
			errors: string[]
	  }
	| {
			kind: 'absence'
			entryId: string
			values: AbsenceFormValues
			errors: string[]
	  }

export interface EventEditFormProps {
	state: EventEditState
	assignees: EventAssigneeOption[]
	onChange: (patch: Partial<TaskFormValues & AbsenceFormValues>) => void
	onToggleAssignee: (resourceId: string) => void
	/** Selected resource ids, derived from the draft by the feature. */
	selectedResourceIds: string[]
}

export function EventEditForm({
	state,
	assignees,
	onChange,
	onToggleAssignee,
	selectedResourceIds,
}: EventEditFormProps) {
	return (
		<div className="flex flex-col gap-3">
			{state.kind === 'task' ? (
				<TaskFields
					values={state.values}
					assignees={assignees}
					selectedResourceIds={selectedResourceIds}
					onChange={onChange}
					onToggleAssignee={onToggleAssignee}
				/>
			) : (
				<AbsenceFields values={state.values} onChange={onChange} />
			)}

			{state.errors.length > 0 ? (
				<ul className="rounded-lg bg-destructive-soft px-3 py-2 text-xs text-destructive">
					{state.errors.map((error) => (
						<li key={error}>{error}</li>
					))}
				</ul>
			) : null}
		</div>
	)
}

interface TaskFieldsProps {
	values: TaskFormValues
	assignees: EventAssigneeOption[]
	selectedResourceIds: string[]
	onChange: (patch: Partial<TaskFormValues>) => void
	onToggleAssignee: (resourceId: string) => void
}

function TaskFields({
	values,
	assignees,
	selectedResourceIds,
	onChange,
	onToggleAssignee,
}: TaskFieldsProps) {
	return (
		<>
			<Field label="Titre">
				<Input
					value={values.title}
					onChange={(event) => onChange({ title: event.target.value })}
				/>
			</Field>

			<AllDaySwitch
				checked={values.allDay}
				onChange={(allDay) => onChange({ allDay })}
			/>

			<DateRangeFields
				allDay={values.allDay}
				startDate={values.startDate}
				startTime={values.startTime}
				endDate={values.endDate}
				endTime={values.endTime}
				onChange={onChange}
			/>

			{assignees.length > 0 ? (
				<Field label="Assignés">
					{/* Des pastilles plutôt qu'un sélecteur : un menu déroulant dans un
					    panneau flottant s'avère fragile à fermer, et une équipe d'artisans
					    tient en quelques noms. */}
					<div className="flex flex-wrap gap-1.5">
						{assignees.map((option) => {
							const selected = selectedResourceIds.includes(option.resourceId)
							return (
								<button
									key={option.resourceId}
									type="button"
									aria-pressed={selected}
									onClick={() => onToggleAssignee(option.resourceId)}
									className={cn(
										'rounded-full border px-2.5 py-1 text-xs font-medium transition-colors',
										selected
											? 'border-primary bg-brand-soft text-primary'
											: 'border-border text-muted-foreground hover:bg-muted',
									)}
								>
									{option.displayName}
								</button>
							)
						})}
					</div>
				</Field>
			) : null}

			<Field label="Description">
				<Textarea
					rows={3}
					value={values.description}
					onChange={(event) => onChange({ description: event.target.value })}
				/>
			</Field>
		</>
	)
}

interface AbsenceFieldsProps {
	values: AbsenceFormValues
	onChange: (patch: Partial<AbsenceFormValues>) => void
}

function AbsenceFields({ values, onChange }: AbsenceFieldsProps) {
	return (
		<>
			<Field label="Motif">
				<Select
					value={values.kind}
					onValueChange={(kind) =>
						onChange({ kind: kind as AbsenceFormValues['kind'] })
					}
				>
					<SelectTrigger className="w-full">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						{Object.entries(ABSENCE_LABELS).map(([value, label]) => (
							<SelectItem key={value} value={value}>
								{label}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
			</Field>

			<AllDaySwitch
				checked={values.allDay}
				onChange={(allDay) => onChange({ allDay })}
			/>

			<div className="grid grid-cols-2 gap-2">
				<Field label="Du">
					<Input
						type="date"
						value={values.range.from}
						onChange={(event) =>
							onChange({
								range: { ...values.range, from: event.target.value },
							})
						}
					/>
				</Field>
				<Field label="Au">
					<Input
						type="date"
						value={values.range.to}
						onChange={(event) =>
							onChange({ range: { ...values.range, to: event.target.value } })
						}
					/>
				</Field>
			</div>

			{!values.allDay ? (
				<div className="grid grid-cols-2 gap-2">
					<Field label="Début">
						<Input
							type="time"
							value={values.startTime}
							onChange={(event) => onChange({ startTime: event.target.value })}
						/>
					</Field>
					<Field label="Fin">
						<Input
							type="time"
							value={values.endTime}
							onChange={(event) => onChange({ endTime: event.target.value })}
						/>
					</Field>
				</div>
			) : null}

			<Field label="Note">
				<Textarea
					rows={2}
					value={values.note}
					onChange={(event) => onChange({ note: event.target.value })}
				/>
			</Field>
		</>
	)
}

interface DateRangeFieldsProps {
	allDay: boolean
	startDate: string
	startTime: string
	endDate: string
	endTime: string
	onChange: (patch: Partial<TaskFormValues>) => void
}

function DateRangeFields({
	allDay,
	startDate,
	startTime,
	endDate,
	endTime,
	onChange,
}: DateRangeFieldsProps) {
	return (
		<div className="grid grid-cols-2 gap-2">
			<Field label="Début">
				<Input
					type="date"
					value={startDate}
					onChange={(event) => onChange({ startDate: event.target.value })}
				/>
			</Field>
			<Field label="Fin">
				<Input
					type="date"
					value={endDate}
					onChange={(event) => onChange({ endDate: event.target.value })}
				/>
			</Field>

			{!allDay ? (
				<>
					<Input
						type="time"
						value={startTime}
						aria-label="Heure de début"
						onChange={(event) => onChange({ startTime: event.target.value })}
					/>
					<Input
						type="time"
						value={endTime}
						aria-label="Heure de fin"
						onChange={(event) => onChange({ endTime: event.target.value })}
					/>
				</>
			) : null}
		</div>
	)
}

function AllDaySwitch({
	checked,
	onChange,
}: {
	checked: boolean
	onChange: (checked: boolean) => void
}) {
	return (
		<div className="flex items-center justify-between gap-3 text-sm">
			<Label htmlFor="event-all-day">Journée entière</Label>
			<Switch id="event-all-day" checked={checked} onCheckedChange={onChange} />
		</div>
	)
}

function Field({
	label,
	children,
}: {
	label: string
	children: React.ReactNode
}) {
	return (
		<div className="flex flex-col gap-1">
			<Label className="text-xs text-muted-foreground">{label}</Label>
			{children}
		</div>
	)
}
