import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { CollectionDetailFeature } from '#/pages/automation/feature/collection-detail-feature'
import { CollectionsListFeature } from '#/pages/automation/feature/collections-list-feature'
import { installFakeTanstackApi } from '#/pages/automation/test/fake-tanstack-api'
import { renderWithRouter } from '#/test/render-with-router'
import { seedPermissionsCacheForOrganization } from '#/test/with-permissions'

const COLLECTIONS_PATH =
	'/api/v1/organizations/{organization_id}/automation/collections'
const COLLECTION_PATH =
	'/api/v1/organizations/{organization_id}/automation/collections/{collection_id}'

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

const ADDRESS_ID = '6f1f6a9e-5f0c-4a43-9d77-1d9d0a3b1c11'
const COMPANY_ID = '0b7a3c1e-2d54-4c85-8b0d-6a4f1d2e9a77'
const IMPORTED_ID = '9c2d8e41-7a36-4f0b-b1a5-3e6d7c8f9a02'

const READ_AND_WRITE = ['VIEW_AUTOMATION', 'MANAGE_AUTOMATION']

function storedCollection(
	overrides: Partial<Schemas.CollectionResponse> = {},
): Schemas.CollectionResponse {
	return {
		id: ADDRESS_ID,
		organization_id: ORGANIZATION.id,
		slug: 'api-adresse',
		title: 'API Adresse',
		version: '1.0',
		description: 'Géocodage des adresses françaises.',
		base_url: 'https://api-adresse.data.gouv.fr',
		source: { kind: 'url', url: 'https://adresse.test/openapi.json' },
		imported_at: '2026-11-06T09:30:00Z',
		security: [
			{ name: 'bearerAuth', kind: { type: 'http', scheme: 'bearer' } },
		],
		operations: [
			{
				id: 'searchAddress',
				method: 'GET',
				path: '/search',
				summary: 'Chercher une adresse',
				tag: 'Recherche',
				parameters: [
					{
						name: 'q',
						location: 'query',
						required: true,
						description: 'Texte recherché',
						schema: { type: 'string' },
					},
				],
				request_body: null,
				response: null,
				response_status: '200',
			},
			{
				id: 'reverseAddress',
				method: 'GET',
				path: '/reverse',
				summary: 'Trouver une adresse depuis des coordonnées',
				tag: 'Recherche',
				parameters: [],
				request_body: null,
				response: null,
				response_status: '200',
			},
		],
		schemas: {},
		history: [],
		...overrides,
	}
}

function summaryOf(
	collection: Schemas.CollectionResponse,
): Schemas.CollectionSummaryResponse {
	return {
		id: collection.id,
		organization_id: collection.organization_id,
		slug: collection.slug,
		title: collection.title,
		version: collection.version,
		base_url: collection.base_url,
		source: collection.source,
		imported_at: collection.imported_at,
		security: collection.security,
		operation_count: collection.operations.length,
	}
}

const COMPANY = storedCollection({
	id: COMPANY_ID,
	slug: 'recherche-entreprises',
	title: 'Recherche d’entreprises',
	version: '2.3.1',
	base_url: 'https://recherche-entreprises.api.gouv.fr',
	source: { kind: 'file', file_name: 'entreprises.yaml' },
	security: [],
	operations: [storedCollection().operations[0] as Schemas.OperationDto],
})

function specDocument(
	overrides: { version?: string; servers?: unknown; paths?: unknown } = {},
) {
	return {
		openapi: '3.0.3',
		info: { title: 'API Adresse', version: overrides.version ?? '1.1' },
		servers: overrides.servers ?? [{ url: 'https://api-adresse.data.gouv.fr' }],
		components: {
			securitySchemes: { bearerAuth: { type: 'http', scheme: 'bearer' } },
		},
		paths: overrides.paths ?? {
			'/search': {
				get: {
					operationId: 'searchAddress',
					summary: 'Chercher une adresse',
					tags: ['Recherche'],
					parameters: [
						{
							name: 'q',
							in: 'query',
							required: true,
							schema: { type: 'string' },
						},
					],
					responses: { '200': { description: 'ok' } },
				},
			},
			'/reverse': {
				get: {
					operationId: 'reverseAddress',
					summary: 'Trouver une adresse depuis des coordonnées',
					tags: ['Recherche'],
					responses: { '200': { description: 'ok' } },
				},
			},
		},
	}
}

