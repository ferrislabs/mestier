import { describe, expect, it } from 'vitest'
import type { AuthScheme } from '#/hooks/use-automation'
import {
	isOauthAuthorization,
	schemeDescription,
	schemeUsageText,
	searchSchemeGroups,
	seededCredentialMeta,
} from '#/pages/automation/lib/credential-meta'

const BEARER: AuthScheme = {
	kind: 'bearer_token',
	label: 'Bearer token',
	fields: [
		{
			name: 'token',
			label: 'Jeton',
			required: true,
			kind: 'Text',
			expression: false,
			secret: true,
		},
	],
}

const UNKNOWN: AuthScheme = {
	kind: 'something_new',
	label: 'Nouveau',
	fields: [
		{
			name: 'a',
			label: 'Champ A',
			required: true,
			kind: 'Text',
			expression: false,
			secret: false,
		},
	],
}

describe('credential meta', () => {
	it('describes known kinds in prose and unknown ones by their fields', () => {
		expect(schemeDescription(BEARER)).toBe(
			'Un jeton envoyé sous la forme « Authorization: Bearer … ».',
		)
		expect(schemeDescription(UNKNOWN)).toBe('Champ A')
	})

	it('states where a kind can be used', () => {
		expect(schemeUsageText('http_basic')).toBe(
			'Utilisable dans : Requête HTTP et opérations de collection, Webhook.',
		)
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
