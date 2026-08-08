import type { Schemas } from '#/api/api.client'

export interface LabelPastilleVM {
	id: string
	name: string
	color: string
}

/**
 * A task's labels as small colored pastilles — the same visual language as
 * `LabelPicker`'s selected chips, but read-only: the list view's rows never
 * edit labels in place, editing goes through the reused `TaskSheet` (see
 * `feature/task-list-feature.tsx`).
 */
export function LabelPastilles({ labels }: { labels: LabelPastilleVM[] }) {
	if (labels.length === 0) {
		return <span className="text-muted-foreground">—</span>
	}

	return (
		<div className="flex flex-wrap gap-1">
			{labels.map((label) => (
				<span
					key={label.id}
					className="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium text-white"
					style={{ backgroundColor: label.color }}
				>
					{label.name}
				</span>
			))}
		</div>
	)
}

/** The one status label dictionary the task list view's rows share — root and subtask rows alike. */
export const STATUS_LABELS: Record<Schemas.TaskStatus, string> = {
	PLANNED: 'Planifiée',
	IN_PROGRESS: 'En cours',
	DONE: 'Terminée',
	CANCELLED: 'Annulée',
}
