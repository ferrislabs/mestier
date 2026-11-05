import type { Schemas } from '#/api/api.client'

export interface ParameterEntry {
	id: string
	label: string
	value: string
	template: unknown
	expression: boolean
}

export function displayValue(value: unknown): string {
	if (value === undefined || value === null) return ''
	return typeof value === 'string' ? value : JSON.stringify(value, null, 2)
}

export function isExpressionValue(value: unknown): boolean {
	return typeof value === 'string' && value.includes('{{')
}

export function parameterEntries(
	connector: Schemas.PlacedConnectorDto | null,
	descriptor: Schemas.ConnectorDescriptorResponse | undefined,
): ParameterEntry[] {
	if (!connector) return []
	const known = new Set<string>()
	const entries: ParameterEntry[] = []
	for (const field of descriptor?.fields ?? []) {
		known.add(field.name)
		if (field.secret) continue
		entries.push({
			id: field.name,
			label: field.label,
			value: displayValue(connector.config[field.name]),
			template: connector.config[field.name],
			expression: isExpressionValue(connector.config[field.name]),
		})
	}
	for (const [key, value] of Object.entries(connector.config)) {
		if (known.has(key)) continue
		entries.push({
			id: key,
			label: key,
			value: displayValue(value),
			template: value,
			expression: isExpressionValue(value),
		})
	}
	return entries
}

export interface NodeNeighbours {
	previous: { id: string; branch?: string }[]
	next: { id: string; branch?: string }[]
}

export function nodeNeighbours(
	graph: Schemas.GraphDto,
	nodeId: string,
): NodeNeighbours {
	return {
		previous: graph.edges
			.filter((edge) => edge.to === nodeId)
			.map((edge) => ({ id: edge.from, branch: edge.branch ?? undefined })),
		next: graph.edges
			.filter((edge) => edge.from === nodeId)
			.map((edge) => ({ id: edge.to, branch: edge.branch ?? undefined })),
	}
}
