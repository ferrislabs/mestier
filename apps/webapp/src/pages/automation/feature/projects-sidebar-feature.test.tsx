import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, screen, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { describe, expect, it } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { ProjectsSidebarFeature } from '#/pages/automation/feature/projects-sidebar-feature'
import { installFakeTanstackApi } from '#/pages/automation/test/fake-tanstack-api'
import { renderWithRouter } from '#/test/render-with-router'

const WORKFLOWS_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows'
const FOLDERS_PATH =
	'/api/v1/organizations/{organization_id}/automation/folders'

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

function folder(id: string, name: string, parentId: string | null = null) {
	return {
		id,
		name,
		parent_id: parentId,
		created_at: '2026-08-01T00:00:00Z',
	}
}

function workflow(id: string, folderId: string | null) {
	return {
		id,
		organization_id: 'org-1',
		name: `Workflow ${id}`,
		description: null,
		enabled: true,
		current_version_id: null,
		folder_id: folderId,
		tags: [],
		archived_at: null,
		created_at: '2026-08-01T00:00:00Z',
		updated_at: '2026-08-01T00:00:00Z',
	}
}

function render(folders: unknown[], workflows: unknown[]) {
	const api = installFakeTanstackApi()
	api.mockGet(FOLDERS_PATH, () => ({ data: folders, pagination: null }))
	api.mockGet(WORKFLOWS_PATH, () => ({ data: workflows, pagination: null }))

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

	return renderWithRouter(
		<Providers>
			<ProjectsSidebarFeature />
		</Providers>,
	).then(() => api)
}

describe('ProjectsSidebarFeature', () => {
	it('lists the root folders of the API, each with the workflows of its whole subtree', async () => {
		await render(
			[
				folder('ventes', 'Ventes'),
				folder('relances', 'Relances', 'ventes'),
				folder('support', 'Support client'),
			],
			[workflow('a', 'ventes'), workflow('b', 'relances'), workflow('c', null)],
		)

		const ventes = await screen.findByRole('link', { name: /Ventes/ })
		expect(ventes.getAttribute('href')).toBe(
			'/o/atelier-bois/automatisation/projets/ventes',
		)
		expect(ventes.textContent).toContain('2')
		expect(
			screen.getByRole('link', { name: /Support client/ }).textContent,
		).toContain('0')
		expect(screen.queryByRole('link', { name: /Relances/ })).toBeNull()
	})

	it('draws the count from the live workflows the API returns, never from a stored archive', async () => {
		const api = await render([folder('ventes', 'Ventes')], [])

		const ventes = await screen.findByRole('link', { name: /Ventes/ })

		expect(ventes.textContent).toContain('0')
		const queries = api.calls
			.filter((call) => call.path === WORKFLOWS_PATH)
			.map((call) => (call.params as { query?: Record<string, unknown> }).query)
		for (const query of queries) expect(query?.archived).toBeUndefined()
	})

	it('shows no Projets section for an organization without folder', async () => {
		const api = await render([], [workflow('a', null)])

		await waitFor(() => {
			expect(api.calls.map((call) => call.path).sort()).toEqual([
				FOLDERS_PATH,
				WORKFLOWS_PATH,
			])
		})
		await act(async () => {})

		expect(screen.queryByRole('heading', { name: 'Projets' })).toBeNull()
	})
})
