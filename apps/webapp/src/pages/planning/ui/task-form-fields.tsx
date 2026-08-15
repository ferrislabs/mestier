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
import {
	resourceIdFromAssigneeRef,
	toggleAssignee,
} from '#/pages/planning/lib/task-drop'
import type { TaskFormValues } from '#/pages/planning/lib/task-form'
import {
	type AssigneeOption,
	AssigneePicker,
} from '#/pages/planning/ui/assignee-picker'
import {
	LabelPicker,
	type LabelPickerOption,
} from '#/pages/planning/ui/label-picker'

// Radix `Select.Item` forbids an empty-string `value` (reserved to mean
// "cleared"), so the "no customer"/"no context" choices each need a
// sentinel — mapped back to `''` in their own `onValueChange` below, which
// is what `TaskFormValues`/`validateTaskDraft` actually key on.
const NO_SELECTION = '__none__'

export interface TaskFormFieldsProps {
	mode: 'create' | 'edit'
	/** Whether this draft is (or will become) a subtask — governs whether dates are required and whether the inherited-window placeholder applies. */
	isSubtask: boolean
	values: TaskFormValues
	onChange: (patch: Partial<TaskFormValues>) => void
	errors: string[]
	/** The parent's window, formatted for the date fields' placeholder — see `lib/subtasks.ts`'s `formatWindowPlaceholder`. `null` for a root task. */
	windowPlaceholder: string | null
	/**
	 * Edit mode only: the task's current customer name, or `null` for none —
	 * static text, never a selector. `PATCH /tasks/{id}` has no
	 * `customer_id`/`customer_context_id` field, so a task's client cannot
	 * change after creation (see `lib/task-form.ts`'s `buildPatchTaskPayload`
	 * doc).
	 */
	customerName?: string | null
	/** Create mode only. */
	customers: { id: string; displayName: string }[]
	customerContexts: { id: string; label: string }[]
	isCustomerContextsLoading?: boolean
	labels: LabelPickerOption[]
	isCreatingLabel?: boolean
	onCreateLabel: (name: string) => void
	assigneeOptions: AssigneeOption[]
}

/**
 * The task form's field set — pure presentation, shared by the create and
 * edit sheets (see `ui/task-sheet.tsx`). `mode` only changes the customer
 * field, the one place create and edit genuinely diverge (see this file's
 * own doc on `customerName`); every other field behaves identically in
 * both.
 */
