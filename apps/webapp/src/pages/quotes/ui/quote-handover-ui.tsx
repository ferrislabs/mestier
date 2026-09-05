import { Link } from '@tanstack/react-router'
import { ArrowLeft, Loader2, Plus, Trash2 } from 'lucide-react'
import { RequirePermission } from '#/components/require-permission'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
} from '#/components/ui/surface'
import type { Quote, TaskProposal } from '#/hooks/use-quotes'
import { TIME_OPTIONS } from '#/pages/planning/lib/task-form'
import {
	type HandoverTaskDraft,
	validateHandoverTaskDraft,
} from '#/pages/quotes/lib/handover-task-form'

export interface QuoteHandoverUIProps {
	backTo: string
	quote: Quote
	proposal: TaskProposal[]
	projectName: string
	tasks: HandoverTaskDraft[]
	isPending: boolean
	error: string | null
	onProjectNameChange: (name: string) => void
	onAddTaskFromLine: (line: TaskProposal) => void
	onAddBlankTask: () => void
	onTaskChange: (index: number, patch: Partial<HandoverTaskDraft>) => void
	onRemoveTask: (index: number) => void
	onSubmit: () => void
}

/**
 * The quote handover screen: the quote's lines on one side, the tasks
 * being built on the other. Each line can become a task, several tasks, or
 * nothing — and a line left unmapped is shown as such at the end, not
 * hidden, because a silent gap is how a job gets underquoted on the ground.
 */
export function QuoteHandoverUI({
	backTo,
	quote,
	proposal,
	projectName,
	tasks,
	isPending,
	error,
	onProjectNameChange,
	onAddTaskFromLine,
	onAddBlankTask,
	onTaskChange,
	onRemoveTask,
	onSubmit,
}: QuoteHandoverUIProps) {
	const taskErrors = tasks.flatMap((task) => validateHandoverTaskDraft(task))
	const canSubmit =
		projectName.trim().length > 0 &&
		tasks.length > 0 &&
		taskErrors.length === 0 &&
		!isPending
	const unmappedLines = proposal.filter(
		(line) =>
			!tasks.some((task) => task.quoteLineIds.includes(line.quote_line_id)),
	)

	return (
		<PageShell>
			<PageHeader
				eyebrow={quote.title}
				title="Transformer le devis en projet"
				description="Le client et le devis du projet créé viennent automatiquement de ce devis accepté ; la marge affichée ensuite sera ce devis moins ce que ces tâches auront coûté."
				actions={
					<Button asChild variant="outline">
						<Link to={backTo}>
							<ArrowLeft />
							Retour au devis
						</Link>
					</Button>
				}
			/>

			<SectionCard>
				<div className="flex flex-col gap-2 p-5 sm:max-w-sm">
					<Label htmlFor="handover-project-name">Nom du projet</Label>
					<Input
						id="handover-project-name"
						value={projectName}
						onChange={(event) => onProjectNameChange(event.target.value)}
					/>
				</div>
			</SectionCard>

			{error ? (
				<div className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{error}
				</div>
			) : null}

			<div className="grid gap-5 lg:grid-cols-2">
				<SectionCard>
					<SectionHeader
						title={`Lignes du devis (${proposal.length})`}
						description="Une ligne au forfait ou à l'unité ne propose aucune durée, plutôt que d'en inventer une."
					/>
					<div className="divide-y">
						{proposal.map((line) => {
							const linked = tasks.filter((task) =>
								task.quoteLineIds.includes(line.quote_line_id),
							)
							return (
								<div key={line.quote_line_id} className="space-y-2 p-4">
									<div className="flex items-start justify-between gap-2">
										<div>
											<p className="font-medium">{line.title}</p>
											<p className="text-xs text-muted-foreground">
												{line.suggested_minutes !== null &&
												line.suggested_minutes !== undefined
													? `Durée suggérée : ${Math.round(line.suggested_minutes)} min`
													: 'Pas de durée suggérée — ligne au forfait ou à l’unité'}
											</p>
										</div>
										<Button
											type="button"
											variant="outline"
											size="sm"
											onClick={() => onAddTaskFromLine(line)}
										>
											<Plus />
											Tâche
										</Button>
									</div>
									{linked.length > 0 ? (
										<p className="text-xs text-muted-foreground">
											{linked.length} tâche{linked.length > 1 ? 's' : ''} liée
											{linked.length > 1 ? 's' : ''} :{' '}
											{linked
												.map((task) => task.title || 'Sans titre')
												.join(', ')}
										</p>
									) : null}
								</div>
							)
						})}
					</div>
				</SectionCard>

				<SectionCard>
					<SectionHeader
						title={`Tâches (${tasks.length})`}
						description="Le même formulaire que la planification, sans client ni assigné : on décide qui fait le travail plus tard."
						actions={
							<Button
								type="button"
								variant="outline"
								size="sm"
								onClick={onAddBlankTask}
							>
								<Plus />
								Tâche vide
							</Button>
						}
					/>
					<div className="space-y-3 p-4">
						{tasks.length === 0 ? (
							<p className="text-sm text-muted-foreground">
								Aucune tâche pour l’instant. Partez d’une ligne à gauche, ou
								ajoutez une tâche vide.
							</p>
						) : null}
						{tasks.map((task, index) => (
							<HandoverTaskCard
								key={task.clientKey}
								index={index}
								task={task}
								tasks={tasks}
								proposal={proposal}
								onChange={(patch) => onTaskChange(index, patch)}
								onRemove={() => onRemoveTask(index)}
							/>
						))}
					</div>
				</SectionCard>
			</div>

			{unmappedLines.length > 0 ? (
				<SectionCard>
					<SectionHeader
						title={`Lignes non affectées (${unmappedLines.length})`}
						description="Volontairement affichées plutôt que masquées : une ligne de fourniture n’est pas du travail, mais un oubli doit se voir."
					/>
					<ul className="space-y-1 p-4 text-sm">
						{unmappedLines.map((line) => (
							<li key={line.quote_line_id} className="text-muted-foreground">
								{line.title}
							</li>
						))}
					</ul>
				</SectionCard>
			) : null}

			<div className="flex justify-end">
				<RequirePermission permission="MANAGE_QUOTES">
					<Button type="button" disabled={!canSubmit} onClick={onSubmit}>
						{isPending ? <Loader2 className="animate-spin" /> : null}
						Créer le projet
					</Button>
				</RequirePermission>
			</div>
		</PageShell>
	)
}