function specFile(document: unknown, name = 'adresse.json') {
	return new File([JSON.stringify(document)], name, {
		type: 'application/json',
	})
}

function apiError(status: number, message: string) {
	return Object.assign(new Error(message), { status })
}

function renderFeature(
	ui: ReactNode,
	{
		permissions = READ_AND_WRITE,
		configure = () => {},
	}: {
		permissions?: string[]
		configure?: (api: ReturnType<typeof installFakeTanstackApi>) => void
	} = {},
) {
	const api = installFakeTanstackApi()
	api.mockGet(COLLECTIONS_PATH, () => ({
		data: [summaryOf(storedCollection()), summaryOf(COMPANY)],
		pagination: null,
	}))
	api.mockGet(COLLECTION_PATH, (params) => {
		const id = (params as { path: { collection_id: string } }).path
			.collection_id
		if (id === COMPANY_ID) return { data: COMPANY, pagination: null }
		if (id === ADDRESS_ID) return { data: storedCollection(), pagination: null }
		throw apiError(404, 'Resource not found')
	})
	configure(api)

	const queryClient = new QueryClient({
		defaultOptions: {
			queries: { retry: false, retryDelay: 0 },
			mutations: { retry: false },
		},
	})
	seedPermissionsCacheForOrganization(queryClient, ORGANIZATION.id, permissions)

	return renderWithRouter(
		<QueryClientProvider client={queryClient}>
			<OrganizationListProvider organizations={[ORGANIZATION]}>
				<ActiveOrganizationProvider activeOrganization={ORGANIZATION}>
					{ui}
				</ActiveOrganizationProvider>
			</OrganizationListProvider>
		</QueryClientProvider>,
	).then((rendered) => ({ ...rendered, api }))
}

afterEach(() => {
	vi.unstubAllGlobals()
})

