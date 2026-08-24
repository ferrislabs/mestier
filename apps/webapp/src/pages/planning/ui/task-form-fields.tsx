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
import type {
	RecurrenceDraft,
	RecurrenceFrequency,
	TaskFormValues,
} from '#/pages/planning/lib/task-form'
import {
	type AssigneeOption,
	AssigneePicker,
} from '#/pages/planning/ui/assignee-picker'
import {
	LabelPicker,
	type LabelPickerOption,
} from '#/pages/planning/ui/label-picker'
import { TaskWindowFields } from '#/pages/planning/ui/task-window-fields'

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
	/**
	 * Create mode only, and only the accepted quotes of the chosen customer: a
	 * projet bills against a quote the customer agreed to, and offering a
	 * draft would invite a margin computed from a number nobody signed.
	 */
	quotes: { id: string; label: string }[]
	isQuotesLoading?: boolean
	/**
	 * Both modes, unlike the customer: `PATCH /tasks/{id}` carries `project_id`,
	 * and it is the only way a task joins or leaves a project. Internal projects
	 * are in this list too — that is what lets a meeting be costed.
	 */
	projects: { id: string; label: string; isInternal: boolean }[]
	isProjectsLoading?: boolean
	labels: LabelPickerOption[]
	isCreatingLabel?: boolean
	onCreateLabel: (name: string) => void
	assigneeOptions: AssigneeOption[]
	/**
	 * Edit mode only: whether the loaded task still follows a series —
	 * drives the detach notice, shown *before* saving so the screen can
	 * explain up front that this edit will not touch next Tuesday's
	 * occurrence, not after the fact.
	 */
	isPartOfSeries?: boolean
}

const WEEKDAY_OPTIONS: { iso: number; label: string }[] = [
	{ iso: 1, label: 'Lun' },
	{ iso: 2, label: 'Mar' },
	{ iso: 3, label: 'Mer' },
	{ iso: 4, label: 'Jeu' },
	{ iso: 5, label: 'Ven' },
	{ iso: 6, label: 'Sam' },
	{ iso: 7, label: 'Dim' },
]

