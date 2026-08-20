import type * as React from 'react'
import { useState } from 'react'
import type { Schemas } from '#/api/api.client'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import type { AbsenceFormValues } from '#/pages/hr/lib/absences'
import type { EventDetailVM } from '#/pages/planning/lib/build-calendar-model'
import type { TaskFormValues } from '#/pages/planning/lib/task-form'
import type { PlanningEntry } from '#/pages/planning/types'
import { EventDetailCard } from '#/pages/planning/ui/event-detail-card'
import type {
	EventAssigneeOption,
	EventEditState,
} from '#/pages/planning/ui/event-edit-form'

/**
 * What the calendar knows how to do with an entry — the rest lives in the
 * feature. The callbacks carry the entry, not the segment representing it:
 * that is what lets the week and month views, which position differently,
 * share the same panel.
 */
export interface CalendarEventCallbacks {
	onChangeStatus?: (entry: PlanningEntry, status: Schemas.TaskStatus) => void
	onDelete?: (entry: PlanningEntry) => void
	isPending?: boolean
	/** Draft in progress, if it is about this entry. */
	editing: EventEditState | null
	assignees: EventAssigneeOption[]
	selectedResourceIds: string[]
	onEdit: (entry: PlanningEntry) => void
	onEditChange: (patch: Partial<TaskFormValues & AbsenceFormValues>) => void
	onToggleAssignee: (resourceId: string) => void
	onEditSubmit: () => void
	onEditCancel: () => void
	/** Opens the same full `TaskSheetFeature` the Team and Task-list views use — task entries only. */
	onOpenDetail?: (entry: PlanningEntry) => void
	/**
	 * Surfaces a failed `onChangeStatus`/`onDelete` — see
	 * `feature/planning-calendar-feature.tsx`'s own doc. Scoped to the entry
	 * it happened on, the same way `editing` is (below): closing that entry's
	 * popover and opening a different one must not show a stale error about
	 * an action the newly opened entry never attempted.
	 */
	quickActionError?: { entryId: string; message: string } | null
}

export interface EventPopoverProps {
	detail: EventDetailVM
	callbacks: CalendarEventCallbacks
	children: React.ReactNode
}

/**
 * Detail panel for an entry, anchored on the clicked element.
 *
 * One click opens what the entry already holds — time, assignees, customer,
 * labels, description — instead of sending the user to a screen to read it.
 * The deeper actions stay behind "ouvrir".
 */
export function EventPopover({
	detail,
	callbacks,
	children,
}: EventPopoverProps) {
	const [open, setOpen] = useState(false)

	return (
		<Popover open={open} onOpenChange={setOpen}>
			<PopoverTrigger asChild>{children}</PopoverTrigger>
			<PopoverContent
				align="start"
				side="right"
				className="w-96 rounded-2xl p-5"
			>
				<EventDetailCard
					event={detail}
					isPending={callbacks.isPending}
					editing={
						callbacks.editing?.entryId === detail.entry.id
							? callbacks.editing
							: null
					}
					assignees={callbacks.assignees}
					selectedResourceIds={callbacks.selectedResourceIds}
					onEdit={() => callbacks.onEdit(detail.entry)}
					onEditChange={callbacks.onEditChange}
					onToggleAssignee={callbacks.onToggleAssignee}
					onEditSubmit={callbacks.onEditSubmit}
					onEditCancel={callbacks.onEditCancel}
					onClose={() => {
						callbacks.onEditCancel()
						setOpen(false)
					}}
					onChangeStatus={
						callbacks.onChangeStatus
							? (status) => callbacks.onChangeStatus?.(detail.entry, status)
							: undefined
					}
					// Does not close the popover itself, unlike `onOpenDetail`
					// below: a delete can fail, and closing eagerly would hide
					// `quickActionError` the instant it appears. On success the
					// entry disappears from `data.entries` once the planning
					// query is invalidated, unmounting this popover along with
					// it — closing it "for real" rather than pre-emptively.
					onDelete={
						callbacks.onDelete
							? () => callbacks.onDelete?.(detail.entry)
							: undefined
					}
					onOpenDetail={
						callbacks.onOpenDetail
							? () => {
									setOpen(false)
									callbacks.onOpenDetail?.(detail.entry)
								}
							: undefined
					}
					quickActionError={
						callbacks.quickActionError?.entryId === detail.entry.id
							? callbacks.quickActionError.message
							: null
					}
				/>
			</PopoverContent>
		</Popover>
	)
}
