import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	collectionCredentialHint,
	credentialSlotAfter,
	displayField,
	withFieldDefaults,
} from '#/pages/automation/lib/connector-field-copy'

function field(
	overrides: Partial<Schemas.FieldResponse> = {},
): Schemas.FieldResponse {
	return {
		expression: false,
		kind: 'Text',
		label: 'Champ',
		name: 'field',
		required: false,
		secret: false,
		visible_when: null,
		...overrides,
	}
}

describe('displayField', () => {
	it('gives the HTTP body its full label', () => {
		const shown = displayField(
			'http.request',
			field({ name: 'body', label: 'Corps', kind: 'Json' }),
		)

		expect(shown.label).toBe('Corps de la requête')
	})

	it('writes the apostrophes of a server label as typographic ones', () => {
		const shown = displayField(
			'http.request',
			field({
				name: 'timeout_seconds',
				label: "Délai d'expiration (secondes)",
				kind: 'Number',
			}),
		)

		expect(shown.label).toBe('Délai d’expiration (secondes)')
	})

	it('also corrects the label of a kind the table does not know', () => {
		expect(
			displayField('inconnu.kind', field({ label: "Nom de l'objet" })).label,
		).toBe('Nom de l’objet')
	})

	it('adds the placeholder of a field that expects a variable', () => {
		const shown = displayField(
			'mestier.support.assign_ticket',
			field({ name: 'ticket_id', label: 'Ticket', expression: true }),
		)

		expect(shown.placeholder).toBe(
			'Glissez une variable depuis la colonne Entrée',
		)
	})

	it('adds the hint of the condition', () => {
		const shown = displayField(
			'flow.condition',
			field({ name: 'predicate', label: 'Condition', expression: true }),
		)

		expect(shown.hint).toBe(
			'Le flux continue par « vrai » si la condition est remplie, par « faux » sinon.',
		)
	})

	it('leaves a field without copy untouched', () => {
		const original = field({ name: 'ticket_id' })

		const shown = displayField('inconnu.kind', original)

		expect(shown).toEqual(original)
	})

	it('keeps the descriptor fields it does not override', () => {
		const shown = displayField(
			'odoo.create_invoice',
			field({
				name: 'amount',
				label: 'Montant',
				kind: 'Number',
				expression: true,
				required: true,
			}),
		)

		expect(shown.kind).toBe('Number')
		expect(shown.expression).toBe(true)
		expect(shown.required).toBe(true)
	})
})

describe('withFieldDefaults', () => {
	it('seeds the HTTP method of a fresh node', () => {
		expect(withFieldDefaults('http.request', {})).toEqual({
			method: 'GET',
		})
	})

	it('keeps what the config already holds', () => {
		expect(
			withFieldDefaults('http.request', {
				method: 'POST',
				url: '',
			}),
		).toBeNull()
	})

	it('only fills the missing keys and keeps the others', () => {
		expect(
			withFieldDefaults('http.request', { url: 'https://a', headers: {} }),
		).toEqual({ url: 'https://a', headers: {}, method: 'GET' })
	})

	it('seeds the badge tone', () => {
		expect(withFieldDefaults('mestier.support.add_badge', {})).toEqual({
			tone: 'neutral',
		})
	})

	it('has nothing to seed for a kind without defaults', () => {
		expect(withFieldDefaults('flow.condition', {})).toBeNull()
		expect(withFieldDefaults('inconnu.kind', {})).toBeNull()
	})

	it('does not mutate the config it receives', () => {
		const config = {}

		withFieldDefaults('http.request', config)

		expect(config).toEqual({})
	})
})

describe('credentialSlotAfter', () => {
	it('puts the HTTP identification right after the URL', () => {
		expect(credentialSlotAfter('http.request')).toBe('url')
	})

	it('puts the identification first for any other kind', () => {
		expect(credentialSlotAfter('odoo.create_partner')).toBeNull()
		expect(credentialSlotAfter('inconnu.kind')).toBeNull()
	})
})

describe('collectionCredentialHint', () => {
	it('names what the spec declares', () => {
		expect(collectionCredentialHint(['bearerAuth', 'apiKey'])).toBe(
			'La spec déclare : bearerAuth, apiKey. Aucune identification n’est appliquée automatiquement : choisissez celle à utiliser.',
		)
	})

	it('says so when the spec declares no authentication', () => {
		expect(collectionCredentialHint([])).toBe(
			'La spec ne déclare pas d’authentification. Aucune identification n’est appliquée automatiquement : choisissez celle à utiliser.',
		)
	})
})