describe('CollectionsListFeature', () => {
	it('lists the collections stored by the API, each linking to its uuid', async () => {
		await renderFeature(<CollectionsListFeature />)

		const link = await screen.findByRole('link', { name: 'API Adresse' })
		expect(link.getAttribute('href')).toBe(
			`/o/atelier-bois/automatisation/collections/${ADDRESS_ID}`,
		)
		expect(
			screen
				.getByRole('link', { name: 'Recherche d’entreprises' })
				.getAttribute('href'),
		).toBe(`/o/atelier-bois/automatisation/collections/${COMPANY_ID}`)
		expect(screen.getByText('v2.3.1')).toBeDefined()
		expect(screen.getByText('2 opérations')).toBeDefined()
		expect(screen.getByText('1 opération')).toBeDefined()
		expect(
			screen.getByText(
				(text) =>
					text.startsWith('Importée depuis le fichier entreprises.yaml') &&
					text.includes('6 novembre 2026'),
			),
		).toBeDefined()
	})

	it('invites to import when the organization has no collection, with no sample injected', async () => {
		await renderFeature(<CollectionsListFeature />, {
			configure: (api) =>
				api.mockGet(COLLECTIONS_PATH, () => ({ data: [], pagination: null })),
		})

		expect(
			await screen.findByText('Aucune collection pour le moment.'),
		).toBeDefined()
		expect(
			screen.getByText(
				'Importez la spec OpenAPI d’un service externe pour l’appeler depuis vos workflows sans écrire les requêtes à la main.',
			),
		).toBeDefined()
		expect(screen.queryByRole('link', { name: 'API Adresse' })).toBeNull()
		expect(
			screen.queryByRole('link', { name: /Recherche d’entreprises/ }),
		).toBeNull()
		expect(localStorage.getItem('mestier:automation-collections:v1')).toBeNull()
	})

	it('says so while the collections load', async () => {
		await renderFeature(<CollectionsListFeature />, {
			configure: (api) =>
				api.mockGet(COLLECTIONS_PATH, () => new Promise(() => {})),
		})

		expect(await screen.findByText('Chargement des collections…')).toBeDefined()
		expect(screen.queryByText('Aucune collection pour le moment.')).toBeNull()
	})

	it('says when the collections cannot be loaded', async () => {
		await renderFeature(<CollectionsListFeature />, {
			configure: (api) =>
				api.mockGet(COLLECTIONS_PATH, () => {
					throw apiError(500, 'Internal server error')
				}),
		})

		expect(
			await screen.findByText(
				'Une erreur est survenue. Réessayez dans un instant.',
			),
		).toBeDefined()
		expect(screen.queryByText('Aucune collection pour le moment.')).toBeNull()
	})

	it('opens the import dialog', async () => {
		const user = userEvent.setup()
		await renderFeature(<CollectionsListFeature />)

		await user.click(
			await screen.findByRole('button', { name: 'Importer une collection' }),
		)

		expect(
			await screen.findByRole('dialog', { name: 'Importer une collection' }),
		).toBeDefined()
		expect(screen.getByLabelText('URL de la spec')).toBeDefined()
	})

	it('hides every write control from a caller without MANAGE_AUTOMATION', async () => {
		await renderFeature(<CollectionsListFeature />, {
			permissions: ['VIEW_AUTOMATION'],
		})

		await screen.findByRole('link', { name: 'API Adresse' })
		expect(
			screen.queryByRole('button', { name: 'Importer une collection' }),
		).toBeNull()
		expect(screen.queryByRole('button', { name: /Actions sur/ })).toBeNull()
	})

	it('deletes a collection through the API and shows the list again', async () => {
		const user = userEvent.setup()
		let remaining = [storedCollection(), COMPANY]
		const { api } = await renderFeature(<CollectionsListFeature />, {
			configure: (fake) => {
				fake.mockGet(COLLECTIONS_PATH, () => ({
					data: remaining.map(summaryOf),
					pagination: null,
				}))
				fake.mockMutation('delete', COLLECTION_PATH, (params) => {
					const id = (params as { path: { collection_id: string } }).path
						.collection_id
					remaining = remaining.filter((item) => item.id !== id)
				})
			},
		})

		await user.click(
			await screen.findByRole('button', { name: 'Actions sur API Adresse' }),
		)
		await user.click(
			await screen.findByRole('menuitem', { name: 'Supprimer la collection' }),
		)

		await waitFor(() =>
			expect(screen.queryByRole('link', { name: 'API Adresse' })).toBeNull(),
		)
		expect(api.calls.find((call) => call.method === 'delete')?.params).toEqual({
			path: { organization_id: 'org-1', collection_id: ADDRESS_ID },
		})
		expect(
			screen.getByRole('link', { name: 'Recherche d’entreprises' }),
		).toBeDefined()
	})

	it('says why a deletion failed', async () => {
		const user = userEvent.setup()
		await renderFeature(<CollectionsListFeature />, {
			configure: (api) =>
				api.mockMutation('delete', COLLECTION_PATH, () => {
					throw apiError(403, 'Forbidden')
				}),
		})

		await user.click(
			await screen.findByRole('button', { name: 'Actions sur API Adresse' }),
		)
		await user.click(
			await screen.findByRole('menuitem', { name: 'Supprimer la collection' }),
		)

		expect(
			await screen.findByText(
				'Vous n’avez pas la permission de gérer les collections.',
			),
		).toBeDefined()
		expect(screen.getByRole('link', { name: 'API Adresse' })).toBeDefined()
	})
})

