import type { Schemas } from '#/api/api.client'

export interface DataTreeLeaf {
	kind: 'leaf'
	key: string
	label: string
	path: string
	value: unknown
}

export interface DataTreeBranch {
	kind: 'branch'
	key: string
	label: string
	path: string
	children: DataTreeNode[]
}

export type DataTreeNode = DataTreeLeaf | DataTreeBranch

export interface AvailableConnectorData {
	id: string
	label: string
	output: unknown
}

export interface AvailableData {
	trigger: unknown
	connectors: AvailableConnectorData[]
}

function isPlainObject(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function buildNode(
	path: string,
	key: string,
	label: string,
	value: unknown,
): DataTreeNode {
	if (Array.isArray(value)) {
		return {
			kind: 'branch',
			key,
			label,
			path,
			children: value.map((item, index) =>
				buildNode(`${path}[${index}]`, String(index), String(index), item),
			),
		}
	}

	if (isPlainObject(value)) {
		return {
			kind: 'branch',
			key,
			label,
			path,
			children: Object.entries(value).map(([childKey, childValue]) =>
				buildNode(`${path}.${childKey}`, childKey, childKey, childValue),
			),
		}
	}

	return { kind: 'leaf', key, label, path, value }
}

function valueChildren(path: string, value: unknown): DataTreeNode[] {
	const node = buildNode(path, '', '', value)
	return node.kind === 'branch' ? node.children : []
}

export function buildAvailableDataTree(data: AvailableData): DataTreeNode[] {
	const triggerNode: DataTreeBranch = {
		kind: 'branch',
		key: 'trigger',
		label: 'trigger',
		path: 'trigger',
		children: valueChildren('trigger', data.trigger),
	}

	const connectorNodes: DataTreeBranch[] = data.connectors.map((connector) => {
		const path = `connectors.${connector.id}.output`
		return {
			kind: 'branch',
			key: path,
			label: `${connector.id} · ${connector.label}`,
			path,
			children: valueChildren(path, connector.output),
		}
	})

	return [triggerNode, ...connectorNodes]
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
	events: Schemas.EventDescriptorResponse[],
	eventNames: string[],
): unknown {
	const selected = new Set(eventNames)
	return events.find((event) => selected.has(event.name))?.payload_example
}
