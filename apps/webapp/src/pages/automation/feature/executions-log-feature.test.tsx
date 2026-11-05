import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { ExecutionsLogFeature } from '#/pages/automation/feature/executions-log-feature'
import { installFakeTanstackApi } from '#/pages/automation/test/fake-tanstack-api'
import { renderWithRouter } from '#/test/render-with-router'

const WORKFLOWS_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows'
const RUNS_PATH = '/api/v1/organizations/{organization_id}/automation/runs'

const ORGANIZATION: Organization = {
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

function run(overrides: Record<string, unknown>) {
	return {
		id: 'run-1',
		organization_id: 'org-1',
		workflow_id: 'wf-1',
		workflow_version_id: 'v1v1v1v1',
		status: 'succeeded',
		created_at: '2026-08-01T00:00:00Z',
		started_at: '2026-08-01T00:00:00Z',
		finished_at: '2026-08-01T00:00:03Z',
		...overrides,
	}
}

function render(onFiltersChange = vi.fn()) {
	const api = installFakeTanstackApi()
	api.mockGet(WORKFLOWS_PATH, () => ({
		data: [
			{
				id: 'wf-1',
				organization_id: 'org-1',
				name: 'Relance des devis',
				enabled: true,
				created_at: '2026-08-01T00:00:00Z',
				updated_at: '2026-08-01T00:00:00Z',
			},
		],
	}))
	api.mockGet(RUNS_PATH, () => ({
		data: [
			run({ id: 'aaaaaaaa-1' }),
			run({ id: 'bbbbbbbb-2', status: 'failed', error: 'SMTP expiré' }),
			run({ id: 'cccccccc-3', workflow_id: 'gone' }),
		],
	}))

	function Providers({ children }: { children: ReactNode }) {
		return (
			<QueryClientProvider
				client={
					new QueryClient({ defaultOptions: { queries: { retry: false } } })
				}
			>
				<OrganizationListProvider organizations={[ORGANIZATION]}>
					<ActiveOrganizationProvider activeOrganization={ORGANIZATION}>
						{children}
					</ActiveOrganizationProvider>
				</OrganizationListProvider>
			</QueryClientProvider>
		)
	}

	return {
		onFiltersChange,
		rendered: renderWithRouter(
			<Providers>
				<ExecutionsLogFeature
					filters={{
						status: 'all',
						workflow: 'all',
						trigger: 'all',
						period: 'all',
					}}
					onFiltersChange={onFiltersChange}
				/>
			</Providers>,
		),
	}
}

describe('ExecutionsLogFeature', () => {
	it('lists every run of the organization under its workflow name', async () => {
		render()

		expect(await screen.findAllByText('Relance des devis')).toHaveLength(2)
		expect(screen.getByText('Workflow supprimé')).toBeDefined()
		expect(screen.getByText('SMTP expiré')).toBeDefined()
	})

	it('narrows the list as the search is typed', async () => {
		const user = userEvent.setup()
		render()
		await screen.findByText('SMTP expiré')

		await user.type(
			screen.getByRole('searchbox', { name: 'Rechercher une exécution' }),
			'smtp',
		)

		expect(screen.queryByText('Workflow supprimé')).toBeNull()
		expect(screen.getByText('SMTP expiré')).toBeDefined()
	})

	it('hands the status filter to the route', async () => {
		const user = userEvent.setup()
		const { onFiltersChange } = render()
		await screen.findByText('SMTP expiré')

		await user.click(screen.getByRole('radio', { name: /Échecs/ }))

		expect(onFiltersChange).toHaveBeenCalledWith({ status: 'error' })
	})
})
