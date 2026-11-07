import { describe, expect, it } from 'vitest'
import type { AuthScheme } from '#/hooks/use-automation'
import {
	CREDENTIAL_CATEGORIES,
	isOauthAuthorization,
	kindLabel,
	localizeScheme,
	newestFirst,
	schemeDescription,
	schemeUsageText,
	searchSchemeGroups,
	seededCredentialMeta,
	usableInWebhook,
	usageSentence,
	webhookCredentialHint,
} from '#/pages/automation/lib/credential-meta'

function field(
	name: string,
	label: string,
	secret = false,
): AuthScheme['fields'][number] {
	return {
		name,
		label,
		required: true,
		kind: 'Text',
		expression: false,
		secret,
	}
}

const BEARER: AuthScheme = {
	kind: 'bearer_token',
	label: 'Bearer token',
	fields: [field('token', 'Token', true)],
}

const BASIC: AuthScheme = {
	kind: 'http_basic',
	label: 'HTTP basic auth',
	fields: [field('username', 'Username'), field('password', 'Password', true)],
}

const HEADER: AuthScheme = {
	kind: 'http_header',
	label: 'HTTP header',
	fields: [
		field('header_name', 'Header name'),
		field('header_value', 'Header value', true),
	],
}

const ODOO: AuthScheme = {
	kind: 'odoo_api',
	label: 'Odoo API key',
	fields: [
		field('base_url', 'Instance URL'),
		field('database', 'Database'),
		field('username', 'Username'),
		field('api_key', 'API key', true),
	],
}

const UNKNOWN: AuthScheme = {
	kind: 'something_new',
	label: 'Nouveau',
	fields: [field('a', 'Champ A')],
}

describe('credential meta', () => {
	it('describes known kinds in prose and unknown ones by their fields', () => {
		expect(schemeDescription(BEARER)).toBe(
			'Un jeton envoyé sous la forme « Authorization: Bearer … ».',
		)
		expect(schemeDescription(UNKNOWN)).toBe('Champ A')
	})

	it('states where a kind can be used, the webhook trigger included', () => {
		for (const kind of ['http_basic', 'http_header', 'bearer_token']) {
			expect(schemeUsageText(kind), kind).toBe(
				'Utilisable dans : Requête HTTP et opérations de collection, Webhook.',
			)
		}
	})

	it('announces only the Odoo actions for an Odoo key, not the HTTP request node', () => {
		expect(schemeUsageText('odoo_api')).toBe(
			'Utilisable dans : Actions Odoo (partenaires, factures).',
		)
	})

	it('says a kind is planned when no node uses it yet', () => {
		expect(usageSentence(['smtp'])).toBe(
			'Prévue pour : E-mail SMTP. Aucun nœud ne l’utilise encore.',
		)
		expect(usageSentence(['smtp', 'webhook'])).toBe(
			'Utilisable dans : Webhook.',
		)
		expect(usageSentence(['http', 'webhook'])).toBe(
			'Utilisable dans : Requête HTTP et opérations de collection, Webhook.',
		)
	})

	it('lists the four categories of the prototype in order', () => {
		expect(CREDENTIAL_CATEGORIES.map((category) => category.label)).toEqual([
			'Requêtes HTTP',
			'Webhooks entrants',
			'E-mail',
			'Serveurs et bases de données',
		])
	})

	it('groups schemes by category and filters on the query', () => {
		expect(
			searchSchemeGroups([BEARER, UNKNOWN], '').map((group) => group.label),
		).toEqual(['Requêtes HTTP'])
		expect(searchSchemeGroups([BEARER, UNKNOWN], 'jeton')[0]?.schemes).toEqual([
			BEARER,
		])
		expect(searchSchemeGroups([BEARER], 'zzz')).toEqual([])
	})

	it('matches on the label, the description and the category, never on the technical code', () => {
		const localized = [BASIC, BEARER, HEADER].map(localizeScheme)
		const names = (query: string) =>
			searchSchemeGroups(localized, query).flatMap((group) =>
				group.schemes.map((scheme) => scheme.kind),
			)

		expect(names('basic auth')).toEqual(['http_basic'])
		expect(names('en-tête authorization')).toEqual(['http_basic'])
		expect(names('requetes http')).toEqual([
			'http_basic',
			'bearer_token',
			'http_header',
		])
		expect(names('http_basic')).toEqual([])
		expect(names('bearer_token')).toEqual([])
	})

	it('seeds a stable status and usage count', () => {
		expect(seededCredentialMeta('cred-1', 'http_basic')).toEqual(
			seededCredentialMeta('cred-1', 'http_basic'),
		)
		expect(seededCredentialMeta('cred-1', 'oauth2_code').status).toBe(
			'not-connected',
		)
	})

	it('only treats authorization-code oauth as needing a connected account', () => {
		expect(isOauthAuthorization('oauth2_authorization_code')).toBe(true)
		expect(isOauthAuthorization('oauth2_client_credentials')).toBe(false)
		expect(isOauthAuthorization('http_basic')).toBe(false)
	})
})

