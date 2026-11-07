import { describe, expect, it } from 'vitest'
import {
	oauthNotice,
	oauthRedirectUri,
	oauthReturnTo,
	parseIdentificationsSearch,
} from '#/pages/automation/lib/oauth-return'

const PATH = '/o/atelier/automatisation/identifications'

describe('parseIdentificationsSearch', () => {
	it('keeps the outcome the server redirects with', () => {
		expect(parseIdentificationsSearch({ oauth: 'ok' })).toEqual({ oauth: 'ok' })
		expect(parseIdentificationsSearch({ oauth: 'error' })).toEqual({
			oauth: 'error',
		})
	})

	it('drops anything else', () => {
		expect(parseIdentificationsSearch({ oauth: 'maybe' })).toEqual({})
		expect(parseIdentificationsSearch({ oauth: ['ok'] })).toEqual({})
		expect(parseIdentificationsSearch({})).toEqual({})
	})
})

describe('oauthRedirectUri', () => {
	it('is the callback of the public API', () => {
		expect(
			oauthRedirectUri('https://api.mestier.fr', 'https://app.mestier.fr'),
		).toBe('https://api.mestier.fr/oauth/callback')
	})

	it('ignores a trailing slash', () => {
		expect(
			oauthRedirectUri('https://api.mestier.fr/', 'https://app.mestier.fr'),
		).toBe('https://api.mestier.fr/oauth/callback')
	})

	it('falls back to the origin of the page when the API shares it', () => {
		expect(oauthRedirectUri('', 'https://app.mestier.fr')).toBe(
			'https://app.mestier.fr/oauth/callback',
		)
		expect(oauthRedirectUri(undefined, 'https://app.mestier.fr')).toBe(
			'https://app.mestier.fr/oauth/callback',
		)
	})
})

describe('oauthReturnTo', () => {
	it('is the local path when the API answers on the origin of the page', () => {
		expect(oauthReturnTo('', 'https://app.mestier.fr', PATH)).toBe(PATH)
		expect(
			oauthReturnTo('https://app.mestier.fr', 'https://app.mestier.fr', PATH),
		).toBe(PATH)
	})

	it('is the address of the page when the API answers elsewhere, since the redirect comes from the API', () => {
		expect(
			oauthReturnTo('https://api.mestier.fr', 'https://app.mestier.fr', PATH),
		).toBe(`https://app.mestier.fr${PATH}`)
		expect(
			oauthReturnTo('http://localhost:3456', 'http://localhost:3000', PATH),
		).toBe(`http://localhost:3000${PATH}`)
	})
})

describe('oauthNotice', () => {
	it('welcomes a connected account', () => {
		expect(oauthNotice('ok')).toEqual({
			tone: 'success',
			text: 'Compte connecté. L’identification est prête à être utilisée.',
		})
	})

	it('tells a failed connection to be retried', () => {
		expect(oauthNotice('error').tone).toBe('error')
		expect(oauthNotice('error').text).toMatch(/réessayez/)
	})
})
