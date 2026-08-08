import { Check, Users } from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import { cn } from '#/lib/utils'

export interface AssigneeOption {
	/** `<kind>:<uuid>`, verbatim `PlanningResourceResponse.resource_id` — see `lib/task-drop.ts`'s `assigneeRefFromResourceId`, which the feature uses to turn a selection back into an `AssigneeRef` for the payload. */
	resourceId: string
	displayName: string
}

export interface AssigneePickerProps {
	options: AssigneeOption[]
	selectedResourceIds: string[]
	onToggle: (resourceId: string) => void
}

/**
 * The assignee multi-select on the task form — pure presentation. A
 * subtask never pre-fills this from its parent's assignees (see the
 * planning remodel design doc's invariant 7); the feature is responsible
 * for that, this component only ever reflects `selectedResourceIds`
 * verbatim.
 */
export function AssigneePicker({
	options,
	selectedResourceIds,
	onToggle,
}: AssigneePickerProps) {
	const selected = options.filter((option) =>
		selectedResourceIds.includes(option.resourceId),
	)

	return (
		<Popover>
			<PopoverTrigger asChild>
				<Button
					type="button"
					variant="outline"
					className="h-auto min-h-9 w-full flex-wrap justify-start gap-1.5 py-1.5"
				>
					<Users className="size-4 shrink-0 text-muted-foreground" />
					{selected.length === 0 ? (
						<span className="text-muted-foreground">Personne assigné</span>
					) : (
						selected.map((option) => (
							<span
								key={option.resourceId}
								className="inline-flex items-center rounded-full bg-secondary px-2 py-0.5 text-xs font-medium text-secondary-foreground"
							>
								{option.displayName}
							</span>
						))
					)}
				</Button>
			</PopoverTrigger>
			<PopoverContent className="w-72 p-2" align="start">
				<div
					role="listbox"
					aria-label="Assignés"
					className="flex max-h-64 flex-col gap-0.5 overflow-y-auto"
				>
					{options.map((option) => {
						const isSelected = selectedResourceIds.includes(option.resourceId)
						return (
							<button
								key={option.resourceId}
								type="button"
								role="option"
								aria-selected={isSelected}
								onClick={() => onToggle(option.resourceId)}
								className={cn(
									'flex items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm hover:bg-accent',
									isSelected && 'bg-accent/60',
								)}
							>
								<span className="flex-1 truncate">{option.displayName}</span>
								{isSelected ? <Check className="size-4 shrink-0" /> : null}
							</button>
						)
					})}
					{options.length === 0 ? (
						<p className="px-2 py-1.5 text-sm text-muted-foreground">
							Aucune ressource disponible
						</p>
					) : null}
				</div>
			</PopoverContent>
		</Popover>
	)
}
