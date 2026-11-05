import { describe, expect, it } from 'vitest'
import {
	curlExample,
	EXAMPLE_PAYLOAD,
	generateSecret,
	validateApiPayload,
} from '#/pages/support/lib/api-example'

describe('validateApiPayload', () => {
	it('accepts the documented example', () => {
		const result = validateApiPayload(EXAMPLE_PAYLOAD)

		expect(result).toEqual({
			ok: true,
			payload: {
				subject: 'Volet roulant bloqué',
				body: 'Le volet du salon ne remonte plus depuis ce matin.',
				requesterEmail: 'claire.fontaine@fontaine-immo.fr',
				requesterName: 'Claire Fontaine',
				externalId: 'EC-20450',
			},
		})
	})

	it('lists every missing required field', () => {
		expect(validateApiPayload({})).toEqual({
			ok: false,
			errors: [
				'subject est obligatoire.',
				'body est obligatoire.',
				'requester.email est obligatoire.',
			],
		})
	})

	it('rejects an e-mail without an arobase', () => {
		expect(
			validateApiPayload({
				subject: 'a',
				body: 'b',
				requester: { email: 'nope' },
			}),
		).toEqual({ ok: false, errors: ['requester.email est invalide.'] })
	})

	it('rejects a body that is not an object', () => {
		expect(validateApiPayload([1])).toEqual({
			ok: false,
			errors: ['Le corps doit être un objet JSON.'],
		})
	})
})

describe('curlExample and generateSecret', () => {
	it('shows the key prefix in the authorization header', () => {
		expect(curlExample('https://app.test', 'mst_live_ab')).toContain(
			'Bearer mst_live_ab…',
		)
	})

	it('builds a prefixed 32 character secret', () => {
		const secret = generateSecret(() => 0)

		expect(secret).toBe(`mst_live_${'A'.repeat(32)}`)
	})
})
