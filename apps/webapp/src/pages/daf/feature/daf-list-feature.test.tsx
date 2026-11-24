import { QueryClient } from '@tanstack/react-query'
import { screen, within } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { DafListFeature } from '#/pages/daf/feature/daf-list-feature'
import { renderWithRouter } from '#/test/render-with-router'
import { wrapWithPermissions } from '#/test/with-permissions'

const LISTING_PATH =
	'/api/v1/organizations/{organization_id}/projects/{project_id}/dafs'
const PERMISSIONS_PATH =
	'/api/v1/organizations/{organization_id}/members/me/permissions'

const SUMMARY: Schemas.DafSummaryResponse = {
	id: 'daf-1',
	number: 1,
	designation: 'Ganivelle',
	price_ref: '2.4.1',
	status: 'accepted',
	issued_on: '2026-11-18',
	sent_on: '2026-11-19',
	returned_on: '2026-11-25',
	review_note: 'Visa sans réserve',
	updated_at: '2026-11-25T08:00:00Z',
}

function listing(dafs: Schemas.DafSummaryResponse[], dossierStored = true) {
	return {
		data: {
			dossier: {
				project_id: 'project-1',
				title: 'DAF Parc Montgroux',
				issuer: 'Treyve Paysages',
				project_owner: 'Commune de Cébazat',
				project_manager: null,
				updated_at: dossierStored ? '2026-11-18T08:00:00Z' : null,
			},
			dafs,
		},
		pagination: null,
	}
}

function install(payload: unknown) {
	// biome-ignore lint/suspicious/noExplicitAny: test-only fake, shape matches TanstackQueryApiClient's used surface
	;(window as any).tanstackApi = {
		get(path: string) {
			const queryKey = [{ _id: path }]
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						if (path === PERMISSIONS_PATH) {
							return {
								data: { permissions: ['MANAGE_PLANNING'] },
								pagination: null,
							}
						}
						if (path === LISTING_PATH) return payload
						return { data: [], pagination: null }
					},
				},
			}
		},
		mutation(method: string, path: string) {
			return {
				mutationOptions: {
					mutationKey: [{ method, path }],
					mutationFn: async () => {
						throw new Error('unmocked')
					},
				},
			}
		},
	}
}

async function renderList(payload: unknown) {
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	const ui = wrapWithPermissions(
		<DafListFeature
			organizationId="org-1"
			organizationSlug="organisation"
			projectId="project-1"
		/>,
		{ queryClient },
	)
	install(payload)
	await renderWithRouter(ui)
}

describe('@spec-daf-10 DafListFeature', () => {
	it('shows the summary row with the accepted badge', async () => {
		await renderList(listing([SUMMARY]))

		const cell = await screen.findByText('D.A.F n°01')
		const row = cell.closest('tr')
		expect(row).not.toBeNull()
		const scope = within(row as HTMLElement)
		expect(scope.getByText('Ganivelle')).toBeDefined()
		expect(scope.getByText('Acceptée')).toBeDefined()
		expect(scope.getByText('2.4.1')).toBeDefined()
		expect(scope.getByText('19/11/2026')).toBeDefined()
		expect(scope.getByText('Visa sans réserve')).toBeDefined()
		expect(screen.getByRole('link', { name: /PDF du dossier/ })).toBeDefined()
		expect(screen.getByRole('button', { name: /Nouvelle DAF/ })).toBeDefined()
	})

	it('shows the empty state and the unsaved dossier hint', async () => {
		await renderList(listing([], false))

		expect(
			await screen.findByText('Ce projet n’a pas encore de DAF'),
		).toBeDefined()
		expect(screen.getByText('Pré-rempli, pas encore enregistré.')).toBeDefined()
		expect(screen.getByDisplayValue('Treyve Paysages')).toBeDefined()
	})
})
