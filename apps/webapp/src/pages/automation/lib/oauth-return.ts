export type OauthOutcome = 'ok' | 'error'

export interface IdentificationsSearch {
	oauth?: OauthOutcome
}

export function parseIdentificationsSearch(
	raw: Record<string, unknown>,
): IdentificationsSearch {
	return raw.oauth === 'ok' || raw.oauth === 'error' ? { oauth: raw.oauth } : {}
}

function apiOrigin(apiUrl: string | undefined, pageOrigin: string): string {
	const configured = apiUrl?.trim() ?? ''
	return (configured === '' ? pageOrigin : configured).replace(/\/+$/, '')
}

function sameOrigin(origin: string, pageOrigin: string): boolean {
	try {
		return new URL(origin).origin === pageOrigin
	} catch {
		return false
	}
}

export function oauthRedirectUri(
	apiUrl: string | undefined,
	pageOrigin: string,
): string {
	return `${apiOrigin(apiUrl, pageOrigin)}/oauth/callback`
}

export function oauthReturnTo(
	apiUrl: string | undefined,
	pageOrigin: string,
	path: string,
): string {
	return sameOrigin(apiOrigin(apiUrl, pageOrigin), pageOrigin)
		? path
		: `${pageOrigin}${path}`
}

export interface OauthNotice {
	tone: 'success' | 'error'
	text: string
}

export function oauthNotice(outcome: OauthOutcome): OauthNotice {
	return outcome === 'ok'
		? {
				tone: 'success',
				text: 'Compte connecté. L’identification est prête à être utilisée.',
			}
		: {
				tone: 'error',
				text: 'La connexion du compte a échoué. L’autorisation a été refusée ou a expiré : réessayez.',
			}
}
