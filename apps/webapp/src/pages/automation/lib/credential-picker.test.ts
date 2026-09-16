import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	allowedCredentialKinds,
	credentialsForAuth,
	generatedCredentials,
} from '#/pages/automation/lib/credential-picker'

function credential(
	id: string,
	kind: string,
	origin: 'supplied' | 'generated' = 'supplied',
): Schemas.CredentialResponse {
	return {
		id,
		kind,
		name: id,
		origin,
		organization_id: 'org-1',
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
	}
}

describe('allowedCredentialKinds', () => {
	it('is empty when the connector accepts no credential', () => {
		expect(allowedCredentialKinds('None')).toEqual([])
	})

	it('is the single scheme when the connector requires exactly one', () => {
		expect(allowedCredentialKinds({ Exactly: 'odoo_api' })).toEqual([
			'odoo_api',
		])
	})

	it('is every listed scheme when the connector accepts any of several', () => {
		expect(
			allowedCredentialKinds({
				AnyOf: ['bearer_token', 'http_basic', 'http_header'],
			}),
		).toEqual(['bearer_token', 'http_basic', 'http_header'])
	})
})

describe('credentialsForAuth', () => {
	const credentials = [
		credential('c1', 'bearer_token'),
		credential('c2', 'http_basic'),
		credential('c3', 'odoo_api'),
		credential('c4', 'http_header'),
	]

	it('offers only the credentials whose kind the AnyOf requirement lists, without naming a connector', () => {
		const auth: Schemas.AuthRequirementResponse = {
			AnyOf: ['bearer_token', 'http_basic', 'http_header'],
		}

		expect(credentialsForAuth(credentials, auth).map((c) => c.id)).toEqual([
			'c1',
			'c2',
			'c4',
		])
	})

	it('offers only the one kind an Exactly requirement names', () => {
		const auth: Schemas.AuthRequirementResponse = { Exactly: 'odoo_api' }

		expect(credentialsForAuth(credentials, auth).map((c) => c.id)).toEqual([
			'c3',
		])
	})

	it('offers nothing when the connector needs no credential', () => {
		expect(credentialsForAuth(credentials, 'None')).toEqual([])
	})
})

describe('generatedCredentials', () => {
	it('keeps only credentials whose origin is generated', () => {
		const credentials = [
			credential('c1', 'bearer_token', 'supplied'),
			credential('c2', 'bearer_token', 'generated'),
			credential('c3', 'http_basic', 'generated'),
		]

		expect(generatedCredentials(credentials).map((c) => c.id)).toEqual([
			'c2',
			'c3',
		])
	})

	it('never offers a supplied credential, even as a free-text-adjacent choice', () => {
		const credentials = [credential('c1', 'bearer_token', 'supplied')]

		expect(generatedCredentials(credentials)).toEqual([])
	})
})
