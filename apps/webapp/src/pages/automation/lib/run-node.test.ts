import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	displayValue,
	isExpressionValue,
	nodeNeighbours,
	parameterEntries,
} from '#/pages/automation/lib/run-node'

const DESCRIPTOR = {
	kind: 'mail',
	label: 'Mail',
	family: 'mestier',
	branches: [],
	fields: [
		{ name: 'to', label: 'Destinataire', secret: false },
		{ name: 'token', label: 'Jeton', secret: true },
	],
} as unknown as Schemas.ConnectorDescriptorResponse

describe('run node helpers', () => {
	it('stringifies values for display', () => {
		expect(displayValue(undefined)).toBe('')
		expect(displayValue('a')).toBe('a')
		expect(displayValue({ a: 1 })).toBe('{\n  "a": 1\n}')
	})

	it('detects expressions', () => {
		expect(isExpressionValue('{{ trigger.email }}')).toBe(true)
		expect(isExpressionValue('plain')).toBe(false)
		expect(isExpressionValue(3)).toBe(false)
	})

	it('lists described fields without secrets, then undeclared config keys', () => {
		const connector = {
			id: 'c1',
			kind: 'mail',
			version: 1,
			config: { to: '{{ trigger.email }}', token: 'x', extra: 1 },
		}
		expect(parameterEntries(connector, DESCRIPTOR)).toEqual([
			{
				id: 'to',
				label: 'Destinataire',
				value: '{{ trigger.email }}',
				expression: true,
			},
			{ id: 'extra', label: 'extra', value: '1', expression: false },
		])
		expect(parameterEntries(null, DESCRIPTOR)).toEqual([])
	})

	it('finds the neighbours of a node with their branch', () => {
		const graph: Schemas.GraphDto = {
			triggers: [],
			connectors: [],
			edges: [
				{ from: 'a', to: 'b' },
				{ from: 'b', to: 'c', branch: 'Then' },
				{ from: 'b', to: 'd', branch: 'Else' },
			],
		}
		expect(nodeNeighbours(graph, 'b')).toEqual({
			previous: [{ id: 'a', branch: undefined }],
			next: [
				{ id: 'c', branch: 'Then' },
				{ id: 'd', branch: 'Else' },
			],
		})
	})
})