describe('CollectionsListFeature — import', () => {
	async function openFileImport(
		configure?: (api: ReturnType<typeof installFakeTanstackApi>) => void,
	) {
		const user = userEvent.setup()
		const rendered = await renderFeature(<CollectionsListFeature />, {
			configure,
		})
		await user.click(
			await screen.findByRole('button', { name: 'Importer une collection' }),
		)
		const dialog = await screen.findByRole('dialog', {
			name: 'Importer une collection',
		})
		await user.click(within(dialog).getByRole('tab', { name: 'Fichier' }))
		const input = within(dialog).getByLabelText(/Déposez un fichier/)
		return { ...rendered, user, dialog, input }
	}

	it('previews a spec chosen as a file, then stores it and opens it', async () => {
		const { user, dialog, input, api, router } = await openFileImport((fake) =>
			fake.mockMutation('post', COLLECTIONS_PATH, () => ({
				data: storedCollection({
					id: IMPORTED_ID,
					source: { kind: 'file', file_name: 'adresse.json' },
				}),
			})),
		)

		await user.upload(input, specFile(specDocument()))

		expect(await within(dialog).findByText('API Adresse')).toBeDefined()
		expect(within(dialog).getByText('1.1')).toBeDefined()
		expect(
			within(dialog).getByText('https://api-adresse.data.gouv.fr'),
		).toBeDefined()
		await user.click(
			within(dialog).getByRole('button', { name: 'Ajouter la collection' }),
		)

		await waitFor(() =>
			expect(router.state.location.pathname).toBe(
				`/o/atelier-bois/automatisation/collections/${IMPORTED_ID}`,
			),
		)
		const post = api.calls.find((call) => call.method === 'post')
		expect(post?.params).toMatchObject({
			path: { organization_id: 'org-1' },
			body: {
				title: 'API Adresse',
				version: '1.1',
				base_url: 'https://api-adresse.data.gouv.fr',
				source: { kind: 'file', file_name: 'adresse.json' },
				security: [
					{ name: 'bearerAuth', kind: { type: 'http', scheme: 'bearer' } },
				],
			},
		})
		const body = (post?.params as { body: Schemas.CollectionRequest }).body
		expect(body.operations.map((operation) => operation.id)).toEqual([
			'searchAddress',
			'reverseAddress',
		])
		expect(body).not.toHaveProperty('id')
		expect(body).not.toHaveProperty('imported_at')
	})

	it('fetches a spec from its url', async () => {
		const user = userEvent.setup()
		const fetchSpec = vi.fn(async () =>
			Response.json(specDocument({ version: '3.0' })),
		)
		vi.stubGlobal('fetch', fetchSpec)
		await renderFeature(<CollectionsListFeature />)
		await user.click(
			await screen.findByRole('button', { name: 'Importer une collection' }),
		)
		const dialog = await screen.findByRole('dialog', {
			name: 'Importer une collection',
		})

		await user.type(
			within(dialog).getByLabelText('URL de la spec'),
			'https://adresse.test/openapi.json',
		)
		await user.click(within(dialog).getByRole('button', { name: 'Charger' }))

		expect(await within(dialog).findByText('3.0')).toBeDefined()
		expect(fetchSpec).toHaveBeenCalledWith('https://adresse.test/openapi.json')
	})

	it('explains a blocked fetch as a CORS problem', async () => {
		const user = userEvent.setup()
		vi.stubGlobal(
			'fetch',
			vi.fn(async () => {
				throw new TypeError('Failed to fetch')
			}),
		)
		await renderFeature(<CollectionsListFeature />)
		await user.click(
			await screen.findByRole('button', { name: 'Importer une collection' }),
		)
		const dialog = await screen.findByRole('dialog', {
			name: 'Importer une collection',
		})

		await user.type(
			within(dialog).getByLabelText('URL de la spec'),
			'https://adresse.test/openapi.json',
		)
		await user.click(within(dialog).getByRole('button', { name: 'Charger' }))

		expect(
			await within(dialog).findByText(
				/refuse l’accès depuis le navigateur \(CORS\)/,
			),
		).toBeDefined()
	})

	it('tells which status the server answered when the spec cannot be downloaded', async () => {
		const user = userEvent.setup()
		vi.stubGlobal(
			'fetch',
			vi.fn(async () => new Response('nope', { status: 404 })),
		)
		await renderFeature(<CollectionsListFeature />)
		await user.click(
			await screen.findByRole('button', { name: 'Importer une collection' }),
		)
		const dialog = await screen.findByRole('dialog', {
			name: 'Importer une collection',
		})

		await user.type(
			within(dialog).getByLabelText('URL de la spec'),
			'https://adresse.test/absent.json',
		)
		await user.click(within(dialog).getByRole('button', { name: 'Charger' }))

		expect(
			await within(dialog).findByText(
				'Le serveur a répondu 404. Vérifiez l’URL.',
			),
		).toBeDefined()
	})

	it('refuses a spec whose server url keeps a variable, before calling the API', async () => {
		const { user, dialog, input, api } = await openFileImport()

		await user.upload(
			input,
			specFile(
				specDocument({
					servers: [{ url: 'https://{tenant}.exemple.fr' }],
				}),
			),
		)

		expect(
			await within(dialog).findByText(/variable sans valeur par défaut/),
		).toBeDefined()
		expect(
			within(dialog).queryByRole('button', { name: 'Ajouter la collection' }),
		).toBeNull()
		expect(api.calls.some((call) => call.method === 'post')).toBe(false)
	})

	it('tells that the collection is too large (413) and keeps the preview', async () => {
		const { user, dialog, input } = await openFileImport((fake) =>
			fake.mockMutation('post', COLLECTIONS_PATH, () => {
				throw apiError(413, 'the request body is larger than 2359296 bytes')
			}),
		)
		await user.upload(input, specFile(specDocument()))
		await user.click(
			await within(dialog).findByRole('button', {
				name: 'Ajouter la collection',
			}),
		)

		expect(
			await within(dialog).findByText(/Cette collection est trop volumineuse/),
		).toBeDefined()
		expect(
			within(dialog).getByRole('button', { name: 'Ajouter la collection' }),
		).toBeDefined()
	})

	it('tells that the 200 collections are reached (409)', async () => {
		const { user, dialog, input } = await openFileImport((fake) =>
			fake.mockMutation('post', COLLECTIONS_PATH, () => {
				throw apiError(
					409,
					'Conflict: an organization holds at most 200 collections',
				)
			}),
		)
		await user.upload(input, specFile(specDocument()))
		await user.click(
			await within(dialog).findByRole('button', {
				name: 'Ajouter la collection',
			}),
		)

		expect(
			await within(dialog).findByText(
				'Cette organisation a atteint la limite de 200 collections. Supprimez-en une avant d’en importer une autre.',
			),
		).toBeDefined()
	})

	it('relays the reason of a refused model (400)', async () => {
		const { user, dialog, input } = await openFileImport((fake) =>
			fake.mockMutation('post', COLLECTIONS_PATH, () => {
				throw apiError(
					400,
					'Validation failed: invalid collection: operation `searchAddress` is declared more than once',
				)
			}),
		)
		await user.upload(input, specFile(specDocument()))
		await user.click(
			await within(dialog).findByRole('button', {
				name: 'Ajouter la collection',
			}),
		)

		expect(
			await within(dialog).findByText(
				'Le serveur refuse cette collection : operation `searchAddress` is declared more than once.',
			),
		).toBeDefined()
	})

	it('tells when the caller lost the right to import (403)', async () => {
		const { user, dialog, input } = await openFileImport((fake) =>
			fake.mockMutation('post', COLLECTIONS_PATH, () => {
				throw apiError(403, 'Forbidden')
			}),
		)
		await user.upload(input, specFile(specDocument()))
		await user.click(
			await within(dialog).findByRole('button', {
				name: 'Ajouter la collection',
			}),
		)

		expect(
			await within(dialog).findByText(
				'Vous n’avez pas la permission de gérer les collections.',
			),
		).toBeDefined()
	})
})

