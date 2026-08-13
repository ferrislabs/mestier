import { describe, expect, it } from 'vitest'
import type {
	Connector,
	Credential,
	Graph,
	PlacedConnector,
} from '#/hooks/use-automation'
import {
	addConnector,
	addEdge,
	connectorById,
	credentialOptionsFor,
	descriptorFor,
	fieldErrorsFor,
	graphErrorsByConnector,
	groupByFamily,
	matchesAuthRequirement,
	removeConnector,
	removeEdge,
	updateConnectorConfig,
	upstreamOf,
} from '#/pages/automation/lib/graph'

function placed(overrides: Partial<PlacedConnector> = {}): PlacedConnector {
	return {
		id: 'c1',
		kind: 'http.request',
		version: 1,
		config: {},
		...overrides,
	}
}

function descriptor(overrides: Partial<Connector> = {}): Connector {
	return {
		kind: 'http.request',
		label: 'Requête HTTP',
		family: 'http',
		version: 1,
		auth: 'None',
		fields: [],
		output_example: null,
		...overrides,
	}
}

function emptyGraph(): Graph {
	return { connectors: [], edges: [] }
}

describe('connectorById / descriptorFor', () => {
	it('finds a placed connector by id', () => {
		const graph: Graph = { connectors: [placed()], edges: [] }
		expect(connectorById(graph, 'c1')?.id).toBe('c1')
		expect(connectorById(graph, 'missing')).toBeUndefined()
	})

	it('resolves a placed connector to its catalogue descriptor by kind', () => {
		const catalogue = [descriptor(), descriptor({ kind: 'odoo.upsert' })]
		expect(descriptorFor(placed(), catalogue)?.kind).toBe('http.request')
	})

	it('is undefined once a kind is retired from the catalogue', () => {
		expect(
			descriptorFor(placed({ kind: 'gone' }), [descriptor()]),
		).toBeUndefined()
	})
})

describe('groupByFamily', () => {
	it('groups catalogue connectors by family, preserving catalogue order within a group', () => {
		const catalogue = [
			descriptor({ kind: 'http.get', family: 'http' }),
			descriptor({ kind: 'odoo.upsert', family: 'odoo' }),
			descriptor({ kind: 'http.post', family: 'http' }),
		]

		const groups = groupByFamily(catalogue)

		expect([...groups.keys()]).toEqual(['http', 'odoo'])
		expect(groups.get('http')?.map((c) => c.kind)).toEqual([
			'http.get',
			'http.post',
		])
	})
})

describe('upstreamOf', () => {
	it('has no ancestors for a connector nothing feeds into', () => {
		const graph: Graph = {
			connectors: [placed({ id: 'a' })],
			edges: [],
		}
		expect(upstreamOf(graph, 'a')).toEqual([])
	})

	it('walks transitively through the whole ancestor chain', () => {
		const graph: Graph = {
			connectors: [],
			edges: [
				{ from: 'a', to: 'b' },
				{ from: 'b', to: 'c' },
			],
		}
		expect(upstreamOf(graph, 'c')).toEqual(['b', 'a'])
	})

	it('never revisits a node reachable through two paths', () => {
		const graph: Graph = {
			connectors: [],
			edges: [
				{ from: 'a', to: 'b' },
				{ from: 'a', to: 'c' },
				{ from: 'b', to: 'd' },
				{ from: 'c', to: 'd' },
			],
		}
		expect(upstreamOf(graph, 'd').sort()).toEqual(['a', 'b', 'c'])
	})
})

describe('addConnector / removeConnector', () => {
	it('appends a connector', () => {
		const graph = addConnector(emptyGraph(), placed())
		expect(graph.connectors).toHaveLength(1)
	})

	it('removing a connector also drops every edge touching it', () => {
		const graph: Graph = {
			connectors: [placed({ id: 'a' }), placed({ id: 'b' })],
			edges: [{ from: 'a', to: 'b' }],
		}

		const next = removeConnector(graph, 'a')

		expect(next.connectors.map((c) => c.id)).toEqual(['b'])
		expect(next.edges).toEqual([])
	})
})

