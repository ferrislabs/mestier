import type { Schemas } from '#/api/api.client'
import { Badge } from '#/components/ui/badge'
import { formatWindowRange } from '#/pages/planning/lib/subtasks'
import { formatAssigneeNames } from '#/pages/planning/lib/task-list'
import {
	LabelPastilles,
	type LabelPastilleVM,
	STATUS_LABELS,
} from '#/pages/planning/ui/task-row-parts'

export interface TaskListSubtaskRowVM {
	id: string
	title: string
	status: Schemas.TaskStatus
	labels: LabelPastilleVM[]
	/** `null` only while the root task's own window hasn't loaded yet — a subtask's window is always resolvable once the root has. See `lib/subtasks.ts`'s `resolveDisplayWindow`. */
	window: { startsAt: string; endsAt: string; inherited: boolean } | null
	/**
	 * Governs how `window` renders (see `lib/subtasks.ts`'s
	 * `formatWindowRange`'s `allDay` option). Already resolved to the right
	 * task's flag by the feature layer — the subtask's own when `window` is
	 * its own, the root's when `window.inherited` is true (see
	 * `lib/task-list.ts`'s `resolveSubtaskAllDay`).
	 */
	allDay: boolean
	assigneeNames: string[]
}

export interface TaskListSubtaskRowsProps {
	subtasks: TaskListSubtaskRowVM[]
	isLoading: boolean
	error: string | null
	timeZone: string
	onOpenSubtask: (id: string) => void
}

/** Column count of the root table this fills in under — kept in sync by hand with `ui/task-list-ui.tsx`'s `<thead>`. */
const COLUMN_COUNT = 5

/**
 * A root task's subtasks, rendered as extra `<tr>`s directly under its own
 * row — pure presentation, mounted only once the feature layer's
 * `useSubtasks` fetch (gated on the row being expanded, see
 * `feature/task-list-feature.tsx`) has something to show. Never offers its
 * own expand control: the domain caps nesting at two levels, so a subtask
 * row never has children of its own (see `lib/subtasks.ts`'s
 * `canAddSubtask`).
 */
export function TaskListSubtaskRows({
	subtasks,
	isLoading,
	error,
	timeZone,
	onOpenSubtask,
}: TaskListSubtaskRowsProps) {
	if (isLoading) {
		return (
			<tr>
				<td
					colSpan={COLUMN_COUNT}
					className="px-5 py-3 text-center text-sm text-muted-foreground"
				>
					Chargement des sous-tâches…
				</td>
			</tr>
		)
	}

	if (error) {
		return (
			<tr>
				<td
					colSpan={COLUMN_COUNT}
					className="px-5 py-3 text-sm text-destructive"
				>
					{error}
				</td>
			</tr>
		)
	}

	if (subtasks.length === 0) {
		return (
			<tr>
				<td
					colSpan={COLUMN_COUNT}
					className="px-5 py-3 text-center text-sm text-muted-foreground"
				>
					Aucune sous-tâche.
				</td>
			</tr>
		)
	}

	return (
		<>
			{subtasks.map((subtask) => (
				<tr
					key={subtask.id}
					data-testid={`subtask-row-${subtask.id}`}
					className="cursor-pointer border-b bg-muted/20 text-sm hover:bg-accent/40"
					tabIndex={0}
					onClick={() => onOpenSubtask(subtask.id)}
					onKeyDown={(event) => {
						if (event.key === 'Enter' || event.key === ' ') {
							event.preventDefault()
							onOpenSubtask(subtask.id)
						}
					}}
				>
					<td className="py-2 pr-5 pl-12">{subtask.title}</td>
					<td className="px-5 py-2 align-middle">
						<LabelPastilles labels={subtask.labels} />
					</td>
					<td className="px-5 py-2 align-middle">
						{STATUS_LABELS[subtask.status]}
					</td>
					<td className="px-5 py-2 align-middle">
						{subtask.window ? (
							<span className="flex flex-wrap items-center gap-1.5">
								{formatWindowRange(subtask.window, timeZone, {
									allDay: subtask.allDay,
								})}
								{subtask.window.inherited ? (
									<Badge variant="outline" className="text-[10px]">
										Hérité
									</Badge>
								) : null}
							</span>
						) : (
							'—'
						)}
					</td>
					<td className="px-5 py-2 align-middle">
						{formatAssigneeNames(subtask.assigneeNames)}
					</td>
				</tr>
			))}
		</>
	)
}
