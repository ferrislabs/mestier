import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import { AutomationSection } from '#/pages/settings/sections/automation-section'
import { renderWithRouter } from '#/test/render-with-router'

/**
 * jsdom has no pointer-capture APIs or `scrollIntoView`. Radix's `Select`
 * calls both when an item is actually *selected* — opening the popover
 * alone never reaches this path, which is why no other test in this repo
 * needed it before this file, the first to click a `SelectItem` rather
 * than just asserting it exists.
 *
 * Deliberately scoped to this file rather than `vitest.setup.ts`: filling
 * the gap globally changes real (if accidental) behavior every other test
 * that opens a Radix `Select`/`Popover` was already running against —
 * tried once, reverted after it broke 24 unrelated tests across 9 files.
 */
for (const method of [
	'hasPointerCapture',
	'setPointerCapture',
	'releasePointerCapture',
	'scrollIntoView',
] as const) {
	if (typeof Element.prototype[method] !== 'function') {
		Element.prototype[method] = (() => false) as never
	}
}

const SETTINGS_PATH =
	'/api/v1/organizations/{organization_id}/automation/settings'

const ORGANIZATION = {
	id: 'org-1',
	name: 'Atelier Bois & Co',
	slug: 'atelier-bois',
	owner_id: 'user-1',
	missing_legal_identity_fields: [],
	field_clock_enabled: false,
	vat_on_debits: false,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

const SETTINGS = {
	event_retention_seconds: 7_776_000,
	succeeded_run_retention_seconds: 2_592_000,
	retry_schedule_seconds: [5, 30],
	disable_target_after: 20,
}

interface FakeApiHandlers {
	putSettings?: (params: unknown) => unknown
}

function installFakeTanstackApi(handlers: FakeApiHandlers = {}) {
	const calls: { method: string; path: string; params: unknown }[] = []

	const fakeApi = {
		get(path: string, params: unknown) {
			const queryKey = [
				{ _id: path, path: (params as { path?: unknown })?.path },
			]
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						calls.push({ method: 'get', path, params })
						if (path === SETTINGS_PATH) return { data: SETTINGS }
						throw new Error(`unmocked GET ${path}`)
					},
				},
			}
		},
		mutation(method: string, path: string) {
			const mutationKey = [{ method, path }]
			return {
				mutationKey,
				mutationOptions: {
					mutationKey,
					mutationFn: async (params: unknown) => {
						calls.push({ method, path, params })
						if (method === 'put' && path === SETTINGS_PATH) {
							if (!handlers.putSettings)
								throw new Error('putSettings not mocked')
							return handlers.putSettings(params)
						}
						throw new Error(`unmocked mutation ${method} ${path}`)
					},
				},
			}
		},
	}

	// biome-ignore lint/suspicious/noExplicitAny: test-only fake, shape matches TanstackQueryApiClient's used surface
	;(window as any).tanstackApi = fakeApi

	return calls
}

async function renderSection(handlers: FakeApiHandlers = {}) {
	const calls = installFakeTanstackApi(handlers)
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})

	function Providers({ children }: { children: ReactNode }) {
		return (
			<QueryClientProvider client={queryClient}>
				<OrganizationListProvider organizations={[ORGANIZATION]}>
					<ActiveOrganizationProvider activeOrganization={ORGANIZATION}>
						{children}
					</ActiveOrganizationProvider>
				</OrganizationListProvider>
			</QueryClientProvider>
		)
	}

	const result = await renderWithRouter(
		<Providers>
			<AutomationSection />
		</Providers>,
	)
	return { ...result, calls }
}

describe('AutomationSection — settings', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('loads the existing settings into the form', async () => {
		await renderSection({})

		expect(await screen.findByDisplayValue('7776000')).toBeDefined()
		expect(screen.getByDisplayValue('5, 30')).toBeDefined()
	})

	it('refuses an invalid retry schedule without calling the API', async () => {
		const user = userEvent.setup()
		const { calls } = await renderSection({})

		const field = await screen.findByLabelText(
			'Plan de nouvelles tentatives (secondes, séparées par des virgules)',
		)
		await user.clear(field)
		await user.type(field, '5, abc')
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		expect(
			screen.getByText(/liste de secondes séparées par des virgules/),
		).toBeDefined()
		expect(
			calls.some((c) => c.method === 'put' && c.path === SETTINGS_PATH),
		).toBe(false)
	})

	it('submits the parsed body on a valid change', async () => {
		const user = userEvent.setup()
		const { calls } = await renderSection({
			putSettings: (params) => ({ data: (params as { body: unknown }).body }),
		})

		const field = await screen.findByLabelText('Rétention des événements')
		await user.clear(field)
		await user.type(field, '864000')
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		await waitFor(() => {
			const call = calls.find(
				(c) => c.method === 'put' && c.path === SETTINGS_PATH,
			)
			expect(call?.params).toMatchObject({
				path: { organization_id: 'org-1' },
				body: { event_retention_seconds: 864_000 },
			})
		})
	})
})