function HandoverTaskCard({
	task,
	index,
	tasks,
	proposal,
	onChange,
	onRemove,
}: {
	task: HandoverTaskDraft
	index: number
	tasks: HandoverTaskDraft[]
	proposal: TaskProposal[]
	onChange: (patch: Partial<HandoverTaskDraft>) => void
	onRemove: () => void
}) {
	const errors = validateHandoverTaskDraft(task)
	const rootOptions = tasks
		.map((candidate, candidateIndex) => ({ candidate, candidateIndex }))
		.filter(
			({ candidate, candidateIndex }) =>
				candidateIndex !== index && candidate.parentIndex === null,
		)

	return (
		<div className="space-y-3 rounded-lg border bg-card p-4">
			<div className="flex items-start justify-between gap-2">
				<div className="flex-1 space-y-1">
					<Label>Titre</Label>
					<Input
						value={task.title}
						onChange={(event) => onChange({ title: event.target.value })}
					/>
				</div>
				<Button
					variant="ghost"
					size="icon"
					onClick={onRemove}
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
						onChange={(event) => onChange({ allDay: event.target.checked })}
					/>
					Journée entière
				</label>
				<label className="flex items-center gap-2 text-sm">
					<input
						type="checkbox"
						className="size-4"
						checked={task.blocksAvailability}
						onChange={(event) =>
							onChange({ blocksAvailability: event.target.checked })
						}
					/>
					Bloque la disponibilité
				</label>
			</div>

			<div className="grid gap-3 sm:grid-cols-2">
				<div className="space-y-1">
					<Label>Date de début</Label>
					<Input
						type="date"
						value={task.startDate}
						onChange={(event) => onChange({ startDate: event.target.value })}
					/>
				</div>
				<div className="space-y-1">
					<Label>Date de fin</Label>
					<Input
						type="date"
						value={task.endDate}
						onChange={(event) => onChange({ endDate: event.target.value })}
					/>
				</div>
			</div>

			{!task.allDay ? (
				<div className="grid gap-3 sm:grid-cols-2">
					<div className="space-y-1">
						<Label>Heure de début</Label>
						<select
							className="h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm"
							value={task.startTime}
							onChange={(event) => onChange({ startTime: event.target.value })}
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
							onChange={(event) => onChange({ endTime: event.target.value })}
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
							onChange({
								parentIndex:
									event.target.value === '' ? null : Number(event.target.value),
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
							onChange({ expensesEuros: event.target.value })
						}
						placeholder="0"
					/>
				</div>
				<div className="space-y-1">
					<Label>Motif des frais</Label>
					<Input
						value={task.expensesLabel}
						onChange={(event) =>
							onChange({ expensesLabel: event.target.value })
						}
					/>
				</div>
			</div>

			<div className="space-y-1">
				<Label>Lignes du devis couvertes</Label>
				<div className="flex flex-wrap gap-3">
					{proposal.map((line) => (
						<label
							key={line.quote_line_id}
							className="flex items-center gap-2 text-sm"
						>
							<input
								type="checkbox"
								className="size-4"
								checked={task.quoteLineIds.includes(line.quote_line_id)}
								onChange={(event) => {
									onChange({
										quoteLineIds: event.target.checked
											? [...task.quoteLineIds, line.quote_line_id]
											: task.quoteLineIds.filter(
													(id) => id !== line.quote_line_id,
												),
									})
								}}
							/>
							{line.title}
						</label>
					))}
				</div>
			</div>

			{errors.length > 0 ? (
				<p className="text-xs text-destructive">{errors.join(' · ')}</p>
			) : null}
		</div>
	)
}
