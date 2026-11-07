import { describe, expect, it } from 'vitest'
import {
	CONTRACT_KINDS,
	type ContractKey,
	contractFromExample,
	contractIssues,
	contractSource,
	defaultWebhook,
	hasContractIssue,
	newContractKey,
	removeContractKey,
	sampleBody,
	updateContractKey,
} from '#/pages/automation/lib/webhook-contract'

function key(overrides: Partial<ContractKey> = {}): ContractKey {
	return {
		name: 'montant',
		kind: 'Number',
		required: true,
		example: '',
		...overrides,
	}
}

describe('CONTRACT_KINDS', () => {
	it('offers the five types the API knows, in French', () => {
		expect(CONTRACT_KINDS).toEqual([
			{ value: 'Text', label: 'Texte' },
			{ value: 'Number', label: 'Nombre' },
			{ value: 'Boolean', label: 'Booléen' },
			{ value: 'Object', label: 'Objet' },
			{ value: 'List', label: 'Liste' },
		])
	})
})

describe('defaultWebhook', () => {
	it('starts as a POST with no path, no identification and no key, refusing a broken body', () => {
		expect(defaultWebhook()).toEqual({
			method: 'POST',
			path: '',
			credential_id: null,
			contract: [],
			on_invalid: 'Reject',
		})
	})
})

describe('newContractKey', () => {
	it('is a required text key with no example', () => {
		expect(newContractKey()).toEqual({
			name: '',
			kind: 'Text',
			required: true,
			example: '',
		})
	})
})

describe('contractIssues', () => {
	it('says nothing about a valid contract', () => {
		expect(
			contractIssues([
				key({ name: 'facture', kind: 'Text', example: 'FA-1' }),
				key({ name: 'montant', example: '2340' }),
				key({ name: 'paye', kind: 'Boolean', example: 'true' }),
				key({ name: 'client', kind: 'Object', example: '{"nom": "A"}' }),
				key({ name: 'lignes', kind: 'List', example: '[1, 2]' }),
				key({ name: '_x-1', example: '' }),
			]),
		).toEqual([null, null, null, null, null, null])
	})

	it('asks for a name', () => {
		expect(contractIssues([key({ name: '' }), key({ name: '   ' })])).toEqual([
			'Donnez un nom à cette clé.',
			'Donnez un nom à cette clé.',
		])
	})

	it.each([
		'1montant',
		'mon montant',
		'montant.ht',
		' montant',
		'é',
	])('refuses the name « %s »', (name) => {
		expect(contractIssues([key({ name })])).toEqual([
			'Lettres, chiffres, « _ » et « - » uniquement.',
		])
	})

	it('flags the second key of a pair of duplicates', () => {
		expect(
			contractIssues([
				key({ name: 'a' }),
				key({ name: 'b' }),
				key({ name: 'a' }),
			]),
		).toEqual([null, null, 'Clé en double à ce niveau.'])
	})

	it('flags an example that does not fit the type, as the sample would drop it', () => {
		expect(
			contractIssues([
				key({ example: 'beaucoup' }),
				key({ name: 'a', kind: 'Boolean', example: 'oui' }),
				key({ name: 'b', kind: 'Object', example: '[1]' }),
				key({ name: 'c', kind: 'List', example: '{"a": 1}' }),
				key({ name: 'd', kind: 'List', example: '[1' }),
				key({ name: 'e', kind: 'Number', example: '007' }),
			]),
		).toEqual([
			'Exemple invalide pour le type « Nombre ».',
			'Exemple invalide pour le type « Booléen ».',
			'Exemple invalide pour le type « Objet ».',
			'Exemple invalide pour le type « Liste ».',
			'Exemple invalide pour le type « Liste ».',
			'Exemple invalide pour le type « Nombre ».',
		])
	})

	it('takes any text as an example of a text key', () => {
		expect(
			contractIssues([key({ kind: 'Text', example: '{pas du json' })]),
		).toEqual([null])
	})

	it('tells whether a contract has an issue', () => {
		expect(hasContractIssue([key()])).toBe(false)
		expect(hasContractIssue([key({ name: '' })])).toBe(true)
		expect(hasContractIssue([])).toBe(false)
	})
})