const RECURRENCE_FREQUENCY_LABELS: Record<RecurrenceFrequency, string> = {
	DAILY: 'Tous les jours',
	WEEKLY: 'Toutes les semaines',
	MONTHLY: 'Tous les mois',
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
	quotes,
	isQuotesLoading,
	projects,
	isProjectsLoading,
	labels,
	isCreatingLabel,
	onCreateLabel,
	assigneeOptions,
	isPartOfSeries,
}: TaskFormFieldsProps) {
	const selectedResourceIds = values.assignees.map(resourceIdFromAssigneeRef)

	return (
		<div className="flex flex-col gap-5">
			{mode === 'edit' && isPartOfSeries ? (
				<p className="rounded-lg border border-amber-300 bg-amber-50 px-4 py-3 text-sm text-amber-900">
					Cette tâche fait partie d'une série. Modifier ce formulaire ne change
					que cette occurrence : elle cessera de suivre la série, et les autres
					occurrences resteront inchangées.
				</p>
			) : null}

			<div className="flex flex-col gap-2">
				<Label htmlFor="task-title">Titre</Label>
				<Input
					id="task-title"
					value={values.title}
					onChange={(event) => onChange({ title: event.target.value })}
					placeholder="Réunion de projet, livraison, formation…"
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

					{values.customerId ? (
						<div className="flex flex-col gap-2">
							<Label htmlFor="task-quote">Devis</Label>
							<Select
								value={values.quoteId || NO_SELECTION}
								onValueChange={(quoteId) =>
									onChange({
										quoteId: quoteId === NO_SELECTION ? '' : quoteId,
									})
								}
								disabled={isQuotesLoading}
							>
								<SelectTrigger
									id="task-quote"
									aria-label="Devis"
									className="w-full"
								>
									<SelectValue placeholder="Aucun devis" />
								</SelectTrigger>
								<SelectContent>
									<SelectItem value={NO_SELECTION}>Aucun devis</SelectItem>
									{quotes.map((quote) => (
										<SelectItem key={quote.id} value={quote.id}>
											{quote.label}
										</SelectItem>
									))}
								</SelectContent>
							</Select>
							<p className="text-xs text-muted-foreground">
								Sans devis rattaché, la rentabilité de ce projet affichera son
								coût mais aucune marge.
							</p>
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

			<div className="flex flex-col gap-2">
				<Label htmlFor="task-project">Projet</Label>
				<Select
					value={values.projectId || NO_SELECTION}
					onValueChange={(projectId) =>
						onChange({
							projectId: projectId === NO_SELECTION ? '' : projectId,
						})
					}
					disabled={isProjectsLoading}
				>
					<SelectTrigger
						id="task-project"
						aria-label="Projet"
						className="w-full"
					>
						<SelectValue placeholder="Aucun projet" />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value={NO_SELECTION}>Aucun projet</SelectItem>
						{projects.map((project) => (
							<SelectItem key={project.id} value={project.id}>
								{project.isInternal
									? `${project.label} (interne)`
									: project.label}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
				<p className="text-xs text-muted-foreground">
					Sans projet, le temps de cette tâche compte pour la personne mais
					n'est rattaché à aucun sujet.
				</p>
			</div>

			<div className="flex flex-col gap-2">
				<Label htmlFor="task-expenses">Frais</Label>
				<div className="flex flex-col gap-2 sm:flex-row">
					<Input
						id="task-expenses"
						inputMode="decimal"
						placeholder="0,00"
						className="sm:w-32"
						value={values.expensesEuros}
						onChange={(event) =>
							onChange({ expensesEuros: event.target.value })
						}
					/>
					<Input
						aria-label="Motif des frais"
						placeholder="Déplacement, benne, repas…"
						value={values.expensesLabel}
						onChange={(event) =>
							onChange({ expensesLabel: event.target.value })
						}
						disabled={values.expensesEuros.trim() === ''}
					/>
				</div>
				<p className="text-xs text-muted-foreground">
					En euros. Un montant doit être justifié : sans motif, il serait
					impossible à retrouver dans trois mois.
				</p>
			</div>

			<div className="flex items-center justify-between rounded-lg border bg-card p-3">
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

			<TaskWindowFields
				values={values}
				isSubtask={isSubtask}
				windowPlaceholder={windowPlaceholder}
				onChange={onChange}
			/>

			<div className="flex items-center justify-between rounded-lg border bg-card p-3">
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

			{mode === 'create' && !isSubtask ? (
				<RecurrenceFields
					recurrence={values.recurrence}
					onChange={(recurrence) => onChange({ recurrence })}
				/>
			) : null}

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

/**
 * The "repeat" control — create mode, root tasks only (see
 * `TaskFormFieldsProps.isPartOfSeries`'s own doc: an occurrence's edit form
 * never renders this, it renders the detach notice instead). Daily, weekly
 * on a set of weekdays, or monthly on a day number, with an optional end
 * date — mirrors the backend's `RecurrenceRule` exactly: an RRULE string was
 * rejected there for the same reason this is three concrete shapes rather
 * than one free-form field.
 */
function RecurrenceFields({
	recurrence,
	onChange,
}: {
	recurrence: RecurrenceDraft
	onChange: (recurrence: RecurrenceDraft) => void
}) {
	return (
		<div className="flex flex-col gap-3 rounded-lg border bg-card p-3">
			<div className="flex items-center justify-between">
				<div>
					<p className="text-sm font-medium">Se répète</p>
					<p className="text-xs text-muted-foreground">
						Crée automatiquement les prochaines occurrences.
					</p>
				</div>
				<Switch
					checked={recurrence.enabled}
					onCheckedChange={(enabled) => onChange({ ...recurrence, enabled })}
					aria-label="Se répète"
				/>
			</div>

			{recurrence.enabled ? (
				<div className="flex flex-col gap-3">
					<div className="flex flex-col gap-2">
						<Label htmlFor="task-recurrence-frequency">Fréquence</Label>
						<Select
							value={recurrence.frequency}
							onValueChange={(frequency) =>
								onChange({
									...recurrence,
									frequency: frequency as RecurrenceFrequency,
								})
							}
						>
							<SelectTrigger
								id="task-recurrence-frequency"
								aria-label="Fréquence"
								className="w-full"
							>
								<SelectValue />
							</SelectTrigger>
							<SelectContent>
								{(
									Object.keys(
										RECURRENCE_FREQUENCY_LABELS,
									) as RecurrenceFrequency[]
								).map((frequency) => (
									<SelectItem key={frequency} value={frequency}>
										{RECURRENCE_FREQUENCY_LABELS[frequency]}
									</SelectItem>
								))}
							</SelectContent>
						</Select>
					</div>

					{recurrence.frequency === 'WEEKLY' ? (
						<div className="flex flex-col gap-2">
							<Label>Jours de la semaine</Label>
							<div className="flex flex-wrap gap-1.5">
								{WEEKDAY_OPTIONS.map((weekday) => {
									const selected = recurrence.weekdays.includes(weekday.iso)
									return (
										<button
											key={weekday.iso}
											type="button"
											aria-pressed={selected}
											onClick={() =>
												onChange({
													...recurrence,
													weekdays: selected
														? recurrence.weekdays.filter(
																(iso) => iso !== weekday.iso,
															)
														: [...recurrence.weekdays, weekday.iso].sort(),
												})
											}
											className={`rounded-md border px-2.5 py-1 text-xs font-medium transition-colors ${
												selected
													? 'border-primary bg-primary text-primary-foreground'
													: 'border-input bg-background text-foreground'
											}`}
										>
											{weekday.label}
										</button>
									)
								})}
							</div>
						</div>
					) : null}

					{recurrence.frequency === 'MONTHLY' ? (
						<div className="flex flex-col gap-2">
							<Label htmlFor="task-recurrence-day-of-month">Jour du mois</Label>
							<Input
								id="task-recurrence-day-of-month"
								type="number"
								min={1}
								max={31}
								className="w-24"
								value={recurrence.dayOfMonth}
								onChange={(event) =>
									onChange({
										...recurrence,
										dayOfMonth: Number(event.target.value) || 1,
									})
								}
							/>
						</div>
					) : null}

					<div className="flex flex-col gap-2">
						<Label htmlFor="task-recurrence-ends-on">
							Fin de la répétition
						</Label>
						<Input
							id="task-recurrence-ends-on"
							type="date"
							value={recurrence.endsOn}
							onChange={(event) =>
								onChange({ ...recurrence, endsOn: event.target.value })
							}
						/>
						<p className="text-xs text-muted-foreground">
							Laissez vide pour une répétition sans fin.
						</p>
					</div>
				</div>
			) : null}
		</div>
	)
}