describe('localizeScheme', () => {
	it('names the kinds and their fields in French, as the prototype does', () => {
		const basic = localizeScheme(BASIC)
		expect(basic.label).toBe('Basic Auth')
		expect(basic.fields.map((entry) => entry.label)).toEqual([
			'Utilisateur',
			'Mot de passe',
		])

		expect(localizeScheme(BEARER).label).toBe('Bearer')
		expect(localizeScheme(BEARER).fields[0]?.label).toBe('Jeton')

		const header = localizeScheme(HEADER)
		expect(header.label).toBe('Clé d’API dans un en-tête')
		expect(header.fields.map((entry) => entry.label)).toEqual([
			'Nom de l’en-tête',
			'Valeur',
		])
	})

	it('gives the header name its default value and leaves the secret without one', () => {
		const header = localizeScheme(HEADER)
		expect(header.fields[0]?.defaultValue).toBe('X-API-Key')
		expect(header.fields[1]?.defaultValue).toBeUndefined()
	})

	it('words the Odoo key in the same tone, with placeholders', () => {
		const odoo = localizeScheme(ODOO)
		expect(odoo.label).toBe('Clé d’API Odoo')
		expect(odoo.fields.map((entry) => entry.label)).toEqual([
			'URL de l’instance',
			'Base de données',
			'Utilisateur',
			'Clé d’API',
		])
		expect(odoo.fields[0]?.placeholder).toBe('https://exemple.odoo.com')
	})

	it('keeps the server wording for a kind it does not know', () => {
		expect(localizeScheme(UNKNOWN)).toEqual(UNKNOWN)
	})

	it('keeps everything else the server said about a field', () => {
		const [token] = localizeScheme(BEARER).fields
		expect(token).toMatchObject({
			name: 'token',
			required: true,
			secret: true,
			kind: 'Text',
		})
	})

	it('is idempotent', () => {
		expect(localizeScheme(localizeScheme(HEADER))).toEqual(
			localizeScheme(HEADER),
		)
	})
})

describe('kindLabel', () => {
	it('prefers the French label, then the one the server sent, then the code', () => {
		expect(kindLabel('http_basic', 'HTTP basic auth')).toBe('Basic Auth')
		expect(kindLabel('http_basic')).toBe('Basic Auth')
		expect(kindLabel('something_new', 'Nouveau')).toBe('Nouveau')
		expect(kindLabel('something_new')).toBe('something_new')
	})
})

describe('newestFirst', () => {
	const at = (id: string, created_at: string) => ({ id, created_at })

	it('puts the most recently created credential at the head', () => {
		expect(
			newestFirst([
				at('a', '2026-03-01T00:00:00Z'),
				at('c', '2026-08-01T00:00:00Z'),
				at('b', '2026-05-01T00:00:00Z'),
			]).map((entry) => entry.id),
		).toEqual(['c', 'b', 'a'])
	})

	it('keeps the given order between credentials created at the same instant', () => {
		expect(
			newestFirst([
				at('a', '2026-03-01T00:00:00Z'),
				at('b', '2026-03-01T00:00:00Z'),
				at('c', '2026-03-01T00:00:00Z'),
			]).map((entry) => entry.id),
		).toEqual(['a', 'b', 'c'])
	})

	it('does not reorder the list it was given', () => {
		const given = [
			at('a', '2026-03-01T00:00:00Z'),
			at('b', '2026-08-01T00:00:00Z'),
		]
		newestFirst(given)
		expect(given.map((entry) => entry.id)).toEqual(['a', 'b'])
	})
})

describe('credential meta — the webhook trigger', () => {
	it('accepts the three header based types and nothing else', () => {
		expect(usableInWebhook('http_header')).toBe(true)
		expect(usableInWebhook('http_basic')).toBe(true)
		expect(usableInWebhook('bearer_token')).toBe(true)
		expect(usableInWebhook('odoo_api')).toBe(false)
		expect(usableInWebhook('unknown')).toBe(false)
	})

	it('hints at the three types the picker of a webhook shows', () => {
		expect(webhookCredentialHint()).toBe(
			'Types acceptés : Basic Auth, Bearer, Clé d’API dans un en-tête.',
		)
	})
})
