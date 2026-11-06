import {
	type ConnectorAppearance,
	connectorAppearance,
	triggerAppearance,
} from '#/pages/automation/lib/connector-appearance'
import type { EditorNodeData } from '#/pages/automation/lib/editor-nodes'
import {
	isEventsTrigger,
	triggerEventNames,
	triggerKindOf,
} from '#/pages/automation/lib/trigger-events'

export function appearanceOf(data: EditorNodeData): ConnectorAppearance {
	return data.trigger
		? triggerAppearance(triggerKindOf(data.trigger.kind))
		: connectorAppearance(data.family, data.connector?.kind)
}

export function needsEventChoice(data: EditorNodeData): boolean {
	return (
		data.trigger !== undefined &&
		isEventsTrigger(data.trigger) &&
		triggerEventNames(data.trigger).length === 0
	)
}