describe('updateConnectorConfig', () => {
	it('patches only the targeted connector, leaving others untouched', () => {
		const graph: Graph = {
			connectors: [placed({ id: 'a', config: { x: 1 } }), placed({ id: 'b' })],
			edges: [],
		}

		const next = updateConnectorConfig(graph, 'a', { config: { x: 2 } })

		expect(connectorById(next, 'a')?.config).toEqual({ x: 2 })
		expect(connectorById(next, 'b')).toEqual(placed({ id: 'b' }))
	})
})

describe('addEdge / removeEdge', () => {
	it('adds an edge with its branch', () => {
		const graph = addEdge(emptyGraph(), { from: 'a', to: 'b', branch: 'Then' })
		expect(graph.edges).toEqual([{ from: 'a', to: 'b', branch: 'Then' }])
	})

	it('re-adding the same from/to is a no-op, not a duplicate edge', () => {
		const graph: Graph = {
			connectors: [],
			edges: [{ from: 'a', to: 'b' }],
		}

		const next = addEdge(graph, { from: 'a', to: 'b', branch: 'Each' })

		expect(next.edges).toHaveLength(1)
		expect(next.edges[0].branch).toBeUndefined()
	})

	it('removeEdge drops only the matching from/to pair', () => {
		const graph: Graph = {
			connectors: [],
			edges: [
				{ from: 'a', to: 'b' },
				{ from: 'a', to: 'c' },
			],
		}

		const next = removeEdge(graph, 'a', 'b')

		expect(next.edges).toEqual([{ from: 'a', to: 'c' }])
	})
})

describe('graphErrorsByConnector', () => {
	it('groups by connector id, and null for a graph-level error', () => {
		const grouped = graphErrorsByConnector([
			{ connector_id: 'a', field: 'url', message: 'URL invalide' },
			{ connector_id: 'a', field: 'method', message: 'Method invalide' },
			{ connector_id: null, field: null, message: 'Cycle détecté' },
		])

		expect(grouped.get('a')).toHaveLength(2)
		expect(grouped.get(null)).toEqual([
			{ connector_id: null, field: null, message: 'Cycle détecté' },
		])
	})
})

describe('fieldErrorsFor', () => {
	it('builds a field-name-keyed map for one connector, for FieldForm', () => {
		const errors = fieldErrorsFor(
			[
				{ connector_id: 'a', field: 'url', message: 'URL invalide' },
				{
					connector_id: 'b',
					field: 'url',
					message: 'Ne devrait pas apparaître',
				},
				{
					connector_id: 'a',
					field: null,
					message: 'Message global, ignoré ici',
				},
			],
			'a',
		)

		expect(errors).toEqual({ url: 'URL invalide' })
	})
})

function credential(overrides: Partial<Credential> = {}): Credential {
	return {
		id: 'cred-1',
		name: 'Odoo production',
		kind: 'odoo',
		origin: 'supplied',
		organization_id: 'org-1',
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
		...overrides,
	}
}

describe('matchesAuthRequirement', () => {
	it('accepts nothing for "None"', () => {
		expect(matchesAuthRequirement('None', 'odoo')).toBe(false)
	})

	it('accepts only the named kind for "Exactly"', () => {
		expect(matchesAuthRequirement({ Exactly: 'odoo' }, 'odoo')).toBe(true)
		expect(matchesAuthRequirement({ Exactly: 'odoo' }, 'http_basic')).toBe(
			false,
		)
	})

	it('accepts any named kind for "AnyOf"', () => {
		const requirement = { AnyOf: ['odoo', 'http_basic'] }
		expect(matchesAuthRequirement(requirement, 'http_basic')).toBe(true)
		expect(matchesAuthRequirement(requirement, 'webhook_signing')).toBe(false)
	})
})

describe('credentialOptionsFor', () => {
	it('keeps only credentials whose kind the connector accepts', () => {
		const connector = descriptor({ auth: { Exactly: 'odoo' } })
		const options = credentialOptionsFor(connector, [
			credential({ id: 'cred-1', kind: 'odoo' }),
			credential({ id: 'cred-2', kind: 'http_basic' }),
		])

		expect(options).toEqual([{ id: 'cred-1', name: 'Odoo production' }])
	})

	it('is empty for a connector with no auth requirement', () => {
		const connector = descriptor({ auth: 'None' })
		expect(credentialOptionsFor(connector, [credential()])).toEqual([])
	})
})
