import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { fireEvent, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { beforeAll, describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { AutomationWorkflowsFeature } from '#/pages/automation/feature/automation-workflows-feature'
import {
	type FakeApi,
	installFakeTanstackApi,
} from '#/pages/automation/test/fake-tanstack-api'
import { renderWithRouter } from '#/test/render-with-router'
import { seedPermissionsCacheForOrganization } from '#/test/with-permissions'

const WORKFLOWS_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows'
const WORKFLOW_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}'
const RUNS_PATH = '/api/v1/organizations/{organization_id}/automation/runs'
const FOLDERS_PATH =
	'/api/v1/organizations/{organization_id}/automation/folders'
const FOLDER_PATH =
	'/api/v1/organizations/{organization_id}/automation/folders/{folder_id}'

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

function workflow(overrides: Record<string, unknown> = {}) {
	return {
		id: 'workflow-1',
		organization_id: 'org-1',
		name: 'Créer une facture Odoo',
		description: null,
		enabled: true,
		current_version_id: null,
		folder_id: null,
		tags: [],
		archived_at: null,
		created_at: '2026-08-01T00:00:00Z',
		updated_at: '2026-08-01T00:00:00Z',
		...overrides,
	}
}

function folder(overrides: Record<string, unknown> = {}) {
	return {
		id: 'folder-ventes',
		name: 'Ventes',
		parent_id: null,
		created_at: '2026-08-01T00:00:00Z',
		...overrides,
	}
}

const VENTES = folder()
const RELANCES = folder({
	id: 'folder-relances',
	name: 'Relances',
	parent_id: 'folder-ventes',
})

function run(overrides: Record<string, unknown> = {}) {
	return {
		id: 'run-1',
		organization_id: 'org-1',
		workflow_id: 'workflow-1',
		workflow_version_id: 'v1',
		status: 'succeeded',
		created_at: '2026-08-01T00:00:00Z',
		...overrides,
	}
}

function renderFeature(configure: (api: FakeApi) => void, folderId?: string) {
	const api = installFakeTanstackApi()
	api.mockGet(WORKFLOWS_PATH, () => ({ data: [workflow()], pagination: null }))
	api.mockGet(RUNS_PATH, () => ({ data: [], pagination: null }))
	api.mockGet(FOLDERS_PATH, () => ({ data: [], pagination: null }))
	configure(api)

	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	seedPermissionsCacheForOrganization(queryClient, ORGANIZATION.id, [
		'VIEW_AUTOMATION',
		'MANAGE_AUTOMATION',
	])

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

	return renderWithRouter(
		<Providers>
			<AutomationWorkflowsFeature folderId={folderId} />
		</Providers>,
	).then(() => api)
}

describe('AutomationWorkflowsFeature — last run', () => {
	it('shows the most recent run for a workflow that ran several times', async () => {
		await renderFeature((api) => {
			api.mockGet(RUNS_PATH, () => ({
				data: [
					run({
						id: 'run-1',
						status: 'failed',
						created_at: '2026-08-01T00:00:00Z',
					}),
					run({
						id: 'run-2',
						status: 'succeeded',
						created_at: '2026-08-03T00:00:00Z',
					}),
				],
				pagination: null,
			}))
		})

		expect(await screen.findByText('Succès')).toBeDefined()
		expect(screen.queryByText('Échec')).toBeNull()
	})

	it('says plainly when a workflow has never run', async () => {
		await renderFeature((api) => {
			api.mockGet(RUNS_PATH, () => ({ data: [], pagination: null }))
		})

		expect(await screen.findByText('Jamais exécuté')).toBeDefined()
	})
})

describe('AutomationWorkflowsFeature — create', () => {
	it('opens a draft in the editor instead of creating a workflow from the list', async () => {
		const api = await renderFeature(() => {})

		const link = await screen.findByRole('link', { name: 'Nouveau workflow' })

		expect(link.getAttribute('href')).toBe(
			'/o/atelier-bois/automatisation/workflows/nouveau',
		)
		expect(
			screen.queryByRole('button', { name: 'Nouveau workflow' }),
		).toBeNull()
		expect(screen.queryByLabelText('Nom')).toBeNull()
		expect(api.calls.some((call) => call.method === 'post')).toBe(false)
	})

	it('offers the same draft link on a project page', async () => {
		await renderFeature((fakeApi) => {
			fakeApi.mockGet(FOLDERS_PATH, () => ({
				data: [VENTES],
				pagination: null,
			}))
		}, 'folder-ventes')

		expect(
			(
				await screen.findByRole('link', { name: 'Nouveau workflow' })
			).getAttribute('href'),
		).toBe('/o/atelier-bois/automatisation/workflows/nouveau')
	})
})

describe('AutomationWorkflowsFeature — rename', () => {
	it('opens the dialog pre-filled and calls the update mutation with the new name', async () => {
		const user = userEvent.setup()
		const api = await renderFeature((fakeApi) => {
			fakeApi.mockMutation('patch', WORKFLOW_PATH, (params) => ({
				data: workflow({
					...(params as { body: Record<string, unknown> }).body,
				}),
			}))
		})

		await user.click(
			await screen.findByRole('button', { name: /Actions pour/ }),
		)
		await user.click(screen.getByRole('menuitem', { name: 'Renommer' }))

		expect(screen.getByDisplayValue('Créer une facture Odoo')).toBeDefined()
		expect(screen.getByText('Modifier Créer une facture Odoo')).toBeDefined()

		const nameField = screen.getByLabelText('Nom')
		await user.clear(nameField)
		await user.type(nameField, 'Facture Odoo renommée')
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		await waitFor(() => {
			const call = api.calls.find(
				(c) => c.method === 'patch' && c.path === WORKFLOW_PATH,
			)
			expect(call?.params).toMatchObject({
				path: { organization_id: 'org-1', workflow_id: 'workflow-1' },
				body: { name: 'Facture Odoo renommée' },
			})
		})
	})
})

describe('AutomationWorkflowsFeature — enable/disable', () => {
	it('calls the update mutation with the flipped enabled flag', async () => {
		const user = userEvent.setup()
		const api = await renderFeature((fakeApi) => {
			fakeApi.mockMutation('patch', WORKFLOW_PATH, (params) => ({
				data: workflow({
					...(params as { body: Record<string, unknown> }).body,
				}),
			}))
		})

		await user.click(await screen.findByRole('switch', { name: /Désactiver/ }))

		await waitFor(() => {
			const call = api.calls.find(
				(c) => c.method === 'patch' && c.path === WORKFLOW_PATH,
			)
			expect(call?.params).toMatchObject({
				path: { organization_id: 'org-1', workflow_id: 'workflow-1' },
				body: { enabled: false },
			})
		})
	})
})

describe('AutomationWorkflowsFeature — delete', () => {
	it('calls the delete mutation once the confirmation is accepted', async () => {
		const user = userEvent.setup()
		const api = await renderFeature((fakeApi) => {
			fakeApi.mockMutation('delete', WORKFLOW_PATH, () => ({ data: undefined }))
		})

		await user.click(
			await screen.findByRole('button', { name: /Actions pour/ }),
		)
		await user.click(screen.getByRole('menuitem', { name: 'Supprimer' }))
		await user.click(screen.getByRole('button', { name: 'Supprimer' }))

		await waitFor(() => {
			const call = api.calls.find(
				(c) => c.method === 'delete' && c.path === WORKFLOW_PATH,
			)
			expect(call?.params).toMatchObject({
				path: { organization_id: 'org-1', workflow_id: 'workflow-1' },
			})
		})
	})
})

describe('AutomationWorkflowsFeature — reaching the editor', () => {
	it('builds the editor link from the active organization, not from nothing', async () => {
		await renderFeature(() => {})

		const link = await screen.findByRole('link', {
			name: /Créer une facture Odoo/,
		})

		expect(link.getAttribute('href')).toBe(
			'/o/atelier-bois/automatisation/workflows/workflow-1',
		)
	})
})

describe('AutomationWorkflowsFeature — projects', () => {
	it('titles the root page Workflows and shows no project breadcrumb', async () => {
		await renderFeature(() => {})

		expect(
			await screen.findByRole('heading', { name: 'Workflows' }),
		).toBeDefined()
		expect(
			screen.queryByRole('navigation', { name: 'Emplacement du projet' }),
		).toBeNull()
	})

	it('opens a project page with its breadcrumb and subprojects', async () => {
		await renderFeature((fakeApi) => {
			fakeApi.mockGet(FOLDERS_PATH, () => ({
				data: [VENTES, RELANCES],
				pagination: null,
			}))
		}, 'folder-ventes')

		expect(await screen.findByRole('heading', { name: 'Ventes' })).toBeDefined()
		expect(
			screen.getByRole('navigation', { name: 'Emplacement du projet' }),
		).toBeDefined()
		const content = screen.getByRole('list', { name: 'Contenu du dossier' })
		expect(content.textContent).toContain('Relances')
	})

	it('says so when the project does not exist', async () => {
		await renderFeature(() => {}, 'inconnu')

		expect(await screen.findByText('Projet introuvable')).toBeDefined()
	})

	it('does not call the project missing while the folders are still loading', async () => {
		let release: (() => void) | undefined
		const loaded = new Promise<void>((resolve) => {
			release = resolve
		})
		await renderFeature((fakeApi) => {
			fakeApi.mockGet(FOLDERS_PATH, async () => {
				await loaded
				return { data: [VENTES], pagination: null }
			})
		}, 'folder-ventes')

		await screen.findByText('Chargement des workflows…')
		expect(screen.queryByText('Projet introuvable')).toBeNull()
		expect(screen.queryByRole('heading', { name: 'Workflows' })).toBeNull()

		release?.()

		expect(await screen.findByRole('heading', { name: 'Ventes' })).toBeDefined()
		expect(screen.queryByText('Projet introuvable')).toBeNull()
	})

	it('does not call the project missing when the folders could not be loaded', async () => {
		await renderFeature((fakeApi) => {
			fakeApi.mockGet(FOLDERS_PATH, () => {
				throw new Error('Dossiers indisponibles')
			})
		}, 'folder-ventes')

		expect(await screen.findByText('Dossiers indisponibles')).toBeDefined()
		expect(screen.queryByText('Projet introuvable')).toBeNull()
	})
})

describe('AutomationWorkflowsFeature — folder and tags of a workflow', () => {
	beforeAll(() => {
		Object.assign(Element.prototype, {
			scrollIntoView: () => {},
			hasPointerCapture: () => false,
			releasePointerCapture: () => {},
		})
	})

	const relanceDesDevis = () =>
		workflow({
			name: 'Relance des devis',
			folder_id: 'folder-relances',
			tags: ['Ventes', 'Clients'],
		})

	it('shows only what the root holds directly, and the tags the API sends', async () => {
		await renderFeature((fakeApi) => {
			fakeApi.mockGet(FOLDERS_PATH, () => ({
				data: [VENTES, RELANCES],
				pagination: null,
			}))
			fakeApi.mockGet(WORKFLOWS_PATH, () => ({
				data: [relanceDesDevis(), workflow({ tags: ['Clients'] })],
				pagination: null,
			}))
		})

		expect(await screen.findByText('Créer une facture Odoo')).toBeDefined()
		expect(screen.getByRole('link', { name: 'Ventes' })).toBeDefined()
		expect(screen.getByText('Clients')).toBeDefined()
		expect(screen.queryByText('Relance des devis')).toBeNull()
		expect(screen.queryByText('Ventes › Relances')).toBeNull()
	})

	it('lists the workflow inside its project and counts it in the parent project', async () => {
		const folders = [VENTES, RELANCES]
		await renderFeature((fakeApi) => {
			fakeApi.mockGet(FOLDERS_PATH, () => ({ data: folders, pagination: null }))
			fakeApi.mockGet(WORKFLOWS_PATH, () => ({
				data: [relanceDesDevis()],
				pagination: null,
			}))
		}, 'folder-relances')

		expect(await screen.findByText('Relance des devis')).toBeDefined()
		expect(
			screen.queryByText('Ce projet ne contient encore aucun workflow.'),
		).toBeNull()
	})

	it('counts the workflow of a subproject in the subproject row of its parent', async () => {
		await renderFeature((fakeApi) => {
			fakeApi.mockGet(FOLDERS_PATH, () => ({
				data: [VENTES, RELANCES],
				pagination: null,
			}))
			fakeApi.mockGet(WORKFLOWS_PATH, () => ({
				data: [relanceDesDevis()],
				pagination: null,
			}))
		}, 'folder-ventes')

		const content = await screen.findByRole('list', {
			name: 'Contenu du dossier',
		})
		expect(content.textContent).toContain('1 workflow')
		expect(screen.queryByText('Relance des devis')).toBeNull()
	})

	it('keeps a workflow out of a project it does not belong to', async () => {
		const support = folder({ id: 'folder-support', name: 'Support' })
		await renderFeature((fakeApi) => {
			fakeApi.mockGet(FOLDERS_PATH, () => ({
				data: [VENTES, support],
				pagination: null,
			}))
			fakeApi.mockGet(WORKFLOWS_PATH, () => ({
				data: [workflow({ folder_id: 'folder-support' })],
				pagination: null,
			}))
		}, 'folder-ventes')

		expect(await screen.findByText('Ce dossier est vide.')).toBeDefined()
		expect(screen.queryByText('Créer une facture Odoo')).toBeNull()
	})

	it('leaves a workflow without folder at the root, with no project and no tag', async () => {
		await renderFeature((fakeApi) => {
			fakeApi.mockGet(FOLDERS_PATH, () => ({
				data: [VENTES],
				pagination: null,
			}))
		})

		expect(await screen.findByText('Créer une facture Odoo')).toBeDefined()
		expect(screen.queryByText('Projet :')).toBeNull()
		expect(screen.getAllByText('Ventes')).toHaveLength(1)
	})

	it('offers the tags of the listed workflows to the tag filter', async () => {
		const user = userEvent.setup()
		await renderFeature((fakeApi) => {
			fakeApi.mockGet(WORKFLOWS_PATH, () => ({
				data: [
					workflow({ id: 'workflow-1', tags: ['Support'] }),
					workflow({ id: 'workflow-2', name: 'Autre', tags: ['Ventes'] }),
				],
				pagination: null,
			}))
		})

		const trigger = await screen.findByRole('combobox', {
			name: 'Filtrer par tag',
		})
		trigger.focus()
		await user.keyboard('{Enter}')

		expect(
			screen.getAllByRole('option').map((option) => option.textContent),
		).toEqual(['Tous les tags', 'Support', 'Ventes'])

		await user.click(screen.getByRole('option', { name: 'Ventes' }))

		expect(screen.getByText('Autre')).toBeDefined()
		expect(screen.queryByText('Créer une facture Odoo')).toBeNull()
	})

	it('asks the API for the live workflows only', async () => {
		const api = await renderFeature(() => {})
		await screen.findByText('Créer une facture Odoo')

		const queries = api.calls
			.filter((call) => call.method === 'get' && call.path === WORKFLOWS_PATH)
			.map((call) => (call.params as { query?: Record<string, unknown> }).query)
		expect(queries.length).toBeGreaterThan(0)
		for (const query of queries) expect(query?.archived).toBeUndefined()
	})
})

describe('AutomationWorkflowsFeature — row actions', () => {
	function archivableApi(fakeApi: FakeApi, options: { failWith?: Error } = {}) {
		let archived = false
		fakeApi.mockGet(WORKFLOWS_PATH, () => ({
			data: archived ? [] : [workflow()],
			pagination: null,
		}))
		fakeApi.mockMutation('patch', WORKFLOW_PATH, () => {
			if (options.failWith) throw options.failWith
			archived = true
			return { data: workflow({ archived_at: '2026-11-05T00:00:00Z' }) }
		})
	}

	it('archives a workflow through the API and drops its row', async () => {
		const user = userEvent.setup()
		const api = await renderFeature((fakeApi) => archivableApi(fakeApi))

		await user.click(
			await screen.findByRole('button', { name: /Actions pour/ }),
		)
		await user.click(screen.getByRole('menuitem', { name: 'Archiver' }))

		await waitFor(() => {
			expect(screen.queryByText('Créer une facture Odoo')).toBeNull()
		})
		const patch = api.calls.find(
			(call) => call.method === 'patch' && call.path === WORKFLOW_PATH,
		)
		expect(patch?.params).toMatchObject({
			path: { organization_id: 'org-1', workflow_id: 'workflow-1' },
			body: { archived: true },
		})
		expect(api.calls.some((call) => call.method === 'delete')).toBe(false)
	})

	it('keeps the row and says why when the API refuses to archive', async () => {
		const user = userEvent.setup()
		const api = await renderFeature((fakeApi) =>
			archivableApi(fakeApi, { failWith: new Error('Archivage impossible') }),
		)

		await user.click(
			await screen.findByRole('button', { name: /Actions pour/ }),
		)
		await user.click(screen.getByRole('menuitem', { name: 'Archiver' }))

		expect(await screen.findByText('Archivage impossible')).toBeDefined()
		expect(screen.getByText('Créer une facture Odoo')).toBeDefined()
		expect(api.calls.some((call) => call.method === 'patch')).toBe(true)
	})

	it('duplicates a workflow into a disabled copy in the same project with the same tags, then saves its graph as the first version', async () => {
		const user = userEvent.setup()
		const graph = { triggers: [], connectors: [], edges: [] }
		const original = workflow({
			description: 'Émet la facture.',
			folder_id: 'folder-ventes',
			tags: ['Ventes', 'Clients'],
		})
		const api = await renderFeature((fakeApi) => {
			fakeApi.mockGet(FOLDERS_PATH, () => ({
				data: [VENTES],
				pagination: null,
			}))
			fakeApi.mockGet(WORKFLOWS_PATH, () => ({
				data: [original],
				pagination: null,
			}))
			fakeApi.mockGet(WORKFLOW_PATH, () => ({
				data: {
					...original,
					current_version: {
						id: 'v1',
						version: 1,
						workflow_id: 'workflow-1',
						created_at: '2026-08-01T00:00:00Z',
						graph,
					},
					layout: null,
				},
			}))
			fakeApi.mockMutation('post', WORKFLOWS_PATH, () => ({
				data: workflow({
					id: 'workflow-2',
					name: 'Créer une facture Odoo (copie)',
					enabled: false,
				}),
			}))
			fakeApi.mockMutation(
				'put',
				'/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}/versions',
				() => ({ data: {} }),
			)
		}, 'folder-ventes')

		await user.click(
			await screen.findByRole('button', { name: /Actions pour « / }),
		)
		await user.click(screen.getByRole('menuitem', { name: 'Dupliquer' }))

		await waitFor(() => {
			expect(
				api.calls.find((call) => call.method === 'put')?.params,
			).toMatchObject({
				path: { organization_id: 'org-1', workflow_id: 'workflow-2' },
				body: { graph },
			})
		})
		const posts = api.calls.filter((call) => call.method === 'post')
		expect(posts).toHaveLength(1)
		expect(posts[0]?.params).toEqual({
			path: { organization_id: 'org-1' },
			body: {
				name: 'Créer une facture Odoo (copie)',
				description: 'Émet la facture.',
				enabled: false,
				folder_id: 'folder-ventes',
				tags: ['Ventes', 'Clients'],
			},
		})
		expect(api.calls.some((call) => call.method === 'patch')).toBe(false)
	})

	it('copies a workflow that sits at the root with no tag as such', async () => {
		const user = userEvent.setup()
		const api = await renderFeature((fakeApi) => {
			fakeApi.mockGet(WORKFLOW_PATH, () => ({
				data: { ...workflow(), current_version: null, layout: null },
			}))
			fakeApi.mockMutation('post', WORKFLOWS_PATH, () => ({
				data: workflow({ id: 'workflow-2' }),
			}))
		})

		await user.click(
			await screen.findByRole('button', { name: /Actions pour « / }),
		)
		await user.click(screen.getByRole('menuitem', { name: 'Dupliquer' }))

		await waitFor(() => {
			expect(
				api.calls.find((call) => call.method === 'post')?.params,
			).toMatchObject({
				body: { enabled: false, folder_id: null, tags: [] },
			})
		})
	})

	it('names the copy of a copy "X (copie)", not "X (copie) (copie)"', async () => {
		const user = userEvent.setup()
		const api = await renderFeature((fakeApi) => {
			fakeApi.mockGet(WORKFLOWS_PATH, () => ({
				data: [workflow({ name: 'Créer une facture Odoo (copie)' })],
				pagination: null,
			}))
			fakeApi.mockGet(WORKFLOW_PATH, () => ({
				data: { ...workflow(), current_version: null, layout: null },
			}))
			fakeApi.mockMutation('post', WORKFLOWS_PATH, () => ({
				data: workflow({ id: 'workflow-2' }),
			}))
		})

		await user.click(
			await screen.findByRole('button', { name: /Actions pour « / }),
		)
		await user.click(screen.getByRole('menuitem', { name: 'Dupliquer' }))

		await waitFor(() => {
			expect(
				api.calls.find((call) => call.method === 'post')?.params,
			).toMatchObject({ body: { name: 'Créer une facture Odoo (copie)' } })
		})
	})

	it('exports a workflow as a JSON download built from its current version, named after the workflow', async () => {
		const user = userEvent.setup()
		const downloads: string[] = []
		const createObjectURL = vi.fn(() => 'blob:export')
		Object.assign(URL, { createObjectURL, revokeObjectURL: vi.fn() })
		const click = vi
			.spyOn(HTMLAnchorElement.prototype, 'click')
			.mockImplementation(function (this: HTMLAnchorElement) {
				downloads.push(this.download)
			})
		await renderFeature((fakeApi) => {
			fakeApi.mockGet(WORKFLOW_PATH, () => ({
				data: {
					...workflow(),
					current_version: null,
					layout: null,
				},
			}))
		})

		await user.click(
			await screen.findByRole('button', { name: /Actions pour/ }),
		)
		await user.click(screen.getByRole('menuitem', { name: 'Exporter' }))

		await waitFor(() => expect(click).toHaveBeenCalled())
		expect(createObjectURL).toHaveBeenCalled()
		expect(downloads).toEqual(['creer-une-facture-odoo.json'])
	})
})

function bodyOf(api: FakeApi, method: string) {
	const call = api.calls.find((entry) => entry.method === method)
	return (call?.params as { body?: Record<string, unknown> } | undefined)?.body
}

describe('AutomationWorkflowsFeature — folders', () => {
	const withFolders = (api: FakeApi) =>
		api.mockGet(FOLDERS_PATH, () => ({
			data: [VENTES, RELANCES],
			pagination: null,
		}))

	it('creates a folder at the root from the workflow list', async () => {
		const user = userEvent.setup()
		const api = await renderFeature((fakeApi) => {
			fakeApi.mockMutation('post', FOLDERS_PATH, () => ({ data: VENTES }))
		})

		await user.click(
			await screen.findByRole('button', { name: 'Nouveau dossier' }),
		)
		await user.type(screen.getByLabelText('Nom du dossier'), '  Chantiers ')
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		await waitFor(() =>
			expect(bodyOf(api, 'post')).toEqual({
				name: 'Chantiers',
				parent_id: null,
			}),
		)
	})

	it('creates a folder inside the current project', async () => {
		const user = userEvent.setup()
		const api = await renderFeature((fakeApi) => {
			withFolders(fakeApi)
			fakeApi.mockMutation('post', FOLDERS_PATH, () => ({ data: RELANCES }))
		}, 'folder-ventes')

		await user.click(
			await screen.findByRole('button', { name: 'Nouveau dossier' }),
		)
		await user.type(screen.getByLabelText('Nom du dossier'), 'Devis')
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		await waitFor(() =>
			expect(bodyOf(api, 'post')).toEqual({
				name: 'Devis',
				parent_id: 'folder-ventes',
			}),
		)
	})

	it('lists the root folders on the workflow list', async () => {
		await renderFeature(withFolders)

		expect(await screen.findByRole('link', { name: 'Ventes' })).toBeDefined()
		expect(screen.queryByRole('link', { name: 'Relances' })).toBeNull()
	})

	it('renames a folder', async () => {
		const user = userEvent.setup()
		const api = await renderFeature((fakeApi) => {
			withFolders(fakeApi)
			fakeApi.mockMutation('patch', FOLDER_PATH, () => ({ data: VENTES }))
		})

		await user.click(
			await screen.findByRole('button', {
				name: 'Dossier « Ventes » : actions',
			}),
		)
		await user.click(screen.getByRole('menuitem', { name: 'Renommer' }))
		const input = screen.getByLabelText('Nom du dossier')
		await user.clear(input)
		await user.type(input, 'Commercial')
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		await waitFor(() =>
			expect(bodyOf(api, 'patch')).toEqual({ name: 'Commercial' }),
		)
	})

	it('moves a folder to the root and never offers its own subtree', async () => {
		const user = userEvent.setup()
		const api = await renderFeature((fakeApi) => {
			withFolders(fakeApi)
			fakeApi.mockMutation('patch', FOLDER_PATH, () => ({ data: RELANCES }))
		}, 'folder-ventes')

		await user.click(
			await screen.findByRole('button', {
				name: 'Dossier « Relances » : actions',
			}),
		)
		await user.click(screen.getByRole('menuitem', { name: 'Déplacer' }))

		const options = screen
			.getAllByRole('option')
			.map((option) => option.textContent)
		expect(options).toEqual(['Aucun dossier (racine)', 'Ventes'])

		await user.selectOptions(screen.getByLabelText('Destination'), '')
		await user.click(screen.getByRole('button', { name: 'Déplacer' }))

		await waitFor(() =>
			expect(bodyOf(api, 'patch')).toEqual({ parent_id: null }),
		)
	})

	it('moves a workflow into a folder', async () => {
		const user = userEvent.setup()
		const api = await renderFeature((fakeApi) => {
			withFolders(fakeApi)
			fakeApi.mockMutation('patch', WORKFLOW_PATH, () => ({ data: workflow() }))
		})

		await user.click(
			await screen.findByRole('button', { name: /Actions pour « / }),
		)
		await user.click(screen.getByRole('menuitem', { name: 'Déplacer' }))
		await user.selectOptions(
			screen.getByLabelText('Destination'),
			'folder-relances',
		)
		await user.click(screen.getByRole('button', { name: 'Déplacer' }))

		await waitFor(() =>
			expect(bodyOf(api, 'patch')).toEqual({ folder_id: 'folder-relances' }),
		)
	})

	it('says why a non-empty folder was not deleted', async () => {
		const user = userEvent.setup()
		await renderFeature((fakeApi) => {
			withFolders(fakeApi)
			fakeApi.mockMutation('delete', FOLDER_PATH, () => {
				throw new Error('Le dossier contient encore des éléments')
			})
		})

		await user.click(
			await screen.findByRole('button', {
				name: 'Dossier « Ventes » : actions',
			}),
		)
		await user.click(screen.getByRole('menuitem', { name: 'Supprimer' }))
		await user.click(screen.getByRole('button', { name: 'Supprimer' }))

		expect(
			await screen.findByText('Le dossier contient encore des éléments'),
		).toBeDefined()
	})

	const transfer = () => ({
		setData: vi.fn(),
		effectAllowed: '',
		dropEffect: '',
	})
	const rowOf = (name: string) =>
		screen.getByText(name).closest('li') as HTMLElement

	it('moves a workflow dropped on a folder', async () => {
		const api = await renderFeature((fakeApi) => {
			withFolders(fakeApi)
			fakeApi.mockMutation('patch', WORKFLOW_PATH, () => ({ data: workflow() }))
		})
		await screen.findByRole('link', { name: 'Ventes' })

		fireEvent.dragStart(rowOf('Créer une facture Odoo'), {
			dataTransfer: transfer(),
		})
		fireEvent.drop(rowOf('Ventes'), { dataTransfer: transfer() })

		await waitFor(() =>
			expect(bodyOf(api, 'patch')).toEqual({ folder_id: 'folder-ventes' }),
		)
	})

	it('moves a folder dropped on the Workflows crumb to the root', async () => {
		const api = await renderFeature((fakeApi) => {
			withFolders(fakeApi)
			fakeApi.mockMutation('patch', FOLDER_PATH, () => ({ data: RELANCES }))
		}, 'folder-ventes')
		await screen.findByRole('link', { name: 'Relances' })

		fireEvent.dragStart(rowOf('Relances'), { dataTransfer: transfer() })
		fireEvent.drop(
			screen
				.getByRole('link', { name: 'Workflows' })
				.closest('li') as HTMLElement,
			{ dataTransfer: transfer() },
		)

		await waitFor(() =>
			expect(bodyOf(api, 'patch')).toEqual({ parent_id: null }),
		)
	})

	it('sends nothing when a folder is dropped on itself', async () => {
		const api = await renderFeature(withFolders)
		await screen.findByRole('link', { name: 'Ventes' })

		fireEvent.dragStart(rowOf('Ventes'), { dataTransfer: transfer() })
		fireEvent.drop(rowOf('Ventes'), { dataTransfer: transfer() })

		expect(api.calls.some((call) => call.method === 'patch')).toBe(false)
	})

	it('shows a workflow of a subfolder only once that folder is open', async () => {
		const configure = (fakeApi: FakeApi) => {
			withFolders(fakeApi)
			fakeApi.mockGet(WORKFLOWS_PATH, () => ({
				data: [
					workflow({ name: 'Relance des devis', folder_id: 'folder-relances' }),
				],
				pagination: null,
			}))
		}

		await renderFeature(configure, 'folder-relances')

		expect(await screen.findByText('Relance des devis')).toBeDefined()
	})

	describe('search', () => {
		const relance = () =>
			workflow({
				id: 'workflow-2',
				name: 'Relance des devis',
				folder_id: 'folder-relances',
			})
		const configure = (fakeApi: FakeApi) => {
			withFolders(fakeApi)
			fakeApi.mockGet(WORKFLOWS_PATH, () => ({
				data: [workflow(), relance()],
				pagination: null,
			}))
		}

		it('finds a workflow of any folder from the root, with its folder path', async () => {
			const user = userEvent.setup()
			await renderFeature(configure)
			await screen.findByText('Créer une facture Odoo')
			expect(screen.queryByText('Relance des devis')).toBeNull()

			await user.type(screen.getByRole('searchbox'), 'relance')

			expect(await screen.findByText('Relance des devis')).toBeDefined()
			expect(screen.getByText('Ventes › Relances')).toBeDefined()
			expect(screen.queryByText('Créer une facture Odoo')).toBeNull()
			expect(screen.queryByRole('link', { name: 'Ventes' })).toBeNull()
		})

		it('searches outside the open folder too, and restores it when cleared', async () => {
			const user = userEvent.setup()
			await renderFeature(configure, 'folder-relances')
			await screen.findByText('Relance des devis')
			expect(screen.queryByText('Créer une facture Odoo')).toBeNull()

			const box = screen.getByRole('searchbox')
			await user.type(box, 'facture')
			expect(await screen.findByText('Créer une facture Odoo')).toBeDefined()
			expect(screen.queryByText('Relance des devis')).toBeNull()

			await user.clear(box)
			expect(await screen.findByText('Relance des devis')).toBeDefined()
			expect(screen.queryByText('Créer une facture Odoo')).toBeNull()
		})
	})
})
