// MOCK: the collection operation a node was made from — no backend field yet. Names, descriptions and disabled flags live on the saved graph.
import type {
	NodeMetaMap,
	NodeOperationRef,
} from '#/pages/automation/lib/editor-nodes'

function storageKey(workflowId: string): string {
	return `mestier:automation:node-meta:${workflowId}`
}

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function operationOf(value: unknown): NodeOperationRef | null {
	if (!isRecord(value) || !isRecord(value.operation)) return null
	const { collectionId, operationId } = value.operation
	return typeof collectionId === 'string' && typeof operationId === 'string'
		? { collectionId, operationId }
		: null
}

export function readNodeMeta(workflowId: string): NodeMetaMap {
	try {
		const raw = window.localStorage.getItem(storageKey(workflowId))
		if (!raw) return {}
		const parsed: unknown = JSON.parse(raw)
		if (!isRecord(parsed)) return {}
		const meta: NodeMetaMap = {}
		for (const [nodeId, entry] of Object.entries(parsed)) {
			const operation = operationOf(entry)
			if (operation) meta[nodeId] = { operation }
		}
		return meta
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
