import { UserManager, WebStorageStateStore } from 'oidc-client-ts'
import { getOidcConfiguration } from '#/lib/runtime-config'

/**
 * FerrisKey rotates the refresh token on every use and revokes the whole family
 * on any replay — a token that has already been spent is not a recoverable
 * error, it is the end of the session. Since the access token lives only five
 * minutes, the webapp refreshes every four: two renewals overlapping is all it
 * takes to kill the session outright.
 *
 * Hence this module. It keeps exactly one renewer, along two axes:
 *  - one `UserManager` per document, because `react-oidc-context` never calls
 *    `stopSilentRenew()` on unmount: every remount (HMR in dev) would otherwise
 *    strand a live timer on the same token;
 *  - one tab renewing at a time, elected by a Web Lock; the others adopt the
 *    token the leader just wrote.
 */

/** Name of the lock that elects the renewing tab. */
const RENEWAL_LOCK = 'mestier.oidc.silent-renew'

/**
 * The key `oidc-client-ts` files the user under: `WebStorageStateStore` prefixes
 * with `oidc.`, `UserManager` composes the rest. We recompute it to listen for
 * other tabs' renewals, and a test checks it still matches what the library
 * actually writes.
 */
export function userStorageKey(authority: string, clientId: string): string {
	return `oidc.user:${authority}:${clientId}`
}

export function createUserManager(): UserManager | null {
	if (typeof window === 'undefined') return null
	const cfg = getOidcConfiguration()
	if (!cfg) return null

	const postLogoutRedirectUri =
		(import.meta.env.VITE_OIDC_POST_LOGOUT_REDIRECT_URI as
			| string
			| undefined) ?? `${window.location.origin}/`

	return new UserManager({
		authority: cfg.authority,
		client_id: cfg.client_id,
		redirect_uri: cfg.redirect_uri,
		scope: cfg.scope,
		silent_redirect_uri: cfg.silent_redirect_uri,
		monitorSession: cfg.monitor_session,
		post_logout_redirect_uri: postLogoutRedirectUri,
		response_type: 'code',
		loadUserInfo: true,
		userStore: new WebStorageStateStore({ store: window.localStorage }),
		// Renewal is started by `leadRenewalsForThisTab` once the lock is held —
		// never by the constructor, which would grant it to every tab.
		automaticSilentRenew: false,
	})
}

/**
 * Takes the renewer role as soon as the lock is free and holds it until the tab
 * closes: the promise handed back to the browser never settles. When the leader
 * goes away so does the lock, and another tab takes over —
 * `SilentRenewService.start()` then re-reads the current token.
 */
export function leadRenewalsForThisTab(
	userManager: UserManager,
): Promise<void> {
	if (typeof navigator === 'undefined' || !navigator.locks) {
		// Without Web Locks there is no way for tabs to agree: better to renew
		// and risk the collision than to let the session die.
		userManager.startSilentRenew()
		return Promise.resolve()
	}

	return navigator.locks.request<void>(RENEWAL_LOCK, async () => {
		userManager.startSilentRenew()
		await new Promise<never>(() => {})
	})
}

/**
 * Aligns this tab on the token the leader wrote. Without it a follower would
 * keep the stale access token in memory and send requests doomed to 401 while a
 * fresh token sits waiting in `localStorage`.
 *
 * Returns the unsubscribe function.
 */
export function adoptRenewalsFromOtherTabs(
	userManager: UserManager,
): () => void {
	const key = userStorageKey(
		userManager.settings.authority,
		userManager.settings.client_id,
	)

	const onStorage = (event: StorageEvent) => {
		if (event.key !== key) return

		// Cleared value: the leader signed out, or its session died. Follow it
		// out rather than carry on with a token the IdP has already forgotten.
		if (event.newValue === null) {
			void userManager.events.unload()
			return
		}

		void userManager.getUser(true)
	}

	window.addEventListener('storage', onStorage)
	return () => window.removeEventListener('storage', onStorage)
}

let instance: UserManager | null | undefined

/**
 * The document's `UserManager`. `undefined` means "not built yet", `null` means
 * "authentication not configured" — telling them apart avoids rebuilding on
 * every render when the configuration is missing.
 */
export function getUserManager(): UserManager | null {
	if (instance !== undefined) return instance

	instance = createUserManager()
	if (instance) {
		adoptRenewalsFromOtherTabs(instance)
		void leadRenewalsForThisTab(instance)
	}

	return instance
}
