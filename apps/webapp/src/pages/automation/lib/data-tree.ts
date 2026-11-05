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
	const selected = new Set(eventNames)
	return events.find((event) => selected.has(event.name))?.payload_example
}
