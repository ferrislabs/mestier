import {
	CalendarClock,
	GitBranch,
	Globe,
	MousePointerClick,
	Webhook,
} from 'lucide-react'
import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	connectorNode,
	type EditorEdge,
	type EditorLookup,
	triggerNode,
} from '#/pages/automation/lib/editor-nodes'
import {
	evaluateContextFor,
	type UpstreamSources,
	upstreamEntries,
} from '#/pages/automation/lib/upstream-entries'

function descriptor(
	kind: string,
	family: string,
	overrides: Partial<Schemas.ConnectorDescriptorResponse> = {},
): Schemas.ConnectorDescriptorResponse {
	return {
		auth: 'None',
		branches: [],
		family,
		fields: [],
		kind,
		label: kind,
		output_example: { ok: true },
		version: 1,
		...overrides,
	}
}

const EVENTS: Schemas.EventDescriptorResponse[] = [
	{
		label: 'Devis accepté',
		name: 'quote.accepted',
		payload_example: { quote: 'q1' },
		subject_kind: 'quote',
		version: 1,
	},
	{
		label: 'Facture payée',
		name: 'invoice.paid',
		payload_example: { invoice: 'i1' },
		subject_kind: 'invoice',
		version: 1,
	},
]

const LOOKUP: EditorLookup = {
	descriptors: new Map([
		[
			'flow.condition',
			descriptor('flow.condition', 'flow', { branches: ['Then', 'Else'] }),
		],
		['http.request', descriptor('http.request', 'http')],
	]),
	events: EVENTS,
}

const SOURCES: UpstreamSources = {
	descriptors: LOOKUP.descriptors,
	events: EVENTS,
	lastRun: null,
	runSteps: new Map(),
}

const AT = { x: 0, y: 0 }

function connector(id: string, kind: string) {
	return connectorNode({ id, kind, version: 1, config: {} }, AT, LOOKUP)
}

function trigger(id: string, kind: Schemas.TriggerKindDto) {
	return triggerNode({ id, kind }, AT, LOOKUP)
}

function edge(source: string, target: string, handle?: string): EditorEdge {
	return {
		id: `${source}->${target}:${handle ?? ''}`,
		source,
		target,
		sourceHandle: handle,
	}
}

describe('upstreamEntries — a connector upstream', () => {
	const nodes = [
		trigger('t1', { Events: ['quote.accepted'] }),
		connector('c1', 'flow.condition'),
		connector('c2', 'http.request'),
	]

	it('takes the icon and hue of its type, not of its family', () => {
		const entries = upstreamEntries(
			'c2',
			nodes,
			[edge('t1', 'c1'), edge('c1', 'c2', 'Then')],
			SOURCES,
		)

		const condition = entries.find((entry) => entry.id === 'c1')
		expect(condition?.icon).toBe(GitBranch)
		expect(condition?.hue).toBe(145)
	})

	it('names the branch it comes from in French', () => {
		const entries = upstreamEntries(
			'c2',
			nodes,
			[edge('t1', 'c1'), edge('c1', 'c2', 'Then')],
			SOURCES,
		)

		expect(entries.find((entry) => entry.id === 'c1')?.branch).toBe('vrai')
	})

	it('has no branch to name when the edge carries none', () => {
		const entries = upstreamEntries('c2', nodes, [edge('t1', 'c2')], SOURCES)

		expect(
			entries.find((entry) => entry.id === 'trigger')?.branch,
		).toBeUndefined()
	})

	it('names the other branch too', () => {
		const entries = upstreamEntries(
			'c2',
			nodes,
			[edge('t1', 'c1'), edge('c1', 'c2', 'Else')],
			SOURCES,
		)

		expect(entries.find((entry) => entry.id === 'c1')?.branch).toBe('faux')
	})
})

describe('upstreamEntries — the trigger entry', () => {
	it('looks like the trigger it comes from', () => {
		const nodes = [trigger('t1', 'Manual'), connector('c1', 'http.request')]

		const entries = upstreamEntries('c1', nodes, [edge('t1', 'c1')], SOURCES)

		expect(entries[0]?.label).toBe('Clic')
		expect(entries[0]?.icon).toBe(MousePointerClick)
		expect(entries[0]?.hue).toBe(220)
	})

	it('shows the example of the first event of the trigger linked to the node', () => {
		const nodes = [
			trigger('t1', { Events: ['quote.accepted'] }),
			trigger('t2', { Events: ['invoice.paid'] }),
			connector('c1', 'http.request'),
		]

		const entries = upstreamEntries('c1', nodes, [edge('t2', 'c1')], SOURCES)

		expect(entries[0]?.items).toEqual([{ invoice: 'i1' }])
	})

	it('follows the order of the selection, whatever the catalogue order', () => {
		const nodes = [
			trigger('t1', { Events: ['invoice.paid', 'quote.accepted'] }),
			connector('c1', 'http.request'),
		]

		const entries = upstreamEntries('c1', nodes, [edge('t1', 'c1')], SOURCES)

		expect(entries[0]?.items).toEqual([{ invoice: 'i1' }])
	})

	it('hands the same example to the expression preview', () => {
		const nodes = [
			trigger('t1', { Events: ['invoice.paid', 'quote.accepted'] }),
			connector('c1', 'http.request'),
		]

		const context = evaluateContextFor('c1', nodes, [edge('t1', 'c1')], SOURCES)

		expect(context.trigger).toEqual({ invoice: 'i1' })
	})
})