describe('CollectionDetailFeature', () => {
	it('shows the stored collection with its version history and security', async () => {
		await renderFeature(<CollectionDetailFeature collectionId={ADDRESS_ID} />, {
			configure: (api) =>
				api.mockGet(COLLECTION_PATH, () => ({
					data: storedCollection({
						version: '1.2',
						history: [
							{
								version: '1.1',
								imported_at: '2026-10-20T08:00:00Z',
								source: { kind: 'file', file_name: 'adresse-1.1.json' },
								base_url: 'https://api-adresse.data.gouv.fr',
								security: [],
							},
							{
								version: '1.0',
								imported_at: '2026-10-01T08:00:00Z',
								source: { kind: 'url', url: 'https://adresse.test/o.json' },
								base_url: 'https://api-adresse.data.gouv.fr',
								security: [],
							},
						],
					}),
					pagination: null,
				})),
		})

		expect(
			await screen.findByRole('heading', { name: 'API Adresse' }),
		).toBeDefined()
		expect(screen.getByText('Géocodage des adresses françaises.')).toBeDefined()
		expect(screen.getByText('bearerAuth (http bearer)')).toBeDefined()
		const previous = screen.getByText(/précédentes/)
		expect(previous.textContent).toContain('1.1, 1.0')
		expect(
			screen.getByTitle(
				'Importée depuis le fichier adresse-1.1.json, le 20 octobre 2026',
			),
		).toBeDefined()
		expect(screen.getByText('2 opérations')).toBeDefined()
	})

	it('expands an operation to show its contract', async () => {
		const user = userEvent.setup()
		await renderFeature(<CollectionDetailFeature collectionId={ADDRESS_ID} />)

		await user.click(
			(
				await screen.findAllByRole('button', { expanded: false })
			)[0] as HTMLElement,
		)

		expect(
			screen.getByRole('tab', { name: 'Contrat TypeScript' }),
		).toBeDefined()
		expect(screen.getAllByText('/search').length).toBeGreaterThan(0)
	})

	it('says so when the collection does not exist', async () => {
		await renderFeature(<CollectionDetailFeature collectionId="nope" />)

		expect(await screen.findByText('Collection introuvable')).toBeDefined()
		expect(
			screen
				.getByRole('link', { name: 'Voir les collections' })
				.getAttribute('href'),
		).toBe('/o/atelier-bois/automatisation/collections')
	})

	it('says so while the collection loads', async () => {
		await renderFeature(<CollectionDetailFeature collectionId={ADDRESS_ID} />, {
			configure: (api) =>
				api.mockGet(COLLECTION_PATH, () => new Promise(() => {})),
		})

		expect(
			await screen.findByText('Chargement de la collection…'),
		).toBeDefined()
		expect(screen.queryByText('Collection introuvable')).toBeNull()
	})

	it('says so when the collection cannot be loaded', async () => {
		await renderFeature(<CollectionDetailFeature collectionId={ADDRESS_ID} />, {
			configure: (api) =>
				api.mockGet(COLLECTION_PATH, () => {
					throw apiError(500, 'Internal server error')
				}),
		})

		expect(
			await screen.findByText('Impossible de charger cette collection'),
		).toBeDefined()
		expect(screen.queryByText('Collection introuvable')).toBeNull()
	})

	it('links the breadcrumb back to the collections', async () => {
		await renderFeature(<CollectionDetailFeature collectionId={ADDRESS_ID} />)

		expect(
			(await screen.findByRole('link', { name: 'Collections' })).getAttribute(
				'href',
			),
		).toBe('/o/atelier-bois/automatisation/collections')
	})

	it('offers the update buttons only to a caller who can manage automation, and the url check only for a url source', async () => {
		const { unmount } = await renderFeature(
			<CollectionDetailFeature collectionId={ADDRESS_ID} />,
		)
		await screen.findByRole('heading', { name: 'API Adresse' })
		expect(
			screen.getByRole('button', { name: 'Importer une version' }),
		).toBeDefined()
		expect(
			screen.getByRole('button', { name: 'Vérifier les mises à jour' }),
		).toBeDefined()
		unmount()

		const fileSourced = await renderFeature(
			<CollectionDetailFeature collectionId={COMPANY_ID} />,
		)
		await screen.findByRole('heading', { name: 'Recherche d’entreprises' })
		expect(
			screen.getByRole('button', { name: 'Importer une version' }),
		).toBeDefined()
		expect(
			screen.queryByRole('button', { name: 'Vérifier les mises à jour' }),
		).toBeNull()
		fileSourced.unmount()

		await renderFeature(<CollectionDetailFeature collectionId={ADDRESS_ID} />, {
			permissions: ['VIEW_AUTOMATION'],
		})
		await screen.findByRole('heading', { name: 'API Adresse' })
		expect(
			screen.queryByRole('button', { name: 'Importer une version' }),
		).toBeNull()
		expect(
			screen.queryByRole('button', { name: 'Vérifier les mises à jour' }),
		).toBeNull()
	})
})

