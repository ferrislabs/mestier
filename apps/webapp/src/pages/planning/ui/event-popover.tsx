import type * as React from 'react'
import { useState } from 'react'
import type { Schemas } from '#/api/api.client'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import type { EventDetailVM } from '#/pages/planning/lib/build-calendar-model'
import type { PlanningEntry } from '#/pages/planning/types'
import { EventDetailCard } from '#/pages/planning/ui/event-detail-card'

/**
 * Ce que le calendrier sait faire d'une entrée — le reste vit dans la feature.
 * Les callbacks portent l'entrée, pas le segment qui la représente : c'est ce
 * qui permet aux vues semaine et mois, qui positionnent différemment, de
 * partager le même panneau.
 */
export interface CalendarEventCallbacks {
	/** Ouvre l'entrée dans son écran complet : fiche de tâche, formulaire d'absence. */
	onOpen: (entry: PlanningEntry) => void
	onChangeStatus?: (entry: PlanningEntry, status: Schemas.TaskStatus) => void
	onDelete?: (entry: PlanningEntry) => void
	isPending?: boolean
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
					onOpen={() => {
						setOpen(false)
						callbacks.onOpen(detail.entry)
					}}
					onClose={() => setOpen(false)}
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
