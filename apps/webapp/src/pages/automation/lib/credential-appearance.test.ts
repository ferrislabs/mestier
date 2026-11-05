import { Boxes, Fingerprint, KeyRound, LockKeyhole, Ticket } from 'lucide-react'
import { describe, expect, it } from 'vitest'
import {
	credentialAppearance,
	searchAuthSchemes,
} from '#/pages/automation/lib/credential-appearance'

describe('credentialAppearance', () => {
	it('gives each kind the icon and hue the prototype gives its equivalent', () => {
		expect(credentialAppearance('http_basic')).toEqual({
			icon: LockKeyhole,
			hue: 250,
		})
		expect(credentialAppearance('bearer_token')).toEqual({
			icon: Ticket,
			hue: 280,
		})
		expect(credentialAppearance('http_header')).toEqual({
			icon: KeyRound,
			hue: 200,
		})
	})

	it('matches the Odoo connectors look for the Odoo key', () => {
		expect(credentialAppearance('odoo_api')).toEqual({ icon: Boxes, hue: 300 })
	})

	it('no longer guesses a look from words in the code', () => {
		expect(credentialAppearance('digest_auth').icon).not.toBe(Fingerprint)
		expect(credentialAppearance('oauth2_client_credentials')).toEqual(
			credentialAppearance('mystery'),
		)
	})

	it('falls back for an unknown scheme', () => {
		expect(credentialAppearance('mystery')).toEqual({
			icon: KeyRound,
			hue: 200,
		})
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

	it('does not match the technical code', () => {
		expect(searchAuthSchemes(schemes, 'bearer')).toEqual([])
		expect(searchAuthSchemes(schemes, 'basic')).toEqual([])
	})

	it('returns everything for an empty query', () => {
		expect(searchAuthSchemes(schemes, '  ')).toEqual(schemes)
	})
})
