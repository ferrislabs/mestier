import { X } from 'lucide-react'
import { useId } from 'react'
import type { Schemas } from '#/api/api.client'
import { Button } from '#/components/ui/button'
import { Label } from '#/components/ui/label'
import {
	findEvent,
	isManualTrigger,
	triggerEventNames,
} from '#/pages/automation/lib/trigger-events'

function EventCard({
	name,
	event,
	removable,
	onChange,
	onRemove,
}: {
	name: string
	event: Schemas.EventDescriptorResponse | undefined
	removable: boolean
	onChange: () => void
	onRemove: () => void
}) {
	return (
		<div className="flex items-center gap-3 rounded-md border border-border bg-card px-3 py-2">
			<span className="min-w-0 flex-1">
				<span className="block truncate font-medium text-sm">
					{event?.label ?? name}
				</span>
				<span className="block truncate font-mono text-muted-foreground text-xs">
					{event ? `${event.name} · version ${event.version}` : name}
				</span>
			</span>
			<Button
				type="button"
				aria-label={`Changer l’événement ${name}`}
				variant="outline"
				size="sm"
				onClick={onChange}
			>
				Changer
			</Button>
			{removable && (
				<Button
					type="button"
					aria-label={`Retirer l’événement ${name}`}
					variant="ghost"
					size="icon-sm"
					onClick={onRemove}
				>
					<X />
				</Button>
			)}
		</div>
	)
}

export function TriggerParameters({
	trigger,
	events,
	onChooseEvent,
	onRemoveEvent,
}: {
	trigger: Schemas.PlacedTriggerDto
	events: readonly Schemas.EventDescriptorResponse[]
	onChooseEvent: (replacing: string | null) => void
	onRemoveEvent: (name: string) => void
}) {
	const controlId = useId()
	if (isManualTrigger(trigger)) {
		return (
			<p className="text-muted-foreground text-sm">
				Ce déclencheur ne part que lorsqu’on exécute le workflow à la main,
				depuis « Tester le workflow ».
			</p>
		)
	}

	const names = triggerEventNames(trigger)
	return (
		<div className="grid gap-2">
			<Label htmlFor={controlId}>Événement</Label>
			<div id={controlId} className="grid gap-2">
				{names.length === 0 ? (
					<div className="flex items-center gap-3 rounded-md border border-border bg-card px-3 py-2">
						<span className="block min-w-0 flex-1 truncate font-medium text-sm">
							Aucun événement choisi
						</span>
						<Button
							type="button"
							aria-label="Choisir l’événement"
							variant="outline"
							size="sm"
							onClick={() => onChooseEvent(null)}
						>
							Choisir
						</Button>
					</div>
				) : (
					<>
						{names.map((name) => (
							<EventCard
								key={name}
								name={name}
								event={findEvent(events, name)}
								removable={names.length > 1}
								onChange={() => onChooseEvent(name)}
								onRemove={() => onRemoveEvent(name)}
							/>
						))}
						<div>
							<Button
								type="button"
								variant="outline"
								size="sm"
								onClick={() => onChooseEvent(null)}
							>
								Ajouter un événement
							</Button>
						</div>
					</>
				)}
			</div>
		</div>
	)
}
