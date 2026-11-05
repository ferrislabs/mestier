import { useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { Button } from '#/components/ui/button'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { EventPayloadTabs } from '#/pages/automation/ui/event-payload-tabs'
import { NdvEmptyState, NdvPanel } from '#/pages/automation/ui/ndv-panel'

export function EventInputPanel({
	events,
	onChooseEvent,
}: {
	events: readonly Schemas.EventDescriptorResponse[]
	onChooseEvent: () => void
}) {
	const [chosen, setChosen] = useState<string | null>(null)
	const event =
		events.find((candidate) => candidate.name === chosen) ?? events[0]
	return (
		<NdvPanel title="Entrée" tone="side">
			{event ? (
				<div className="grid min-w-0 gap-3 [&>*]:min-w-0">
					{events.length > 1 && (
						<Select value={event.name} onValueChange={setChosen}>
							<SelectTrigger className="w-full" aria-label="Événement affiché">
								<SelectValue />
							</SelectTrigger>
							<SelectContent>
								{events.map((candidate) => (
									<SelectItem key={candidate.name} value={candidate.name}>
										{candidate.label}
									</SelectItem>
								))}
							</SelectContent>
						</Select>
					)}
					<p className="text-muted-foreground text-sm">
						L’événement{' '}
						<span className="font-medium text-foreground">{event.label}</span>{' '}
						transmet ce payload au workflow.
					</p>
					<p className="font-mono text-muted-foreground text-xs">
						{event.name} · version {event.version}
					</p>
					<EventPayloadTabs key={event.name} event={event} />
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
