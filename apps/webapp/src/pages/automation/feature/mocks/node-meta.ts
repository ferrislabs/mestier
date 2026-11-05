// MOCK: node names, descriptions, disabled flags and the collection operation a node was made from — no backend field yet. Replace with per-node `name`, `description`, `disabled` and an operation reference on the saved graph (PlacedConnectorDto / PlacedTriggerDto).
import type { NodeMetaMap } from '#/pages/automation/lib/editor-nodes'

function storageKey(workflowId: string): string {
	return `mestier:automation:node-meta:${workflowId}`
}

export function readNodeMeta(workflowId: string): NodeMetaMap {
	try {
		const raw = window.localStorage.getItem(storageKey(workflowId))
		if (!raw) return {}
		const parsed: unknown = JSON.parse(raw)
		return typeof parsed === 'object' &&
			parsed !== null &&
			!Array.isArray(parsed)
			? (parsed as NodeMetaMap)
			: {}
	} catch {
		return {}
	}
}

export function writeNodeMeta(workflowId: string, meta: NodeMetaMap): void {
	try {
		window.localStorage.setItem(storageKey(workflowId), JSON.stringify(meta))
	} catch {
		return
	}
}
