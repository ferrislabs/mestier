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

describe('displayField — the lot 7 nodes', () => {
	it('asks the ticket of a note to be dragged in, and gives its text three lines', () => {
		const ticket = displayField(
			'mestier.note.add',
			field({ name: 'ticket_id', label: 'Ticket', expression: true }),
		)
		const body = displayField(
			'mestier.note.add',
			field({ name: 'body', label: 'Note', expression: true }),
		)

		expect(ticket.placeholder).toBe(
			'Glissez une variable depuis la colonne Entrée',
		)
		expect(body.rows).toBe(3)
		expect(body.label).toBe('Note')
	})

	it('gives the message of a channel post four lines and seeds nothing else', () => {
		const message = displayField(
			'mestier.chat.post_message',
			field({ name: 'message', label: 'Message', expression: true }),
		)

		expect(message.rows).toBe(4)
	})

	it('asks the quote and the name of a project to be dragged in, and says the quote must be accepted', () => {
		const quote = displayField(
			'mestier.project.create',
			field({ name: 'quote_id', label: 'Devis', expression: true }),
		)
		const name = displayField(
			'mestier.project.create',
			field({ name: 'name', label: 'Nom du projet', expression: true }),
		)

		expect(quote.placeholder).toBe(
			'Glissez une variable depuis la colonne Entrée',
		)
		expect(quote.hint).toBe('Le devis doit être accepté.')
		expect(name.placeholder).toBe('Glissez une variable ou saisissez un nom')
	})

	it('asks the recipient of an e-mail to be dragged in, gives its body seven lines and explains the lists', () => {
		const to = displayField(
			'smtp.send',
			field({ name: 'to', label: 'Destinataire', expression: true }),
		)
		const cc = displayField(
			'smtp.send',
			field({ name: 'cc', label: 'Copie', expression: true }),
		)
		const body = displayField(
			'smtp.send',
			field({ name: 'body', label: 'Corps', expression: true }),
		)

		expect(to.placeholder).toBe('Glissez une variable depuis la colonne Entrée')
		expect(to.hint).toBe(
			'Séparez les adresses par une virgule ou un point-virgule : 50 destinataires au maximum.',
		)
		expect(cc.hint).toBe(to.hint)
		expect(body.rows).toBe(7)
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

	it('seeds the wait with the two days of the prototype', () => {
		expect(withFieldDefaults('flow.wait', {})).toEqual({
			duration: 2,
			unit: 'days',
		})
	})

	it('seeds the merge in append mode', () => {
		expect(withFieldDefaults('flow.merge', {})).toEqual({ mode: 'append' })
	})

	it('seeds the quote search with the sent quotes of the last seven days, fifty at most', () => {
		expect(withFieldDefaults('mestier.quote.search', {})).toEqual({
			status: 'SENT',
			since_days: 7,
			limit: 50,
		})
	})

	it('seeds the channel post with the channel and message of the prototype', () => {
		expect(withFieldDefaults('mestier.chat.post_message', {})).toEqual({
			channel: '#general',
			message: 'Nouveau devis à valider.',
		})
	})

	it('seeds the note with a follow-up sentence the expression engine understands', () => {
		expect(withFieldDefaults('mestier.note.add', {})).toEqual({
			body: 'Relance envoyée le {{ format_date(now(), "%d/%m/%Y") }}',
		})
	})

	it('seeds the e-mail with the subject, the body and the text format', () => {
		expect(withFieldDefaults('smtp.send', {})).toEqual({
			subject: 'Votre dossier chez Atelier Duval',
			body: 'Bonjour,\n\nCordialement,\nAtelier Duval',
			format: 'text',
		})
	})

	it('keeps the values a lot 7 node already holds', () => {
		expect(
			withFieldDefaults('flow.wait', { duration: 5, unit: 'hours' }),
		).toBeNull()
		expect(withFieldDefaults('mestier.quote.search', { limit: 10 })).toEqual({
			limit: 10,
			status: 'SENT',
			since_days: 7,
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
		expect(credentialSlotAfter('smtp.send')).toBeNull()
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
