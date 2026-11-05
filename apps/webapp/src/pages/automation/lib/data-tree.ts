import type { Schemas } from '#/api/api.client'

export interface AvailableConnectorData {
	id: string
	label: string
	output: unknown
}

export interface AvailableData {
	trigger: unknown
	connectors: AvailableConnectorData[]
}

export function toEvaluateContext(
	data: AvailableData,
): Schemas.EvaluateContextBody {
	return {
		trigger: data.trigger,
		connectors: Object.fromEntries(
			data.connectors.map((connector) => [
				connector.id,
				{ output: connector.output },
			]),
		),
		loop: null,
	}
}

export function resolveTriggerExample(
	events: readonly Schemas.EventDescriptorResponse[],
	eventNames: string[],
): unknown {
	for (const name of eventNames) {
		const event = events.find((candidate) => candidate.name === name)
		if (event) return event.payload_example
	}
	return undefined
}
