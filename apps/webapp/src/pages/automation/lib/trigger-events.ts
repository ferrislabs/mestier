import type { Schemas } from '#/api/api.client'
import { triggerCopy } from '#/pages/automation/lib/connector-copy'

export function triggerEventNames(trigger: Schemas.PlacedTriggerDto): string[] {
	return trigger.kind === 'Manual' ? [] : trigger.kind.Events
}

export function isManualTrigger(trigger: Schemas.PlacedTriggerDto): boolean {
	return trigger.kind === 'Manual'
}

export function findEvent(
	events: readonly Schemas.EventDescriptorResponse[],
	name: string,
): Schemas.EventDescriptorResponse | undefined {
	return events.find((event) => event.name === name)
}

export interface TriggerPresentation {
	label: string
	subtitle: string
}

export function triggerPresentation(
	trigger: Schemas.PlacedTriggerDto,
	events: readonly Schemas.EventDescriptorResponse[],
): TriggerPresentation {
	if (isManualTrigger(trigger)) {
		const { label, subtitle } = triggerCopy('manual')
		return { label, subtitle }
	}
	const eventsLabel = triggerCopy('events').label
	const names = triggerEventNames(trigger)
	if (names.length === 0) {
		return {
			label: eventsLabel,
			subtitle: 'Cliquez pour choisir l’événement',
		}
	}
	if (names.length === 1) {
		const name = names[0]
		return {
			label: findEvent(events, name)?.label ?? name,
			subtitle: `Event Mestier · ${name}`,
		}
	}
	return {
		label: eventsLabel,
		subtitle: `${names.length} événements`,
	}
}
