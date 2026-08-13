import type {
	Connector,
	Credential,
	Graph,
	GraphEdge,
	GraphError,
	PlacedConnector,
} from '#/hooks/use-automation'

/**
 * Pure helpers over a `Graph` (`{ connectors, edges }`) — no React, no
 * fetch, no ids generated here. The canvas feature owns `crypto.randomUUID()`
 * (the convention already used for client-generated ids, see
 * `pages/hr/types.ts`) and calls these with the id already decided, so every
 * function here stays a plain, table-testable transform.
 *
 * Branch legality (which `BranchDto` an edge may carry) is deliberately not
 * decided here: the backend hardcodes it per connector *kind* and the
 * catalogue exposes no branch metadata to mirror. Duplicating that would
 * hardcode a connector kind client-side, which issue #204 explicitly rules
 * out. Instead every edge accepts every `BranchDto` (or none), and an
 * illegal combination comes back as a `GraphError` at save time — see
 * `graphErrorsByConnector`.
 */

export function connectorById(
	graph: Graph,
	connectorId: string,
): PlacedConnector | undefined {
	return graph.connectors.find((connector) => connector.id === connectorId)
}

/** The catalogue descriptor a placed connector was created from — `undefined`
 * once a connector kind is retired from the catalogue but a workflow still
 * references it. */
export function descriptorFor(
	connector: PlacedConnector,
	catalogue: Connector[],
): Connector | undefined {
	return catalogue.find((candidate) => candidate.kind === connector.kind)
}

export function groupByFamily(
	catalogue: Connector[],
): Map<string, Connector[]> {
	const groups = new Map<string, Connector[]>()
	for (const connector of catalogue) {
		const group = groups.get(connector.family)
		if (group) {
			group.push(connector)
		} else {
			groups.set(connector.family, [connector])
		}
	}
	return groups
}

/** Every connector with a direct or transitive edge into `connectorId` —
 * what the expression assistant offers as "available output" at that node.
 * Discovery order (each ancestor once, nearest first), not topological. */
export function upstreamOf(graph: Graph, connectorId: string): string[] {
	const seen = new Set<string>()
	const order: string[] = []
	const queue = [connectorId]

	while (queue.length > 0) {
		const current = queue.shift()
		if (current === undefined) break
		for (const edge of graph.edges) {
			if (edge.to !== current || seen.has(edge.from)) continue
			seen.add(edge.from)
			order.push(edge.from)
			queue.push(edge.from)
		}
	}

	return order
}

export function addConnector(graph: Graph, connector: PlacedConnector): Graph {
	return { ...graph, connectors: [...graph.connectors, connector] }
}

export function updateConnectorConfig(
	graph: Graph,
	connectorId: string,
	patch: Partial<Pick<PlacedConnector, 'config' | 'credential_id'>>,
): Graph {
	return {
		...graph,
		connectors: graph.connectors.map((connector) =>
			connector.id === connectorId ? { ...connector, ...patch } : connector,
		),
	}
}

/** Also drops every edge touching the connector — a dangling edge is not a
 * representable state. */
export function removeConnector(graph: Graph, connectorId: string): Graph {
	return {
		connectors: graph.connectors.filter(
			(connector) => connector.id !== connectorId,
		),
		edges: graph.edges.filter(
			(edge) => edge.from !== connectorId && edge.to !== connectorId,
		),
	}
}

/** No-op if the exact same `from`/`to` already exists — re-drawing an edge
 * the canvas already has is not a second edge. Changing only the `branch` of
 * an existing edge goes through `removeEdge` + `addEdge`, kept as two calls
 * so neither hides a silent branch change. */
export function addEdge(graph: Graph, edge: GraphEdge): Graph {
	const exists = graph.edges.some(
		(candidate) => candidate.from === edge.from && candidate.to === edge.to,
	)
	return exists ? graph : { ...graph, edges: [...graph.edges, edge] }
}

export function removeEdge(graph: Graph, from: string, to: string): Graph {
	return {
		...graph,
		edges: graph.edges.filter(
			(edge) => !(edge.from === from && edge.to === to),
		),
	}
}

/** Keyed by connector id, `null` for a graph-level error (no `connector_id`
 * on the wire). Field-level errors nest one level further so a connector's
 * config panel can hand its slice straight to `FieldForm`'s `errors` prop. */
export function graphErrorsByConnector(
	errors: GraphError[],
): Map<string | null, GraphError[]> {
	const grouped = new Map<string | null, GraphError[]>()
	for (const error of errors) {
		const key = error.connector_id ?? null
		const group = grouped.get(key)
		if (group) {
			group.push(error)
		} else {
			grouped.set(key, [error])
		}
	}
	return grouped
}

/** `FieldForm`'s `errors` prop shape for one connector — field-named errors
 * only; an error naming no field (`field: null`) is a connector-level
 * message and is not in this map. */
export function fieldErrorsFor(
	errors: GraphError[],
	connectorId: string,
): Record<string, string> {
	const fields: Record<string, string> = {}
	for (const error of errors) {
		if (error.connector_id === connectorId && error.field) {
			fields[error.field] = error.message
		}
	}
	return fields
}

/** `"None"` accepts nothing — a connector with no auth requirement never
 * offers a credential picker at all, so this is never asked about it in
 * practice. `Exactly` names the one kind that satisfies it; `AnyOf` names
 * several. */
export function matchesAuthRequirement(
	requirement: Connector['auth'],
	credentialKind: string,
): boolean {
	if (requirement === 'None') return false
	if ('Exactly' in requirement) return requirement.Exactly === credentialKind
	return requirement.AnyOf.includes(credentialKind)
}

/** The options a connector's own `credential_id` picker should offer —
 * every organization credential whose `kind` the connector's `auth`
 * accepts. This is the top-level connector credential, never the
 * `signing_credential_id` field-name exception (`field-form.tsx` handles
 * that one on its own, filtered to `origin: 'generated'`). */
export function credentialOptionsFor(
	connector: Connector,
	credentials: Credential[],
): Array<{ id: string; name: string }> {
	return credentials
		.filter((credential) =>
			matchesAuthRequirement(connector.auth, credential.kind),
		)
		.map((credential) => ({ id: credential.id, name: credential.name }))
}
