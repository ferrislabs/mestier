import { describe, expect, it } from 'vitest'
import {
	displayPath,
	escapeNodeLabel,
	findNodeReferences,
	listVariables,
	nodeReference,
	readPath,
	usesLegacyRoot,
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
	it('names the node and reads the json of its current item', () => {
		expect(variableExpression('Devis accepté', ['client', 'nom'])).toBe(
			"{{ $('Devis accepté').item.json.client.nom }}",
		)
		expect(variableExpression('Si', ['lignes', 0, 'a b'])).toBe(
			'{{ $(\'Si\').item.json.lignes[0]["a b"] }}',
		)
	})

	it('escapes the apostrophes and the backslashes of the name', () => {
		expect(variableExpression("Prix d'achat", ['prix'])).toBe(
			"{{ $('Prix d\\'achat').item.json.prix }}",
		)
		expect(nodeReference('A\\B')).toBe("$('A\\\\B')")
		expect(escapeNodeLabel("l'un")).toBe("l\\'un")
	})

	it('reads the whole item when the path is empty', () => {
		expect(variableExpression('Si', [])).toBe("{{ $('Si').item.json }}")
	})
})

describe('readPath', () => {
	it('follows dots, brackets and indexes', () => {
		const value = { a: { 'b c': [{ d: 7 }] } }
		expect(readPath(value, '.a["b c"][0].d')).toBe(7)
		expect(readPath(value, '.a.missing')).toBeUndefined()
	})
})

describe('findNodeReferences', () => {
	it('finds every node named in a text, with the path read in its item', () => {
		expect(
			findNodeReferences(
				"{{ $('Si').item.json.id }} et {{ $('Créer un projet').item.json.client[\"nom\"] }}",
			),
		).toEqual([
			{ label: 'Si', accessor: '.id' },
			{ label: 'Créer un projet', accessor: '.client["nom"]' },
		])
	})

	it('unescapes the name', () => {
		expect(
			findNodeReferences("{{ $('Prix d\\'achat').item.json.prix[0].code }}"),
		).toEqual([{ label: "Prix d'achat", accessor: '.prix[0].code' }])
	})

	it('reads first, last and the item the same way', () => {
		expect(
			findNodeReferences(
				"{{ $('A').first().json.x }} {{ $('B').last().json.y }} {{ $('C').item.json }}",
			),
		).toEqual([
			{ label: 'A', accessor: '.x' },
			{ label: 'B', accessor: '.y' },
			{ label: 'C', accessor: '' },
		])
	})

	it('names the node but has no path to check for all()', () => {
		expect(findNodeReferences("{{ len($('A').all()) }}")).toEqual([
			{ label: 'A', accessor: null },
		])
	})

	it('finds nothing in a text without a node reference', () => {
		expect(findNodeReferences('{{ trigger.id }} et {{ now() }}')).toEqual([])
	})
})

describe('usesLegacyRoot', () => {
	it('spots the root connectors.<id>.output', () => {
		expect(usesLegacyRoot('{{ connectors.c1.output.total }}')).toBe(true)
		expect(usesLegacyRoot('a {{ upper(connectors.c2.output.name) }}')).toBe(
			true,
		)
	})

	it('leaves the new syntax and the other roots alone', () => {
		expect(usesLegacyRoot("{{ $('Si').item.json.total }}")).toBe(false)
		expect(usesLegacyRoot('{{ trigger.total }}')).toBe(false)
		expect(usesLegacyRoot('connectors are nice')).toBe(false)
	})
})

describe('displayPath', () => {
	it('flattens an accessor to a readable path', () => {
		expect(displayPath('.client["nom complet"].x')).toBe('client.nom complet.x')
	})
})
