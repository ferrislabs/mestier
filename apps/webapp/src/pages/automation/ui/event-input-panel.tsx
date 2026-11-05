import type { Schemas } from '#/api/api.client'
import { Button } from '#/components/ui/button'
import { EventPayloadTabs } from '#/pages/automation/ui/event-payload-tabs'
import { NdvEmptyState, NdvPanel } from '#/pages/automation/ui/ndv-panel'

export function EventInputPanel({
	event,
	onChooseEvent,
}: {
	event: Schemas.EventDescriptorResponse | null
	onChooseEvent: () => void
}) {
	return (
		<NdvPanel title="Entrée" tone="side">
			{event ? (
				<div className="grid min-w-0 gap-3 [&>*]:min-w-0">
					<p className="text-muted-foreground text-sm">
						L’événement{' '}
						<span className="font-medium text-foreground">{event.label}</span>{' '}
						transmet ce payload au workflow.
					</p>
					<p className="font-mono text-muted-foreground text-xs">
						{event.name} · version {event.version}
					</p>
					<EventPayloadTabs event={event} />
				</div>
			) : (
				<div className="flex flex-col items-center">
					<NdvEmptyState
						title="Aucun événement choisi"
						text="Choisissez l’événement Mestier qui démarre ce workflow pour voir son payload."
					/>
					<Button variant="outline" size="sm" onClick={onChooseEvent}>
						Choisir l’événement
					</Button>
				</div>
			)}
		</NdvPanel>
	)
}
