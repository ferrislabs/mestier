import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { describe, expect, it } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import { useVisibleModules } from '#/modules/use-visible-modules'
import { FeaturesSection } from '#/pages/settings/sections/features-section'
import { renderWithRouter } from '#/test/render-with-router'
import { seedPermissionsCacheForOrganization } from '#/test/with-permissions'

const FEATURES_PATH = '/api/v1/organizations/{organization_id}/features'
const FEATURE_PATH = '/api/v1/organizations/{organization_id}/features/{flag}'

const ORGANIZATION = {
	id: 'org-1',
	name: 'Atelier Bois & Co',
	slug: 'atelier-bois',
	owner_id: 'user-1',
	missing_legal_identity_fields: [],
	field_clock_enabled: false,
	vat_on_debits: false,
	mcp_enabled: false,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

const SURVEYS = {
	key: 'surveys',
	label: 'Questionnaires',
	description:
		'Questionnaires et campagnes de réponses, ouverts au public ou aux membres.',
}

interface FakeBackend {
	enabled: boolean
	putError?: Error
	holdPut?: Promise<void>
}

function installFakeTanstackApi(backend: FakeBackend) {
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
						if (path === FEATURES_PATH) {
							return {
								data: {
									features: backend.enabled ? ['surveys'] : [],
									self_service: [{ ...SURVEYS, enabled: backend.enabled }],
								},
							}
						}
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
						if (method === 'put' && path === FEATURE_PATH) {
							if (backend.holdPut) await backend.holdPut
							if (backend.putError) throw backend.putError
							const { enabled } = (params as { body: { enabled: boolean } })
								.body
							backend.enabled = enabled
							return { data: { key: 'surveys', enabled, changed: true } }
						}
						throw new Error(`unmocked mutation ${method} ${path}`)
					},
				},
			}
		},
	}

	Object.assign(window, { tanstackApi: fakeApi })

	return calls
}

function VisibleModules() {
	const modules = useVisibleModules()
	return (
		<ul aria-label="Modules visibles">
			{modules.map((module) => (
				<li key={module.id}>{module.label}</li>
			))}
		</ul>
	)
}

interface RenderOptions extends Partial<FakeBackend> {
	permissions?: string[]
}

async function renderSection({
	permissions = ['MANAGE_ORG', 'VIEW_SURVEYS'],
	...rest
}: RenderOptions = {}) {
	const backend: FakeBackend = { enabled: false, ...rest }
	const calls = installFakeTanstackApi(backend)
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	seedPermissionsCacheForOrganization(queryClient, ORGANIZATION.id, permissions)

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
			<VisibleModules />
			<FeaturesSection />
		</Providers>,
	)
	return { ...result, calls, backend }
}

function putCalls(calls: { method: string; path: string; params: unknown }[]) {
	return calls.filter(
		(call) => call.method === 'put' && call.path === FEATURE_PATH,
	)
}

describe('FeaturesSection', () => {
	it('shows each self-service feature with its label, description and a switch', async () => {
		await renderSection()

		const toggle = await screen.findByRole('switch', { name: 'Questionnaires' })
		expect(toggle.getAttribute('aria-checked')).toBe('false')
		expect(screen.getByText(SURVEYS.description)).toBeDefined()
	})

	it('turns surveys on and the Questionnaires module appears without a reload', async () => {
		const user = userEvent.setup()
		const { calls } = await renderSection()
		const modules = screen.getByRole('list', { name: 'Modules visibles' })

		await user.click(
			await screen.findByRole('switch', { name: 'Questionnaires' }),
		)

		await waitFor(() => {
			expect(
				screen
					.getByRole('switch', { name: 'Questionnaires' })
					.getAttribute('aria-checked'),
			).toBe('true')
		})
		expect(putCalls(calls)[0]?.params).toEqual({
			path: { organization_id: 'org-1', flag: 'surveys' },
			body: { enabled: true },
		})
		await waitFor(() => {
			expect(modules.textContent).toContain('Questionnaires')
		})
	})

	it('turns surveys off and the Questionnaires module disappears', async () => {
		const user = userEvent.setup()
		await renderSection({ enabled: true })
		const modules = screen.getByRole('list', { name: 'Modules visibles' })
		await waitFor(() => {
			expect(modules.textContent).toContain('Questionnaires')
		})

		await user.click(screen.getByRole('switch', { name: 'Questionnaires' }))

		await waitFor(() => {
			expect(modules.textContent).not.toContain('Questionnaires')
		})
	})

	it('disables the switch while the change is pending', async () => {
		const user = userEvent.setup()
		let release = () => {}
		const holdPut = new Promise<void>((resolve) => {
			release = resolve
		})
		await renderSection({ holdPut })

		const toggle = await screen.findByRole('switch', { name: 'Questionnaires' })
		await user.click(toggle)

		await waitFor(() => {
			expect((toggle as HTMLButtonElement).disabled).toBe(true)
		})
		expect(toggle.getAttribute('aria-checked')).toBe('false')
		release()
		await waitFor(() => {
			expect(toggle.getAttribute('aria-checked')).toBe('true')
		})
	})

	it('shows the refusal and keeps the previous state when the change fails', async () => {
		const user = userEvent.setup()
		await renderSection({
			putError: Object.assign(new Error('Forbidden'), { status: 403 }),
		})

		const toggle = await screen.findByRole('switch', { name: 'Questionnaires' })
		await user.click(toggle)

		expect((await screen.findByRole('alert')).textContent).toContain(
			"Vous n'avez plus la permission nécessaire pour cette action.",
		)
		expect(toggle.getAttribute('aria-checked')).toBe('false')

		await user.click(screen.getByRole('button', { name: 'Fermer le message' }))
		expect(screen.queryByRole('alert')).toBeNull()
	})

	it('shows the state without any switch to a member without MANAGE_ORG', async () => {
		const { calls } = await renderSection({ permissions: ['VIEW_SURVEYS'] })

		expect(await screen.findByText('Désactivée')).toBeDefined()
		expect(screen.getByText('Questionnaires', { selector: 'h3' })).toBeDefined()
		expect(screen.queryByRole('switch')).toBeNull()
		expect(
			screen.getByText(
				"Seuls les membres autorisés à gérer l'organisation peuvent activer ou désactiver une fonctionnalité.",
			),
		).toBeDefined()
		expect(putCalls(calls)).toHaveLength(0)
	})
})
