import {
	type ConnectorAppearance,
	connectorAppearance,
	triggerAppearance,
} from '#/pages/automation/lib/connector-appearance'
import type { RunNodeView } from '#/pages/automation/lib/run-view'

export function runNodeAppearance(
	node:
		| Pick<RunNodeView, 'kind' | 'trigger' | 'family' | 'connector'>
		| undefined,
): ConnectorAppearance {
	if (node?.kind === 'trigger')
		return triggerAppearance(node.trigger?.kind === 'Manual')
	return connectorAppearance(node?.family, node?.connector?.kind)
}
