import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	connectorNode,
	type EditorLookup,
} from '#/pages/automation/lib/editor-nodes'
import { nodeIssues } from '#/pages/automation/lib/node-issues'

const lookup: EditorLookup = { descriptors: new Map(), events: [] }

function node(id: string, config: Record<string, unknown> = {}) {
	const connector: Schemas.PlacedConnectorDto = {
		id,
		kind: 'http.request',
		version: 1,
		config,
	}
	return connectorNode(connector, { x: 0, y: 0 }, lookup)
}

const nothing = () => undefined

describe('nodeIssues', () => {
	it('flags a reference to a node that no longer exists', () => {
		const target = node('c2', { url: '{{ connectors.c9.output.id }}' })
		expect(nodeIssues(target, [target], [], nothing)).toEqual([
			{ parameter: 'url', message: 'Le nœud « c9 » n’existe plus.' },
		])
	})

	it('flags a reference to a node that is not upstream', () => {
		const source = node('c1')
		const target = node('c2', { url: '{{ connectors.c1.output.id }}' })
		expect(nodeIssues(target, [source, target], [], nothing)[0].message).toBe(
			'Le nœud « c1 » n’est pas relié en amont de ce nœud.',
		)
	})

	it('flags a key missing from a known output', () => {
		const source = node('c1')
		const target = node('c2', { url: '{{ connectors.c1.output.id }}' })
		const edges = [{ id: 'e', source: 'c1', target: 'c2' }]
		expect(
			nodeIssues(target, [source, target], edges, () => ({ name: 'x' }))[0]
				.message,
		).toBe('La clé « id » est absente de la sortie de « c1 ».')
	})

	it('stays quiet when the reference resolves or the output is unknown', () => {
		const source = node('c1')
		const target = node('c2', { url: '{{ connectors.c1.output.id }}' })
		const edges = [{ id: 'e', source: 'c1', target: 'c2' }]
		expect(
			nodeIssues(target, [source, target], edges, () => ({ id: 1 })),
		).toEqual([])
		expect(nodeIssues(target, [source, target], edges, nothing)).toEqual([])
	})
})