export function TaskFormFields({
	mode,
	isSubtask,
	values,
	onChange,
	errors,
	windowPlaceholder,
	customerName,
	customers,
	customerContexts,
	isCustomerContextsLoading,
	labels,
	isCreatingLabel,
	onCreateLabel,
	assigneeOptions,
}: TaskFormFieldsProps) {
	const selectedResourceIds = values.assignees.map(resourceIdFromAssigneeRef)

	return (
		<div className="flex flex-col gap-5">
			<div className="flex flex-col gap-2">
				<Label htmlFor="task-title">Titre</Label>
				<Input
					id="task-title"
					value={values.title}
					onChange={(event) => onChange({ title: event.target.value })}
					placeholder="Réunion de chantier, livraison, formation…"
				/>
			</div>

			<div className="flex flex-col gap-2">
				<Label htmlFor="task-description">Description</Label>
				<Textarea
					id="task-description"
					value={values.description}
					onChange={(event) => onChange({ description: event.target.value })}
					placeholder="Optionnel"
				/>
			</div>

			{mode === 'create' ? (
				<div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
					<div className="flex flex-col gap-2">
						<Label htmlFor="task-customer">Client</Label>
						<Select
							value={values.customerId || NO_SELECTION}
							onValueChange={(customerId) =>
								onChange({
									customerId: customerId === NO_SELECTION ? '' : customerId,
									customerContextId: '',
								})
							}
						>
							<SelectTrigger
								id="task-customer"
								aria-label="Client"
								className="w-full"
							>
								<SelectValue />
							</SelectTrigger>
							<SelectContent>
								<SelectItem value={NO_SELECTION}>
									Aucun — réunion, déplacement…
								</SelectItem>
								{customers.map((customer) => (
									<SelectItem key={customer.id} value={customer.id}>
										{customer.displayName}
									</SelectItem>
								))}
							</SelectContent>
						</Select>
					</div>

					{values.customerId ? (
						<div className="flex flex-col gap-2">
							<Label htmlFor="task-customer-context">Contexte</Label>
							<Select
								value={values.customerContextId || NO_SELECTION}
								onValueChange={(customerContextId) =>
									onChange({
										customerContextId:
											customerContextId === NO_SELECTION
												? ''
												: customerContextId,
									})
								}
								disabled={isCustomerContextsLoading}
							>
								<SelectTrigger
									id="task-customer-context"
									aria-label="Contexte"
									className="w-full"
								>
									<SelectValue />
								</SelectTrigger>
								<SelectContent>
									<SelectItem value={NO_SELECTION}>
										Aucun contexte précis
									</SelectItem>
									{customerContexts.map((context) => (
										<SelectItem key={context.id} value={context.id}>
											{context.label}
										</SelectItem>
									))}
								</SelectContent>
							</Select>
						</div>
					) : null}
				</div>
			) : (
				<div className="flex flex-col gap-1.5">
					<Label>Client</Label>
					<p className="text-sm">
						{customerName ?? (
							<span className="text-muted-foreground">
								Aucun client — réunion, déplacement…
							</span>
						)}
					</p>
				</div>
			)}

			<div className="flex items-center justify-between rounded-lg border p-3">
				<div>
					<p className="text-sm font-medium">Journée entière</p>
					<p className="text-xs text-muted-foreground">
						Sans heure précise, la période couvre chaque jour en entier.
					</p>
				</div>
				<Switch
					checked={values.allDay}
					onCheckedChange={(allDay) => onChange({ allDay })}
					aria-label="Journée entière"
				/>
			</div>

			<div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
				<div className="flex flex-col gap-2">
					<Label htmlFor="task-start-date">Date de début</Label>
					<Input
						id="task-start-date"
						type="date"
						value={values.startDate}
						placeholder={windowPlaceholder ?? undefined}
						onChange={(event) => onChange({ startDate: event.target.value })}
					/>
				</div>
				<div className="flex flex-col gap-2">
					<Label htmlFor="task-end-date">Date de fin</Label>
					<Input
						id="task-end-date"
						type="date"
						value={values.endDate}
						placeholder={windowPlaceholder ?? undefined}
						onChange={(event) => onChange({ endDate: event.target.value })}
					/>
				</div>
			</div>

			{!values.allDay ? (
				<div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
					<div className="flex flex-col gap-2">
						<Label htmlFor="task-start-time">Heure de début</Label>
						<Input
							id="task-start-time"
							aria-label="Heure de début"
							placeholder="09:00"
							value={values.startTime}
							onChange={(event) => onChange({ startTime: event.target.value })}
						/>
					</div>
					<div className="flex flex-col gap-2">
						<Label htmlFor="task-end-time">Heure de fin</Label>
						<Input
							id="task-end-time"
							aria-label="Heure de fin"
							placeholder="10:00"
							value={values.endTime}
							onChange={(event) => onChange({ endTime: event.target.value })}
						/>
					</div>
				</div>
			) : null}

			{isSubtask && windowPlaceholder ? (
				<p className="text-xs text-muted-foreground">
					Laissez les dates vides pour hériter de la fenêtre du parent.
				</p>
			) : null}

			<div className="flex items-center justify-between rounded-lg border p-3">
				<div>
					<p className="text-sm font-medium">Rend indisponible</p>
					<p className="text-xs text-muted-foreground">
						Cette tâche rend-elle la personne indisponible pour tout autre
						engagement sur ce créneau ?
					</p>
				</div>
				<Switch
					checked={values.blocksAvailability}
					onCheckedChange={(blocksAvailability) =>
						onChange({ blocksAvailability })
					}
					aria-label="Rend indisponible"
				/>
			</div>

			<div className="flex flex-col gap-2">
				<Label>Labels</Label>
				<LabelPicker
					labels={labels}
					selectedIds={values.labelIds}
					isCreating={isCreatingLabel}
					onToggle={(labelId) =>
						onChange({
							labelIds: values.labelIds.includes(labelId)
								? values.labelIds.filter((id) => id !== labelId)
								: [...values.labelIds, labelId],
						})
					}
					onCreate={onCreateLabel}
				/>
			</div>

			<div className="flex flex-col gap-2">
				<Label>Assignés</Label>
				<AssigneePicker
					options={assigneeOptions}
					selectedResourceIds={selectedResourceIds}
					onToggle={(resourceId) =>
						onChange({
							assignees: toggleAssignee(values.assignees, resourceId),
						})
					}
				/>
				{isSubtask ? (
					<p className="text-xs text-muted-foreground">
						Une sous-tâche n’hérite jamais des assignés de son parent.
					</p>
				) : null}
			</div>

			{errors.length > 0 ? (
				<ul className="flex flex-col gap-1 rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{errors.map((error) => (
						<li key={error}>{error}</li>
					))}
				</ul>
			) : null}
		</div>
	)
}
