import {
	type ConnectorAppearance,
	connectorAppearance,
	triggerAppearance,
} from '#/pages/automation/lib/connector-appearance'
import type { EditorNodeData } from '#/pages/automation/lib/editor-nodes'
import {
	isManualTrigger,
	triggerEventNames,
} from '#/pages/automation/lib/trigger-events'

export function appearanceOf(data: EditorNodeData): ConnectorAppearance {
	return data.trigger
		? triggerAppearance(isManualTrigger(data.trigger))
		: connectorAppearance(data.family, data.connector?.kind)
}

export function needsEventChoice(data: EditorNodeData): boolean {
	return (
		data.trigger !== undefined &&
		!isManualTrigger(data.trigger) &&
		triggerEventNames(data.trigger).length === 0
	)
}
