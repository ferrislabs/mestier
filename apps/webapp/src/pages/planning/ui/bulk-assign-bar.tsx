import { X } from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	type AssigneeOption,
	AssigneePicker,
} from '#/pages/planning/ui/assignee-picker'

export interface BulkAssignBarProps {
	selectedCount: number
	assigneeOptions: AssigneeOption[]
	draftResourceIds: string[]
	onToggleDraftAssignee: (resourceId: string) => void
	onApply: () => void
	onCancel: () => void
	isApplying: boolean
	error: string | null
}

/**
 * Shown above the task list table once at least one row is selected — pure
 * presentation, like every other `ui/` component. `feature/task-list-feature.tsx`
 * owns the selection set, the draft assignee picks and the mutation; this
 * only ever reflects the props handed down, mirroring `AssigneePicker`
 * itself just below in the tree.
 */
export function BulkAssignBar({
	selectedCount,
	assigneeOptions,
	draftResourceIds,
	onToggleDraftAssignee,
	onApply,
	onCancel,
	isApplying,
	error,
}: BulkAssignBarProps) {
	return (
		<div className="flex flex-col gap-2 border-b bg-brand-soft/40 px-5 py-3">
			<div className="flex flex-wrap items-center gap-3">
				<p className="text-sm font-medium">
					{selectedCount} tâche{selectedCount > 1 ? 's' : ''} sélectionnée
					{selectedCount > 1 ? 's' : ''}
				</p>
				<div className="w-64">
					<AssigneePicker
						options={assigneeOptions}
						selectedResourceIds={draftResourceIds}
						onToggle={onToggleDraftAssignee}
					/>
				</div>
				<Button
					type="button"
					size="sm"
					disabled={draftResourceIds.length === 0 || isApplying}
					onClick={onApply}
				>
					Assigner
				</Button>
				<Button type="button" variant="ghost" size="sm" onClick={onCancel}>
					<X className="size-4" />
					Annuler
				</Button>
			</div>
			{error ? <p className="text-sm text-destructive">{error}</p> : null}
		</div>
	)
}
