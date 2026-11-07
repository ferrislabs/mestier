import { describe, expect, it } from 'vitest'
import {
	acceptsUsage,
	allowsGeneratedOrigin,
	credentialAcceptsUsage,
	findKind,
	kindLabel,
	kindsAccepting,
	searchKindGroups,
	usageSentence,
	webhookCredentialHint,
} from '#/pages/automation/lib/credential-kinds'
import {
	BEARER_TOKEN,
	HMAC,
	HTTP_BASIC,
	HTTP_HEADER,
	KINDS,
	kind,
	ODOO_API,
	SMTP,
	SSH,
} from '#/pages/automation/test/credential-fixtures'

describe('findKind and kindLabel', () => {
	it('reads the label from the descriptor the server sent', () => {
		expect(kindLabel(KINDS, 'http_basic')).toBe('Basic Auth')
		expect(findKind(KINDS, 'ssh')).toBe(SSH)
	})

	it('falls back to the code for a kind the server no longer lists', () => {
		expect(kindLabel(KINDS, 'mystery')).toBe('mystery')
		expect(findKind(KINDS, 'mystery')).toBeUndefined()
	})
})

describe('usages', () => {
	it('tells whether a kind lists a usage', () => {
		expect(acceptsUsage(HTTP_BASIC, 'webhook')).toBe(true)
		expect(acceptsUsage(ODOO_API, 'webhook')).toBe(false)
	})

	it('keeps, in the server order, the kinds that list the usage', () => {
		expect(kindsAccepting(KINDS, 'webhook').map((k) => k.kind)).toEqual([
			'http_basic',
			'bearer_token',
			'http_header',
			'hmac',
		])
	})

	it('tells whether the kind of a credential lists the usage, false for an unknown kind', () => {
		expect(credentialAcceptsUsage(KINDS, { kind: 'hmac' }, 'webhook')).toBe(
			true,
		)
		expect(credentialAcceptsUsage(KINDS, { kind: 'odoo_api' }, 'webhook')).toBe(
			false,
		)
		expect(credentialAcceptsUsage(KINDS, { kind: 'mystery' }, 'webhook')).toBe(
			false,
		)
	})

	it('names the usages that have a node when there is one', () => {
		expect(usageSentence(BEARER_TOKEN)).toBe(
			'Utilisable dans : Requête HTTP et opérations de collection, Webhook.',
		)
	})

	it('says nothing runs the kind yet when none of its usages has a node', () => {
		expect(usageSentence(SMTP)).toBe(
			'Prévue pour : E-mail SMTP. Aucun nœud ne l’utilise encore.',
		)
	})

	it('lists the kinds a webhook accepts', () => {
		expect(webhookCredentialHint(KINDS)).toBe(
			'Types acceptés : Basic Auth, Bearer, Clé d’API dans un en-tête, Signature HMAC.',
		)
	})

	it('has no hint while the kinds are not there', () => {
		expect(webhookCredentialHint([])).toBe('')
	})
})

describe('allowsGeneratedOrigin', () => {
	it('follows the origin choices of the server', () => {
		expect(allowsGeneratedOrigin(BEARER_TOKEN)).toBe(true)
		expect(allowsGeneratedOrigin(HTTP_HEADER)).toBe(false)
	})
})

describe('searchKindGroups', () => {
	it('groups the kinds by the categories of the server, in the order it lists them', () => {
		expect(
			searchKindGroups(KINDS, '').map((group) => [
				group.id,
				group.label,
				group.kinds.length,
			]),
		).toEqual([
			['http', 'Requêtes HTTP', 6],
			['webhook', 'Webhooks entrants', 1],
			['email', 'E-mail', 1],
			['servers', 'Serveurs et bases de données', 2],
			['other', 'Autres', 1],
		])
	})

	it('finds a kind by its label, its description or its category, ignoring case and accents', () => {
		const ids = (query: string) =>
			searchKindGroups(KINDS, query).flatMap((group) =>
				group.kinds.map((k) => k.kind),
			)

		expect(ids('BEARER')).toEqual(['bearer_token'])
		expect(ids('mutuelle')).toEqual(['mtls'])
		expect(ids('e-mail')).toEqual(['smtp'])
		expect(ids('serveurs')).toEqual(['ssh', 'sql'])
	})

	it('does not match the technical code', () => {
		expect(searchKindGroups(KINDS, 'http_header')).toEqual([])
		expect(searchKindGroups(KINDS, 'odoo_api')).toEqual([])
	})

	it('drops the groups left empty and the kinds the server does not support', () => {
		const hidden = kind({ kind: 'hidden', supported: false })

		expect(searchKindGroups([hidden, HMAC], '')).toEqual([
			{ id: 'webhook', label: 'Webhooks entrants', kinds: [HMAC] },
		])
	})
})
