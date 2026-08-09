import { UserManager, WebStorageStateStore } from 'oidc-client-ts'
import { getOidcConfiguration } from '#/lib/runtime-config'

/**
 * FerrisKey fait tourner le jeton de rafraîchissement à chaque usage et révoque
 * toute la famille au moindre rejeu — un jeton déjà consommé n'est pas une
 * erreur récupérable, c'est la fin de la session. Comme l'access token ne vit
 * que cinq minutes, la webapp rafraîchit toutes les quatre minutes : deux
 * renouvellements qui se chevauchent suffisent à tuer la session entière.
 *
 * D'où ce module. Il ne reste qu'un seul renouveleur, sur deux axes :
 *  - un seul `UserManager` par document, car `react-oidc-context` n'appelle
 *    jamais `stopSilentRenew()` au démontage : chaque remontage (le HMR en dev)
 *    laisserait sinon un minuteur orphelin sur le même jeton ;
 *  - un seul onglet renouvelle à la fois, désigné par un Web Lock ; les autres
 *    adoptent le jeton que le leader vient d'écrire.
 */

/** Nom du verrou désignant l'onglet renouveleur. */
const RENEWAL_LOCK = 'mestier.oidc.silent-renew'

/**
 * Clé sous laquelle `oidc-client-ts` range l'utilisateur : `WebStorageStateStore`
 * préfixe par `oidc.`, `UserManager` compose le reste. On la recalcule pour
 * écouter les renouvellements des autres onglets, et un test vérifie qu'elle
 * correspond toujours à ce que la librairie écrit réellement.
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
		// Le renouvellement est démarré par `leadRenewalsForThisTab`, une fois le
		// verrou obtenu — jamais par le constructeur, qui l'accorderait à tous.
		automaticSilentRenew: false,
	})
}

/**
 * Prend le rôle de renouveleur dès que le verrou est libre, et le garde jusqu'à
 * la fermeture de l'onglet : la promesse rendue au navigateur ne se résout
 * jamais. Quand le leader disparaît, le verrou l'est aussi et un autre onglet
 * prend la relève — `SilentRenewService.start()` relit alors le jeton courant.
 */
export function leadRenewalsForThisTab(
	userManager: UserManager,
): Promise<void> {
	if (typeof navigator === 'undefined' || !navigator.locks) {
		// Sans Web Locks, on ne sait pas s'entendre entre onglets : mieux vaut
		// renouveler et risquer la collision que laisser la session mourir.
		userManager.startSilentRenew()
		return Promise.resolve()
	}

	return navigator.locks.request<void>(RENEWAL_LOCK, async () => {
		userManager.startSilentRenew()
		await new Promise<never>(() => {})
	})
}

/**
 * Aligne cet onglet sur le jeton écrit par le leader. Sans ça un onglet suiveur
 * garderait en mémoire l'access token périmé et enverrait des requêtes vouées
 * au 401, alors qu'un jeton frais l'attend dans `localStorage`.
 *
 * Rend la fonction de désabonnement.
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

		// Valeur effacée : le leader s'est déconnecté, ou sa session est morte.
		// Suivre plutôt que de continuer avec un jeton que l'IdP a déjà oublié.
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
 * Le `UserManager` du document. `undefined` signifie « pas encore construit »,
 * `null` « authentification non configurée » — les distinguer évite de
 * reconstruire à chaque rendu quand la configuration est absente.
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
