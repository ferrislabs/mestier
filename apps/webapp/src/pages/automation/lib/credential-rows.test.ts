import { describe, expect, it } from 'vitest'
import {
	credentialRow,
	credentialRows,
} from '#/pages/automation/lib/credential-rows'
import { credential, KINDS } from '#/pages/automation/test/credential-fixtures'

describe('credentialRow', () => {
	it('carries what the server says: description, status, account, usage and secrets', () => {
		const row = credentialRow(
			credential({
				kind: 'oauth2_authorization_code',
				name: 'Google',
				description: 'Agenda de l’équipe',
				status: 'expired',
				account: 'a@exemple.fr',
				used_by_workflows: 3,
				secrets_set: ['client_secret'],
				data: { client_id: 'abc' },
			}),
			KINDS,
		)

		expect(row).toMatchObject({
			name: 'Google',
			kindLabel: 'OAuth2',
			summary: 'Connecté : a@exemple.fr',
			description: 'Agenda de l’équipe',
			status: 'expired',
			account: 'a@exemple.fr',
			usedByWorkflows: 3,
			secretsSet: ['client_secret'],
			data: { client_id: 'abc' },
		})
	})

	it('reads the origin the server sends, supplied for anything else', () => {
		expect(
			credentialRow(credential({ origin: 'generated' }), KINDS).origin,
		).toBe('generated')
		expect(credentialRow(credential({ origin: 'other' }), KINDS).origin).toBe(
			'supplied',
		)
	})

	it('shows the code of a kind the catalogue does not know', () => {
		expect(
			credentialRow(credential({ kind: 'mystery' }), KINDS).kindLabel,
		).toBe('mystery')
	})
})

describe('credentialRows', () => {
	it('lists the most recently created first', () => {
		const rows = credentialRows(
			[
				credential({ id: 'old', created_at: '2026-01-01T00:00:00Z' }),
				credential({ id: 'new', created_at: '2026-08-01T00:00:00Z' }),
				credential({ id: 'mid', created_at: '2026-04-01T00:00:00Z' }),
			],
			KINDS,
		)

		expect(rows.map((row) => row.id)).toEqual(['new', 'mid', 'old'])
	})
})
