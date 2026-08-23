import { Plus, Trash2 } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import { TIME_OPTIONS } from '#/pages/planning/lib/task-form'
import {
	dayOffsetLabel,
	emptyTemplateTaskDraft,
	type TemplateTaskDraft,
	validateTemplateTaskDraft,
} from '#/pages/project-templates/lib/template-task-form'

export interface TemplateTaskShapeEditorProps {
	tasks: TemplateTaskDraft[]
	onChange: (tasks: TemplateTaskDraft[]) => void
}

/**
 * The task-shape builder — the task form's own shape, minus assignees and
 * absolute dates: a title, an offset in days, hours or all-day, expenses,
 * and an optional parent among the other rows. Pure presentation over the
 * array its caller owns; every edit replaces the whole array, mirroring how
 * the rest of this codebase treats "the complete list" fields.
 */
export function TemplateTaskShapeEditor({
	tasks,
	onChange,
}: TemplateTaskShapeEditorProps) {
	const updateAt = (index: number, patch: Partial<TemplateTaskDraft>) => {
		onChange(
			tasks.map((task, taskIndex) =>
				taskIndex === index ? { ...task, ...patch } : task,
			),
		)
	}

	const removeAt = (index: number) => {
		// Any row pointing at the removed one, or at a row that shifts index,
		// would silently end up pointing at the wrong parent — clear those
		// links rather than let that happen invisibly.
		const next = tasks
			.filter((_, taskIndex) => taskIndex !== index)
			.map((task) => {
				if (task.parentIndex === null) return task
				if (task.parentIndex === index) return { ...task, parentIndex: null }
				return {
					...task,
					parentIndex:
						task.parentIndex > index ? task.parentIndex - 1 : task.parentIndex,
				}
			})
		onChange(next)
	}

	return (
		<div className="space-y-3">
			{tasks.map((task, index) => {
				const errors = validateTemplateTaskDraft(task)
				const rootOptions = tasks
					.map((candidate, candidateIndex) => ({ candidate, candidateIndex }))
					.filter(
						({ candidate, candidateIndex }) =>
							candidateIndex !== index && candidate.parentIndex === null,
					)

				return (
					// biome-ignore lint/suspicious/noArrayIndexKey: a draft row has no id of its own until saved, and `parentIndex` on other rows is itself position-based, so reordering is not a case this editor supports — order is stable within a render.
					<div key={index} className="space-y-3 rounded-lg border bg-card p-4">
						<div className="flex items-start justify-between gap-2">
							<div className="grid flex-1 gap-3 sm:grid-cols-2">
								<div className="space-y-1">
									<Label>Titre</Label>
									<Input
										value={task.title}
										onChange={(event) =>
											updateAt(index, { title: event.target.value })
										}
										placeholder="Préparer le chantier"
									/>
								</div>
								<div className="space-y-1">
									<Label>Décalage ({dayOffsetLabel(task.dayOffset)})</Label>
									<Input
										type="number"
										value={task.dayOffset}
										onChange={(event) =>
											updateAt(index, {
												dayOffset: Number(event.target.value) || 0,
											})
										}
									/>
								</div>
							</div>
							<Button
								variant="ghost"
								size="icon"
								onClick={() => removeAt(index)}
								aria-label="Supprimer cette tâche"
							>
								<Trash2 className="size-4" />
							</Button>
						</div>

						<div className="flex flex-wrap items-center gap-4">
							<label className="flex items-center gap-2 text-sm">
								<input
									type="checkbox"
									className="size-4"
									checked={task.allDay}
									onChange={(event) =>
										updateAt(index, { allDay: event.target.checked })
									}
								/>
								Journée entière
							</label>
							<label className="flex items-center gap-2 text-sm">
								<input
									type="checkbox"
									className="size-4"
									checked={task.blocksAvailability}
									onChange={(event) =>
										updateAt(index, {
											blocksAvailability: event.target.checked,
										})
									}
								/>
								Bloque la disponibilité
							</label>
						</div>

						{!task.allDay ? (
							<div className="grid gap-3 sm:grid-cols-2">
								<div className="space-y-1">
									<Label>Heure de début</Label>
									<select
										className="h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm"
										value={task.startTime}
										onChange={(event) =>
											updateAt(index, { startTime: event.target.value })
										}
									>
										{TIME_OPTIONS.map((time) => (
											<option key={time} value={time}>
												{time}
											</option>
										))}
									</select>
								</div>
								<div className="space-y-1">
									<Label>Heure de fin</Label>
									<select
										className="h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm"
										value={task.endTime}
										onChange={(event) =>
											updateAt(index, { endTime: event.target.value })
										}
									>
										{TIME_OPTIONS.map((time) => (
											<option key={time} value={time}>
												{time}
											</option>
										))}
									</select>
								</div>
							</div>
						) : null}

						<div className="grid gap-3 sm:grid-cols-3">
							<div className="space-y-1">
								<Label>Sous-tâche de</Label>
								<select
									className="h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm"
									value={task.parentIndex ?? ''}
									onChange={(event) =>
										updateAt(index, {
											parentIndex:
												event.target.value === ''
													? null
													: Number(event.target.value),
										})
									}
								>
									<option value="">Aucune — tâche racine</option>
									{rootOptions.map(({ candidate, candidateIndex }) => (
										<option key={candidateIndex} value={candidateIndex}>
											{candidate.title || `Tâche ${candidateIndex + 1}`}
										</option>
									))}
								</select>
							</div>
							<div className="space-y-1">
								<Label>Frais (€)</Label>
								<Input
									value={task.expensesEuros}
									onChange={(event) =>
										updateAt(index, { expensesEuros: event.target.value })
									}
									placeholder="0"
								/>
							</div>
							<div className="space-y-1">
								<Label>Motif des frais</Label>
								<Input
									value={task.expensesLabel}
									onChange={(event) =>
										updateAt(index, { expensesLabel: event.target.value })
									}
									placeholder="Location compacteur"
								/>
							</div>
						</div>

						{errors.length > 0 ? (
							<p className="text-xs text-destructive">{errors.join(' · ')}</p>
						) : null}
					</div>
				)
			})}

			<Button
				variant="outline"
				onClick={() => onChange([...tasks, emptyTemplateTaskDraft()])}
			>
				<Plus />
				Ajouter une tâche
			</Button>
		</div>
	)
}
