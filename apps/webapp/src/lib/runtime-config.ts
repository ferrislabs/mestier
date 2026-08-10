import { createClientOnlyFn } from '@tanstack/react-start'

type ApiClient = import('#/api/api.client').ApiClient
type TanstackQueryApiClient =
	import('#/api/api.tanstack').TanstackQueryApiClient

export interface OidcConfiguration {
	authority: string
	client_id: string
	redirect_uri: string
	scope: string
	silent_redirect_uri?: string
	monitor_session?: boolean
}

declare global {
	interface Window {
		api: ApiClient
		tanstackApi: TanstackQueryApiClient
		apiUrl: string
		issuerUrl?: string
		oidcConfiguration?: OidcConfiguration
		inDevelopmentMode: boolean
	}
}

interface RawConfig {
	api_url?: string
	issuer_url?: string
}

let loadingPromise: Promise<void> | null = null

const initializeApiClients = createClientOnlyFn(async (apiUrl: string) => {
	const [{ createApiClient }, { fetcher }, { TanstackQueryApiClient }] =
		await Promise.all([
			import('#/api/api.client'),
			import('#/api/api.fetch'),
			import('#/api/api.tanstack'),
		])
	const api = createApiClient({ fetch: fetcher }).setBaseUrl(apiUrl)
	window.api = api
	window.tanstackApi = new TanstackQueryApiClient(api)
})

export function loadRuntimeConfig(): Promise<void> {
	if (typeof window === 'undefined') return Promise.resolve()
	if (loadingPromise) return loadingPromise

	loadingPromise = (async () => {
		const isDev = import.meta.env.DEV
		window.inDevelopmentMode = isDev

		let apiUrl: string | undefined
		let issuerUrl: string | undefined

		if (isDev) {
			apiUrl = import.meta.env.VITE_API_URL as string | undefined
			issuerUrl = import.meta.env.VITE_OIDC_AUTHORITY as string | undefined
		} else {
			try {
				const res = await fetch('/config.json', { cache: 'no-store' })
				if (res.ok) {
					const data: RawConfig = await res.json()
					apiUrl = isPlaceholder(data.api_url) ? undefined : data.api_url
					issuerUrl = isPlaceholder(data.issuer_url)
						? undefined
						: data.issuer_url
				}
			} catch (err) {
				console.error('Failed to load /config.json', err)
			}
		}

		window.apiUrl = apiUrl ?? ''
		window.issuerUrl = issuerUrl
		await initializeApiClients(apiUrl ?? '')

		const clientId = import.meta.env.VITE_OIDC_CLIENT_ID as string | undefined
		const scope =
			(import.meta.env.VITE_OIDC_SCOPE as string | undefined) ??
			'openid profile email'
		const redirectUri =
			(import.meta.env.VITE_OIDC_REDIRECT_URI as string | undefined) ??
			`${window.location.origin}/`

		if (issuerUrl && clientId) {
			window.oidcConfiguration = {
				authority: issuerUrl,
				client_id: clientId,
				redirect_uri: redirectUri,
				scope,
			}
		} else {
			window.oidcConfiguration = undefined
		}
	})()

	return loadingPromise
}

function isPlaceholder(value: string | undefined): boolean {
	if (!value) return true
	return value.startsWith('${') && value.endsWith('}')
}

export function getOidcConfiguration(): OidcConfiguration | undefined {
	if (typeof window === 'undefined') return undefined
	return window.oidcConfiguration
}

export function isDevelopmentMode(): boolean {
	if (typeof window === 'undefined') return import.meta.env.DEV
	return window.inDevelopmentMode
}