describe('CollectionDetailFeature — update', () => {
	function updatableApi() {
		let current = storedCollection()
		return (api: ReturnType<typeof installFakeTanstackApi>) => {
			api.mockGet(COLLECTION_PATH, () => ({ data: current, pagination: null }))
			api.mockMutation('put', COLLECTION_PATH, (params) => {
				const body = (params as { body: Schemas.CollectionRequest }).body
				current = storedCollection({
					version: body.version,
					source: body.source,
					operations: body.operations,
					history: [
						{
							version: current.version,
							imported_at: current.imported_at,
							source: current.source,
							base_url: current.base_url,
							security: current.security,
						},
					],
				})
				return { data: current }
			})
		}
	}

	const RENAMED_SEARCH = {
		'/search': {
			get: {
				operationId: 'search_address_v2',
				summary: 'Chercher une adresse',
				tags: ['Recherche'],
				parameters: [
					{
						name: 'q',
						in: 'query',
						required: true,
						schema: { type: 'string' },
					},
					{
						name: 'limit',
						in: 'query',
						required: true,
						schema: { type: 'integer' },
					},
				],
				responses: { '200': { description: 'ok' } },
			},
		},
	}

	it('checks the source url, shows the diff, then replaces the stored collection and keeps the operation ids', async () => {
		const user = userEvent.setup()
		vi.stubGlobal(
			'fetch',
			vi.fn(async () =>
				Response.json(specDocument({ version: '2.0', paths: RENAMED_SEARCH })),
			),
		)
		const { api } = await renderFeature(
			<CollectionDetailFeature collectionId={ADDRESS_ID} />,
			{ configure: updatableApi() },
		)

		await user.click(
			await screen.findByRole('button', { name: 'Vérifier les mises à jour' }),
		)
		const dialog = await screen.findByRole('dialog', {
			name: 'Mettre à jour « API Adresse »',
		})
		expect(await within(dialog).findByText('Opération supprimée')).toBeDefined()
		expect(
			within(dialog).getByText(/Nouveau paramètre requis « limit »/),
		).toBeDefined()
		await user.click(
			within(dialog).getByRole('button', { name: 'Mettre à jour vers 2.0' }),
		)

		expect(
			await screen.findByText(/Collection mise à jour vers la version 2\.0\./),
		).toBeDefined()
		const put = api.calls.find((call) => call.method === 'put')
		expect(put?.params).toMatchObject({
			path: { organization_id: 'org-1', collection_id: ADDRESS_ID },
			body: {
				version: '2.0',
				source: { kind: 'url', url: 'https://adresse.test/openapi.json' },
			},
		})
		const body = (put?.params as { body: Schemas.CollectionRequest }).body
		expect(body.operations.map((operation) => operation.id)).toEqual([
			'searchAddress',
		])
		await waitFor(() =>
			expect(screen.getByText(/précédentes/).textContent).toContain('1.0'),
		)
	})

	it('compares a spec chosen as a file', async () => {
		const user = userEvent.setup()
		const { api } = await renderFeature(
			<CollectionDetailFeature collectionId={ADDRESS_ID} />,
			{ configure: updatableApi() },
		)

		await user.click(
			await screen.findByRole('button', { name: 'Importer une version' }),
		)
		const dialog = await screen.findByRole('dialog', {
			name: 'Mettre à jour « API Adresse »',
		})
		await user.click(within(dialog).getByRole('tab', { name: 'Fichier' }))
		await user.upload(
			within(dialog).getByLabelText(/Déposez un fichier/),
			specFile(specDocument({ version: '1.5' }), 'adresse-1.5.json'),
		)
		await user.click(
			await within(dialog).findByRole('button', {
				name: 'Mettre à jour vers 1.5',
			}),
		)

		expect(
			await screen.findByText(/Collection mise à jour vers la version 1\.5\./),
		).toBeDefined()
		expect(
			(
				api.calls.find((call) => call.method === 'put')?.params as {
					body: Schemas.CollectionRequest
				}
			).body.source,
		).toEqual({ kind: 'file', file_name: 'adresse-1.5.json' })
	})

	it('tells that the collection is already up to date', async () => {
		const user = userEvent.setup()
		vi.stubGlobal(
			'fetch',
			vi.fn(async () => Response.json(specDocument({ version: '1.0' }))),
		)
		const { api } = await renderFeature(
			<CollectionDetailFeature collectionId={ADDRESS_ID} />,
			{ configure: updatableApi() },
		)

		await user.click(
			await screen.findByRole('button', { name: 'Vérifier les mises à jour' }),
		)

		expect(await screen.findByText(/La collection est à jour/)).toBeDefined()
		expect(api.calls.some((call) => call.method === 'put')).toBe(false)
	})

	it('shows the reason of a refused replacement in the dialog and keeps the diff', async () => {
		const user = userEvent.setup()
		vi.stubGlobal(
			'fetch',
			vi.fn(async () => Response.json(specDocument({ version: '2.0' }))),
		)
		await renderFeature(<CollectionDetailFeature collectionId={ADDRESS_ID} />, {
			configure: (api) =>
				api.mockMutation('put', COLLECTION_PATH, () => {
					throw apiError(413, 'the collection model holds 3 bytes')
				}),
		})

		await user.click(
			await screen.findByRole('button', { name: 'Vérifier les mises à jour' }),
		)
		const dialog = await screen.findByRole('dialog')
		await user.click(
			await within(dialog).findByRole('button', {
				name: 'Mettre à jour vers 2.0',
			}),
		)

		expect(
			await within(dialog).findByText(/Cette collection est trop volumineuse/),
		).toBeDefined()
		expect(
			within(dialog).getByRole('button', { name: 'Mettre à jour vers 2.0' }),
		).toBeDefined()
	})

	it('explains a failed url check as a CORS problem', async () => {
		const user = userEvent.setup()
		vi.stubGlobal(
			'fetch',
			vi.fn(async () => {
				throw new TypeError('Failed to fetch')
			}),
		)
		await renderFeature(<CollectionDetailFeature collectionId={ADDRESS_ID} />)

		await user.click(
			await screen.findByRole('button', { name: 'Vérifier les mises à jour' }),
		)

		expect(await screen.findByText(/\(CORS\)/)).toBeDefined()
	})
})
