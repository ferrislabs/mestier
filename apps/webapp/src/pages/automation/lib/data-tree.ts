import type { Schemas } from '#/api/api.client'

export interface AvailableNodeData {
	label: string
	items: readonly unknown[]
}

export interface AvailableData {
	trigger: unknown
	nodes: AvailableNodeData[]
}

export function toEvaluateContext(
	data: AvailableData,
	position = 0,
	loop: Schemas.LoopFrameBody | null = null,
): Schemas.EvaluateContextBody {
	return {
		trigger: data.trigger,
		nodes: Object.fromEntries(
			data.nodes.map((node) => [node.label, [...node.items]]),
		),
		position,
		loop,
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
