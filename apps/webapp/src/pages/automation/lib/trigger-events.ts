import type { Schemas } from '#/api/api.client'

export interface EventGroup {
	prefix: string
	events: Schemas.EventDescriptorResponse[]
}

export function groupEventsByPrefix(
	events: readonly Schemas.EventDescriptorResponse[],
): EventGroup[] {
	const groups = new Map<string, Schemas.EventDescriptorResponse[]>()

	for (const event of events) {
		const prefix = event.name.split('.')[0] ?? event.name
		const existing = groups.get(prefix) ?? []
		existing.push(event)
		groups.set(prefix, existing)
	}

	return [...groups.entries()]
		.sort(([a], [b]) => a.localeCompare(b))
		.map(([prefix, prefixEvents]) => ({ prefix, events: prefixEvents }))
}

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
		return { label: 'Clic', subtitle: 'Déclenchement manuel' }
	}
	const names = triggerEventNames(trigger)
	if (names.length === 0) {
		return {
			label: 'Event Mestier',
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
		label: 'Event Mestier',
		subtitle: `${names.length} événements`,
	}
}
