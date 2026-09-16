import type { Schemas } from '#/api/api.client'

export interface EventGroup {
	prefix: string
	events: Schemas.EventDescriptorResponse[]
}

export function groupEventsByPrefix(
	events: Schemas.EventDescriptorResponse[],
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
