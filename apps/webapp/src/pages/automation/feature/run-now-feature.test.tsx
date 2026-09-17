import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { RunNowFeature } from '#/pages/automation/feature/run-now-feature'

const { navigateSpy } = vi.hoisted(() => ({ navigateSpy: vi.fn() }))

vi.mock('@tanstack/react-router', async (importOriginal) => {
	const actual = await importOriginal<typeof import('@tanstack/react-router')>()
	return { ...actual, useNavigate: () => navigateSpy }
})

const EVENTS_PATH = '/api/v1/organizations/{organization_id}/automation/events'
const WORKFLOW_TRIGGER_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}/trigger'
const WORKFLOW_RUNS_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}/runs'
const MY_PERMISSIONS_PATH =
	'/api/v1/organizations/{organization_id}/members/me/permissions'

const ORGANIZATION: Organization = {
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

function event(
	name: string,
	payloadExample: unknown,
): Schemas.EventDescriptorResponse {
	return {
		name,
		label: name,
		subject_kind: name.split('.')[0] ?? name,
		version: 1,
		payload_example: payloadExample,
	}
}

type Handler = (params: unknown) => unknown

interface FakeApi {
	mockGet: (path: string, handler: Handler) => void
	mockMutation: (method: string, path: string, handler: Handler) => void
}

function installFakeTanstackApi(): FakeApi {
	const getHandlers = new Map<string, Handler>()
	const mutationHandlers = new Map<string, Handler>()

	function queryKeyFor(path: string, params: unknown) {
		const p = (params ?? {}) as { path?: unknown; query?: unknown }
		return [{ _id: path, path: p.path, query: p.query }]
	}

	const fakeApi = {
		get(path: string, params: unknown) {
			const queryKey = queryKeyFor(path, params)
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						const handler = getHandlers.get(path)
						if (!handler) throw new Error(`unmocked GET ${path}`)
						return handler(params)
					},
				},
			}
		},
		mutation(method: string, path: string) {
			return {
				mutationOptions: {
					mutationKey: [{ method, path }],
					mutationFn: async (params: unknown) => {
						const handler = mutationHandlers.get(`${method}:${path}`)
						if (!handler)
							throw new Error(`unmocked ${method.toUpperCase()} ${path}`)
						return handler(params)
					},
				},
			}
		},
	}

	// biome-ignore lint/suspicious/noExplicitAny: test-only fake, shape matches TanstackQueryApiClient's used surface
	;(window as any).tanstackApi = fakeApi

	return {
		mockGet: (path, handler) => getHandlers.set(path, handler),
		mockMutation: (method, path, handler) =>
			mutationHandlers.set(`${method}:${path}`, handler),
	}
}

function renderFeature(configure: (api: FakeApi) => void) {
	const api = installFakeTanstackApi()
	api.mockGet(EVENTS_PATH, () => ({
		data: [event('quote.accepted', { quote_id: 'q-1' })],
		pagination: null,
	}))
	api.mockGet(WORKFLOW_TRIGGER_PATH, () => ({
		data: { event_names: ['quote.accepted'] },
		pagination: null,
	}))
	api.mockGet(MY_PERMISSIONS_PATH, () => ({
		data: { permissions: ['MANAGE_AUTOMATION'] },
		pagination: null,
	}))
	configure(api)

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

	render(
		<Providers>
			<RunNowFeature workflowId="workflow-1" />
		</Providers>,
	)

	return api
}

beforeEach(() => {
	navigateSpy.mockReset()
})

describe('RunNowFeature — opening the dialog', () => {
	it('offers "Exécuter maintenant" and opens the dialog warning of a real run', async () => {
		const user = userEvent.setup()
		renderFeature(() => {})

		await user.click(
			await screen.findByRole('button', { name: 'Exécuter maintenant' }),
		)

		expect(await screen.findByRole('alert')).toBeDefined()
	})
})

describe('RunNowFeature — confirming a run', () => {
	it('starts the run through the real mutation without leaving the editor', async () => {
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockMutation('post', WORKFLOW_RUNS_PATH, () => ({
				data: { run_id: 'run-42' },
				pagination: null,
			}))
		})

		await user.click(
			await screen.findByRole('button', { name: 'Exécuter maintenant' }),
		)
		const dialog = await screen.findByRole('dialog')
		await user.click(
			within(dialog).getByRole('button', { name: 'Exécuter maintenant' }),
		)

		await waitFor(() => {
			expect(screen.queryByRole('dialog')).toBeNull()
		})
		expect(navigateSpy).not.toHaveBeenCalled()
	})

	it('shows the failure and keeps the dialog open when the start is refused', async () => {
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockMutation('post', WORKFLOW_RUNS_PATH, () => {
				throw Object.assign(new Error('Service Unavailable'), { status: 503 })
			})
		})

		await user.click(
			await screen.findByRole('button', { name: 'Exécuter maintenant' }),
		)
		const dialog = await screen.findByRole('dialog')
		await user.click(
			within(dialog).getByRole('button', { name: 'Exécuter maintenant' }),
		)

		expect(await screen.findByText('Service Unavailable')).toBeDefined()
		expect(navigateSpy).not.toHaveBeenCalled()
	})
})

describe('RunNowFeature — the execution history link', () => {
	it('navigates to the run list for this workflow', async () => {
		const user = userEvent.setup()
		renderFeature(() => {})

		await user.click(screen.getByRole('button', { name: /Historique/ }))

		expect(navigateSpy).toHaveBeenCalledWith(
			expect.objectContaining({ params: { workflowId: 'workflow-1' } }),
		)
	})
})
