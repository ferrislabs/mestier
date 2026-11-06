import {
	type ConnectorAppearance,
	connectorAppearance,
	triggerAppearance,
} from '#/pages/automation/lib/connector-appearance'
import type { RunNodeView } from '#/pages/automation/lib/run-view'
import { triggerKindOf } from '#/pages/automation/lib/trigger-events'

export function runNodeAppearance(
	node:
		| Pick<RunNodeView, 'kind' | 'trigger' | 'family' | 'connector'>
		| undefined,
): ConnectorAppearance {
	if (node?.kind === 'trigger')
		return triggerAppearance(
			node.trigger ? triggerKindOf(node.trigger.kind) : 'events',
		)
	return connectorAppearance(node?.family, node?.connector?.kind)
}
