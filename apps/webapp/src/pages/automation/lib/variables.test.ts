import { describe, expect, it } from 'vitest'
import {
	displayPath,
	findConnectorReferences,
	listVariables,
	readPath,
	variableExpression,
} from '#/pages/automation/lib/variables'

describe('listVariables', () => {
	it('lists nested keys with their type and depth', () => {
		const variables = listVariables({ client: { nom: 'Ada' }, total: 3 })
		expect(variables.map((v) => [v.key, v.type, v.depth])).toEqual([
			['client', 'objet', 0],
			['nom', 'texte', 1],
			['total', 'nombre', 0],
		])
	})

	it('descends into the first element of a list', () => {
		const variables = listVariables({ lignes: [{ prix: 1 }] })
		expect(variables.map((v) => v.key)).toEqual(['lignes', '[0]', 'prix'])
		expect(variables[0].preview).toBe('1 élément')
	})

	it('stops at the depth limit', () => {
		expect(listVariables({ a: { b: { c: 1 } } }, 1).map((v) => v.key)).toEqual([
			'a',
			'b',
		])
	})
})

describe('variableExpression', () => {
	it('builds the expression the evaluator understands', () => {
		expect(variableExpression('trigger', ['client', 'nom'])).toBe(
			'{{ trigger.client.nom }}',
		)
		expect(
			variableExpression('connectors.c1.output', ['lignes', 0, 'a b']),
		).toBe('{{ connectors.c1.output.lignes[0]["a b"] }}')
	})
})

describe('readPath', () => {
	it('follows dots, brackets and indexes', () => {
		const value = { a: { 'b c': [{ d: 7 }] } }
		expect(readPath(value, '.a["b c"][0].d')).toBe(7)
		expect(readPath(value, '.a.missing')).toBeUndefined()
	})
})

describe('findConnectorReferences', () => {
	it('finds every connector output referenced in a text', () => {
		expect(
			findConnectorReferences(
				'{{ connectors.c1.output.id }} et {{ connectors.c2.output.client["nom"] }}',
			),
		).toEqual([
			{ connectorId: 'c1', accessor: '.id' },
			{ connectorId: 'c2', accessor: '.client["nom"]' },
		])
	})
})

describe('displayPath', () => {
	it('flattens an accessor to a readable path', () => {
		expect(displayPath('.client["nom complet"].x')).toBe('client.nom complet.x')
	})
})