describe('sampleBody', () => {
	it('builds the body the way the dry run does, from the examples', () => {
		expect(
			sampleBody([
				key({ name: 'evenement', kind: 'Text', example: 'paiement.recu' }),
				key({ name: 'montant', example: '2340' }),
				key({ name: 'paye', kind: 'Boolean', example: 'false' }),
				key({ name: 'client', kind: 'Object', example: '{"nom": "Marc"}' }),
				key({ name: 'lignes', kind: 'List', example: '["a"]' }),
			]),
		).toEqual({
			evenement: 'paiement.recu',
			montant: 2340,
			paye: false,
			client: { nom: 'Marc' },
			lignes: ['a'],
		})
	})

	it('falls back to the default of each type when the example is empty or does not fit', () => {
		expect(
			sampleBody([
				key({ name: 'texte', kind: 'Text' }),
				key({ name: 'nombre', example: 'beaucoup' }),
				key({ name: 'booleen', kind: 'Boolean' }),
				key({ name: 'objet', kind: 'Object', example: '[1]' }),
				key({ name: 'liste', kind: 'List', example: '{}' }),
			]),
		).toEqual({
			texte: 'texte',
			nombre: 12.5,
			booleen: true,
			objet: {},
			liste: [],
		})
	})

	it('skips a key that has no name yet', () => {
		expect(sampleBody([key({ name: '' }), key({ name: 'a' })])).toEqual({
			a: 12.5,
		})
	})
})

describe('contractSource', () => {
	it('declares the body, optional keys marked, named after the path', () => {
		expect(
			contractSource('stripe/paiement-recu', [
				key({ name: 'evenement', kind: 'Text' }),
				key({ name: 'montant' }),
				key({ name: 'paye', kind: 'Boolean', required: false }),
				key({ name: 'client', kind: 'Object', required: false }),
				key({ name: 'lignes', kind: 'List' }),
			]),
		).toBe(
			[
				'export interface StripePaiementRecuBody {',
				'  evenement: string',
				'  montant: number',
				'  paye?: boolean',
				'  client?: Record<string, unknown>',
				'  lignes: unknown[]',
				'}',
			].join('\n'),
		)
	})

	it('falls back to a generic name and leaves out the keys that cannot be written yet', () => {
		expect(contractSource('', [key({ name: '' }), key({ name: 'a b' })])).toBe(
			'export interface WebhookBody {}',
		)
	})
})

describe('editing helpers', () => {
	const contract = [key({ name: 'a' }), key({ name: 'b' }), key({ name: 'c' })]

	it('updates one key without touching the others', () => {
		const next = updateContractKey(contract, 1, {
			kind: 'Text',
			required: false,
		})

		expect(next.map((entry) => entry.kind)).toEqual([
			'Number',
			'Text',
			'Number',
		])
		expect(next[1]).toMatchObject({ name: 'b', required: false })
		expect(contract[1]?.kind).toBe('Number')
	})

	it('removes one key by position, so that duplicates are not removed together', () => {
		expect(
			removeContractKey([key({ name: 'a' }), key({ name: 'a' })], 0),
		).toHaveLength(1)
		expect(removeContractKey(contract, 1).map((entry) => entry.name)).toEqual([
			'a',
			'c',
		])
	})
})

describe('contractFromExample', () => {
	it('infers a required key per top level entry, keeping objects and lists as JSON examples', () => {
		expect(
			contractFromExample({
				facture: 'FA-1',
				montant: 2340,
				paye: true,
				client: { nom: 'Marc' },
				lignes: [1, 2],
				vide: null,
			}),
		).toEqual([
			{ name: 'facture', kind: 'Text', required: true, example: 'FA-1' },
			{ name: 'montant', kind: 'Number', required: true, example: '2340' },
			{ name: 'paye', kind: 'Boolean', required: true, example: 'true' },
			{
				name: 'client',
				kind: 'Object',
				required: true,
				example: '{"nom":"Marc"}',
			},
			{ name: 'lignes', kind: 'List', required: true, example: '[1,2]' },
			{ name: 'vide', kind: 'Text', required: true, example: '' },
		])
	})

	it('refuses anything that is not an object', () => {
		expect(contractFromExample([1])).toBeNull()
		expect(contractFromExample('x')).toBeNull()
		expect(contractFromExample(null)).toBeNull()
	})

	it('infers keys whose examples pass the contract check', () => {
		const inferred = contractFromExample({ a: 1, b: { c: 1 }, d: [1], e: 'x' })
		expect(hasContractIssue(inferred ?? [])).toBe(false)
	})
})
