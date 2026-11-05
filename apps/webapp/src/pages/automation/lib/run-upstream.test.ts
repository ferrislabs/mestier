import { Zap } from 'lucide-react'
import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { runUpstreamEntries } from '#/pages/automation/lib/run-upstream'
import type { RunNodeView } from '#/pages/automation/lib/run-view'

const GRAPH: Schemas.GraphDto = {
	triggers: [{ id: 't1', kind: 'Manual' }],
	connectors: [],
	edges: [
		{ from: 't1', to: 'c1' },
		{ from: 'c1', to: 'c2', branch: 'Then' },
	],
}

function node(overrides: Partial<RunNodeView>): RunNodeView {
	return {
		id: 'c1',
		kind: 'connector',
		label: 'Facture',
		family: undefined,
		subtitle: '',
		status: 'success',
		durationMs: null,
		step: null,
		connector: null,
		trigger: null,
		note: '',
		order: null,
		...overrides,
	}
}

describe('runUpstreamEntries', () => {
	it('walks back through the graph with distance, branch and outputs', () => {
		const entries = runUpstreamEntries({
			graph: GRAPH,
			nodeId: 'c2',
			nodes: [
				node({ id: 't1', kind: 'trigger', label: 'Déclencheur' }),
				node({
					id: 'c1',
					step: { output: [{ a: 1 }, { a: 2 }] } as Schemas.RunStepResponse,
				}),
			],
			triggerPayload: { number: 'DV-1' },
			appearanceOf: () => ({ icon: Zap, hue: 10 }),
		})

		expect(entries.map((entry) => entry.id)).toEqual(['c1', 't1'])
		expect(entries[0]).toMatchObject({
			rootPath: 'connectors.c1.output',
			distance: 1,
			direct: true,
			branch: 'Then',
			status: 'success',
		})
		expect(entries[0]?.items).toHaveLength(2)
		expect(entries[1]).toMatchObject({
			rootPath: 'trigger',
			distance: 2,
			direct: false,
		})
		expect(entries[1]?.items).toEqual([{ number: 'DV-1' }])
	})

	it('reports a failed upstream node with its error', () => {
		const entries = runUpstreamEntries({
			graph: GRAPH,
			nodeId: 'c2',
			nodes: [
				node({
					status: 'error',
					step: { error: 'SMTP expiré' } as Schemas.RunStepResponse,
				}),
			],
			triggerPayload: undefined,
			appearanceOf: () => ({ icon: Zap, hue: 10 }),
		})

		expect(entries[0]).toMatchObject({ status: 'error', error: 'SMTP expiré' })
	})
})
