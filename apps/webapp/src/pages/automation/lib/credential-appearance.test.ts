import { describe, expect, it } from 'vitest'
import {
	credentialAppearance,
	searchAuthSchemes,
} from '#/pages/automation/lib/credential-appearance'

describe('credentialAppearance', () => {
	it('picks a distinct look by the scheme name', () => {
		expect(credentialAppearance('oauth2_client_credentials').hue).toBe(150)
		expect(credentialAppearance('bearer_token').hue).toBe(280)
		expect(credentialAppearance('hmac_signature').hue).toBe(20)
	})

	it('falls back for an unknown scheme', () => {
		expect(credentialAppearance('mystery').hue).toBe(200)
	})
})

describe('searchAuthSchemes', () => {
	const schemes = [
		{ kind: 'bearer', label: 'Jeton porteur' },
		{ kind: 'basic', label: 'Identifiant et mot de passe' },
	]

	it('matches the label ignoring case and accents', () => {
		expect(searchAuthSchemes(schemes, 'JETON')).toEqual([schemes[0]])
		expect(searchAuthSchemes(schemes, 'mot de passe')).toEqual([schemes[1]])
	})

	it('returns everything for an empty query', () => {
		expect(searchAuthSchemes(schemes, '  ')).toEqual(schemes)
	})
})
