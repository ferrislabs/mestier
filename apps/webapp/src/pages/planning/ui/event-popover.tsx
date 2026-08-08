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
 * Ce que le calendrier sait faire d'une entrée — le reste vit dans la feature.
 * Les callbacks portent l'entrée, pas le segment qui la représente : c'est ce
 * qui permet aux vues semaine et mois, qui positionnent différemment, de
 * partager le même panneau.
 */
export interface CalendarEventCallbacks {
	onChangeStatus?: (entry: PlanningEntry, status: Schemas.TaskStatus) => void
	onDelete?: (entry: PlanningEntry) => void
	isPending?: boolean
	/** Brouillon en cours, s'il porte sur cette entrée. */
	editing: EventEditState | null
	assignees: EventAssigneeOption[]
	selectedResourceIds: string[]
	onEdit: (entry: PlanningEntry) => void
	onEditChange: (patch: Partial<TaskFormValues & AbsenceFormValues>) => void
	onToggleAssignee: (resourceId: string) => void
	onEditSubmit: () => void
	onEditCancel: () => void
}

export interface EventPopoverProps {
	detail: EventDetailVM
	callbacks: CalendarEventCallbacks
	children: React.ReactNode
}

/**
 * Panneau de détail d'une entrée, ancré sur l'élément cliqué.
 *
 * Un clic ouvre ce que l'entrée contient déjà — horaire, assignés, client,
 * étiquettes, description — au lieu d'envoyer vers un écran pour le lire. Les
 * actions de fond restent derrière « ouvrir ».
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
					onDelete={
						callbacks.onDelete
							? () => {
									setOpen(false)
									callbacks.onDelete?.(detail.entry)
								}
							: undefined
					}
				/>
			</PopoverContent>
		</Popover>
	)
}
