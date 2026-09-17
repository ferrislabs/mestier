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

export interface DataTreeNotice {
	kind: 'notice'
	key: string
	label: string
	path: string
	message: string
}

export type DataTreeNode = DataTreeLeaf | DataTreeBranch | DataTreeNotice

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
	const triggerNode: DataTreeNode =
		data.trigger === undefined || data.trigger === null
			? {
					kind: 'notice',
					key: 'trigger',
					label: 'trigger',
					path: 'trigger',
					message: 'Aucun événement déclencheur n’est configuré.',
				}
			: {
					kind: 'branch',
					key: 'trigger',
					label: 'trigger',
					path: 'trigger',
					children: valueChildren('trigger', data.trigger),
				}

	const connectorNodes: DataTreeNode[] = data.connectors.map((connector) => {
		const path = `connectors.${connector.id}.output`
		const label = `${connector.id} · ${connector.label}`
		const children = valueChildren(path, connector.output)

		if (children.length === 0) {
			return {
				kind: 'notice',
				key: path,
				label,
				path,
				message: 'Ce connecteur n’expose aucune donnée pour l’instant.',
			}
		}

		return { kind: 'branch', key: path, label, path, children }
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
