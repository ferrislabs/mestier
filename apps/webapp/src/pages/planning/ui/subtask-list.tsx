import { ChevronRight, Plus } from 'lucide-react'
import { Badge } from '#/components/ui/badge'
import { Button } from '#/components/ui/button'

export interface SubtaskListItem {
	id: string
	title: string
	status: 'PLANNED' | 'IN_PROGRESS' | 'DONE' | 'CANCELLED'
	assigneeCount: number
	/** Whether this subtask's window is its own or inherited from the parent — see `lib/subtasks.ts`'s `resolveDisplayWindow`. */
	inheritedWindow: boolean
}

export interface SubtaskListProps {
	subtasks: SubtaskListItem[]
	isLoading: boolean
	error: string | null
	/** `false` on a subtask itself — the two-level cap means the "add" action is never even offered on a subtask, not merely rejected server-side (see `lib/subtasks.ts`'s `canAddSubtask`). */
	canAddSubtask: boolean
	onAddSubtask: () => void
	onOpenSubtask: (id: string) => void
}

const STATUS_LABELS: Record<SubtaskListItem['status'], string> = {
	PLANNED: 'Planifiée',
	IN_PROGRESS: 'En cours',
	DONE: 'Terminée',
	CANCELLED: 'Annulée',
}

/**
 * A root task's subtasks — pure presentation. Never renders a nested "add
 * subtask" affordance on a subtask row: the domain caps nesting at two
 * levels, and this list only ever shows one level of children, never their
 * own children.
 */
export function SubtaskList({
	subtasks,
	isLoading,
	error,
	canAddSubtask,
	onAddSubtask,
	onOpenSubtask,
}: SubtaskListProps) {
	return (
		<div className="flex flex-col gap-2">
			{isLoading ? (
				<p className="py-6 text-center text-sm text-muted-foreground">
					Chargement des sous-tâches…
				</p>
			) : error ? (
				<p className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{error}
				</p>
			) : subtasks.length === 0 ? (
				<p className="py-6 text-center text-sm text-muted-foreground">
					Aucune sous-tâche pour l’instant.
				</p>
			) : (
				<ul className="flex flex-col gap-1.5">
					{subtasks.map((subtask) => (
						<li key={subtask.id} data-testid={`subtask-${subtask.id}`}>
							<button
								type="button"
								className="flex w-full items-center gap-2 rounded-lg border p-2.5 text-left text-sm hover:bg-accent"
								onClick={() => onOpenSubtask(subtask.id)}
							>
								<div className="min-w-0 flex-1">
									<p className="truncate font-medium">{subtask.title}</p>
									<div className="mt-0.5 flex flex-wrap items-center gap-1.5 text-xs text-muted-foreground">
										<span>{STATUS_LABELS[subtask.status]}</span>
										<span aria-hidden="true">·</span>
										<span>
											{subtask.assigneeCount === 0
												? 'Personne assigné'
												: `${subtask.assigneeCount} assigné${subtask.assigneeCount > 1 ? 's' : ''}`}
										</span>
										{subtask.inheritedWindow ? (
											<Badge variant="outline" className="text-[10px]">
												Dates héritées
											</Badge>
										) : null}
									</div>
								</div>
								<ChevronRight className="size-4 shrink-0 text-muted-foreground" />
							</button>
						</li>
					))}
				</ul>
			)}

			{canAddSubtask ? (
				<Button
					type="button"
					variant="outline"
					size="sm"
					className="mt-1 gap-1.5 self-start"
					onClick={onAddSubtask}
				>
					<Plus className="size-4" />
					Ajouter une sous-tâche
				</Button>
			) : null}
		</div>
	)
}