describe('upstreamEntries — the webhook and the scheduler', () => {
	const hook: Schemas.TriggerKindDto = {
		Webhook: {
			method: 'POST',
			path: 'paiement-recu',
			contract: [
				{ name: 'facture', kind: 'Text', required: true, example: 'FA-1' },
				{ name: 'montant', kind: 'Number', required: true, example: '2340' },
			],
		},
	}
	const cron: Schemas.TriggerKindDto = {
		Schedule: { frequency: 'Daily', time: '09:00', timezone: 'Europe/Paris' },
	}

	it('looks like a webhook and offers the keys of its contract as variables', () => {
		const nodes = [trigger('t1', hook), connector('c1', 'http.request')]

		const [entry] = upstreamEntries('c1', nodes, [edge('t1', 'c1')], SOURCES)

		expect(entry?.label).toBe('Webhook')
		expect(entry?.icon).toBe(Webhook)
		expect(entry?.hue).toBe(335)
		expect(entry?.items).toEqual([{ facture: 'FA-1', montant: 2340 }])
	})

	it('looks like a scheduler and offers the four keys of its payload', () => {
		const nodes = [trigger('t1', cron), connector('c1', 'http.request')]

		const [entry] = upstreamEntries('c1', nodes, [edge('t1', 'c1')], SOURCES)

		expect(entry?.label).toBe('Planificateur')
		expect(entry?.icon).toBe(CalendarClock)
		expect(entry?.hue).toBe(255)
		expect(Object.keys(entry?.items[0] as object)).toEqual([
			'scheduled_for',
			'horodatage',
			'jour',
			'fuseau',
		])
	})

	it('hands the contract example to the expression preview', () => {
		const nodes = [trigger('t1', hook), connector('c1', 'http.request')]

		const context = evaluateContextFor('c1', nodes, [edge('t1', 'c1')], SOURCES)

		expect(context.trigger).toEqual({ facture: 'FA-1', montant: 2340 })
	})

	it('prefers the payload of the last run to the example', () => {
		const nodes = [trigger('t1', hook), connector('c1', 'http.request')]
		const sources: UpstreamSources = {
			...SOURCES,
			lastRun: { triggerPayload: { facture: 'FA-9' }, connectorOutputs: {} },
		}

		const [entry] = upstreamEntries('c1', nodes, [edge('t1', 'c1')], sources)

		expect(entry?.items).toEqual([{ facture: 'FA-9' }])
	})

	it('shows the example of the trigger that is linked to the node, not the first one', () => {
		const nodes = [
			trigger('t1', { Events: ['quote.accepted'] }),
			trigger('t2', hook),
			connector('c1', 'http.request'),
		]

		const [entry] = upstreamEntries('c1', nodes, [edge('t2', 'c1')], SOURCES)

		expect(entry?.label).toBe('Webhook')
		expect(entry?.items).toEqual([{ facture: 'FA-1', montant: 2340 }])
	})
})

describe('upstreamEntries — the icon of an unknown connector', () => {
	it('falls back to the family', () => {
		const nodes = [
			trigger('t1', 'Manual'),
			connector('c1', 'http.request'),
			connectorNode(
				{ id: 'c2', kind: 'inconnu.kind', version: 1, config: {} },
				AT,
				LOOKUP,
			),
		]

		const entries = upstreamEntries(
			'c2',
			nodes,
			[edge('t1', 'c1'), edge('c1', 'c2')],
			SOURCES,
		)

		expect(entries.find((entry) => entry.id === 'c1')?.icon).toBe(Globe)
	})
})

describe('evaluateContextFor — several triggers', () => {
	it('previews with the event of the trigger the node is linked to', () => {
		const nodes = [
			trigger('t1', { Events: ['quote.accepted'] }),
			trigger('t2', { Events: ['invoice.paid'] }),
			connector('c1', 'http.request'),
		]

		const context = evaluateContextFor('c1', nodes, [edge('t2', 'c1')], SOURCES)

		expect(context.trigger).toEqual({ invoice: 'i1' })
	})
})
