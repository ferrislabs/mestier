import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

// Regression coverage for the incident this module caused: `client_id` used
// to come only from `import.meta.env.VITE_OIDC_CLIENT_ID`, a Vite
// build-time constant. The production image is built once and reused
// across every environment, so that constant is never set there — the app
// silently ran with `client_id: undefined` in every real deployment.
// `client_id` now follows `api_url`/`issuer_url`'s existing split: dev reads
// the Vite env var, everything else reads `/config.json` at runtime — see
// `loadRuntimeConfig`.
describe('loadRuntimeConfig (production mode)', () => {
	beforeEach(() => {
		vi.resetModules()
		// @ts-expect-error test-only override of a Vite-injected constant —
		// vitest defaults it to true (dev/test mode).
		import.meta.env.DEV = false
	})

	afterEach(() => {
		vi.unstubAllGlobals()
		// @ts-expect-error see above
		import.meta.env.DEV = true
	})

	it('reads client_id from /config.json, not from the build-time env var', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue({
				ok: true,
				json: async () => ({
					api_url: 'https://api.mestier.fr',
					issuer_url: 'https://auth.mestier.fr/realms/mestier',
					client_id: 'mestier',
				}),
			}),
		)

		const { loadRuntimeConfig, getOidcConfiguration } = await import(
			'#/lib/runtime-config'
		)
		await loadRuntimeConfig()

		expect(getOidcConfiguration()).toMatchObject({
			authority: 'https://auth.mestier.fr/realms/mestier',
			client_id: 'mestier',
		})
	})

	it('treats an unsubstituted OIDC_CLIENT_ID placeholder as absent, leaving auth unconfigured', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn().mockResolvedValue({
				ok: true,
				json: async () => ({
					api_url: 'https://api.mestier.fr',
					issuer_url: 'https://auth.mestier.fr/realms/mestier',
					// What's actually in config.json when the infra forgets to set
					// OIDC_CLIENT_ID: docker-entrypoint.sh's sed substitution has
					// nothing to replace it with.
					// biome-ignore lint/suspicious/noTemplateCurlyInString: literal placeholder text left by the entrypoint script, not a template
					client_id: '${OIDC_CLIENT_ID}',
				}),
			}),
		)

		const { loadRuntimeConfig, getOidcConfiguration } = await import(
			'#/lib/runtime-config'
		)
		await loadRuntimeConfig()

		expect(getOidcConfiguration()).toBeUndefined()
	})
})
