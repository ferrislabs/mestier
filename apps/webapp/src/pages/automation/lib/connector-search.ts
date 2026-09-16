import type { Schemas } from '#/api/api.client'

export interface ConnectorFamilyGroup {
	family: string
	connectors: Schemas.ConnectorDescriptorResponse[]
}

export function searchConnectors(
	connectors: Schemas.ConnectorDescriptorResponse[],
	query: string,
): ConnectorFamilyGroup[] {
	const normalized = query.trim().toLowerCase()
	const filtered = normalized
		? connectors.filter((connector) =>
				connector.label.toLowerCase().includes(normalized),
			)
		: connectors

	const families = new Map<string, Schemas.ConnectorDescriptorResponse[]>()
	for (const connector of filtered) {
		const group = families.get(connector.family) ?? []
		group.push(connector)
		families.set(connector.family, group)
	}

	return [...families.entries()]
		.sort(([a], [b]) => a.localeCompare(b))
		.map(([family, groupConnectors]) => ({
			family,
			connectors: groupConnectors,
		}))
}
