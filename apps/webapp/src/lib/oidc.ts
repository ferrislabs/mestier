import { WebStorageStateStore } from 'oidc-client-ts'
import type { AuthProviderProps } from 'react-oidc-context'
import { getOidcConfiguration } from '#/lib/runtime-config'

export function buildOidcProviderProps(): AuthProviderProps | null {
	if (typeof window === 'undefined') return null
	const cfg = getOidcConfiguration()
	if (!cfg) return null

	const postLogoutRedirectUri =
		(import.meta.env.VITE_OIDC_POST_LOGOUT_REDIRECT_URI as
			| string
			| undefined) ?? `${window.location.origin}/`

	return {
		authority: cfg.authority,
		client_id: cfg.client_id,
		redirect_uri: cfg.redirect_uri,
		scope: cfg.scope,
		silent_redirect_uri: cfg.silent_redirect_uri,
		monitorSession: cfg.monitor_session,
		automaticSilentRenew: cfg.automaticSilentRenew ?? true,
		post_logout_redirect_uri: postLogoutRedirectUri,
		response_type: 'code',
		loadUserInfo: true,
		userStore: new WebStorageStateStore({ store: window.localStorage }),
		onSigninCallback:
			cfg.onSigninCallback ??
			(() => {
				window.history.replaceState(
					{},
					document.title,
					window.location.pathname,
				)
			}),
	}
}
