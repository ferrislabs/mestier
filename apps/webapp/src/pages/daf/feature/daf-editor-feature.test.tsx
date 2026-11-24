import { QueryClient } from '@tanstack/react-query'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { DafEditorFeature } from '#/pages/daf/feature/daf-editor-feature'
import { renderWithRouter } from '#/test/render-with-router'
import { wrapWithPermissions } from '#/test/with-permissions'

const DAF_PATH = '/api/v1/organizations/{organization_id}/dafs/{daf_id}'
const PERMISSIONS_PATH =
	'/api/v1/organizations/{organization_id}/members/me/permissions'

const DAF: Schemas.DafResponse = {
	id: 'daf-1',
	project_id: 'project-1',
	number: 1,
	designation: 'Ganivelle',
	attachments: [],
	status: 'pending',
	issued_on: '2026-11-18',
	created_at: '2026-11-18T08:00:00Z',
	updated_at: '2026-11-18T09:00:00Z',
}

const DOSSIER: Schemas.DafDossierResponse = {
	project_id: 'project-1',
	title: 'DAF',
	issuer: 'Treyve',
	project_owner: 'Commune de Cébazat',
	project_manager: 'Atelier Casa',
	updated_at: '2026-11-18T08:00:00Z',
}

function install(onPut: (params: { body: unknown }) => unknown) {
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
						return { data: [], pagination: null }
					},
				},
			}
		},
		mutation(method: string, path: string) {
			return {
				mutationOptions: {
					mutationKey: [{ method, path }],
					mutationFn: async (params: { body: unknown }) => {
						if (method === 'put' && path === DAF_PATH) return onPut(params)
						throw new Error(`unmocked ${method} ${path}`)
					},
				},
			}
		},
	}
}

Element.prototype.scrollIntoView ??= () => {}
Element.prototype.hasPointerCapture ??= () => false
Element.prototype.setPointerCapture ??= () => {}
Element.prototype.releasePointerCapture ??= () => {}

describe('@spec-daf-10 DafEditorFeature', () => {
	it('saves the status change with null for empty fields and the concurrency token', async () => {
		const user = userEvent.setup()
		let sent: unknown = null
		const queryClient = new QueryClient({
			defaultOptions: {
				queries: { retry: false },
				mutations: { retry: false },
			},
		})
		const ui = wrapWithPermissions(
			<DafEditorFeature
				daf={DAF}
				dossier={DOSSIER}
				organizationId="org-1"
				organizationSlug="organisation"
				projectId="project-1"
				projectName="Parc Montgroux"
			/>,
			{ queryClient },
		)
		install(({ body }) => {
			sent = body
			return { data: { ...DAF, status: 'accepted', updated_at: 'later' } }
		})
		await renderWithRouter(ui)

		await user.click(screen.getByRole('combobox', { name: 'Statut' }))
		await user.click(await screen.findByRole('option', { name: 'Acceptée' }))
		await user.type(screen.getByLabelText('Maîtrise d’œuvre'), ' Casa ')
		await user.click(screen.getByRole('button', { name: /Enregistrer/ }))

		await waitFor(() => expect(sent).not.toBeNull())
		expect(sent).toMatchObject({
			number: 1,
			designation: 'Ganivelle',
			status: 'accepted',
			supplier: null,
			price_ref: null,
			project_owner: null,
			project_manager: 'Casa',
			expected_updated_at: '2026-11-18T09:00:00Z',
		})
		expect(await screen.findByText('Enregistré')).toBeDefined()
	})

	it('shows the dossier value as placeholder for the overrides', async () => {
		const queryClient = new QueryClient()
		const ui = wrapWithPermissions(
			<DafEditorFeature
				daf={DAF}
				dossier={DOSSIER}
				organizationId="org-1"
				organizationSlug="organisation"
				projectId="project-1"
				projectName="Parc Montgroux"
			/>,
			{ queryClient },
		)
		install(() => ({ data: DAF }))
		await renderWithRouter(ui)

		expect(screen.getByPlaceholderText('Atelier Casa')).toBeDefined()
		expect(screen.getByPlaceholderText('Commune de Cébazat')).toBeDefined()
	})
})
