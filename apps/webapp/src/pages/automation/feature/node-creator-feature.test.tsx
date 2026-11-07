import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import {
	type CreatorChoice,
	NodeCreatorFeature,
} from '#/pages/automation/feature/node-creator-feature'
import { installFakeTanstackApi } from '#/pages/automation/test/fake-tanstack-api'
import { renderWithRouter } from '#/test/render-with-router'

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

const PETS_ID = '6f1f6a9e-5f0c-4a43-9d77-1d9d0a3b1c11'
const SHOP_ID = '0b7a3c1e-2d54-4c85-8b0d-6a4f1d2e9a77'

function collectionResponse(
	overrides: Partial<Schemas.CollectionResponse> &
		Pick<Schemas.CollectionResponse, 'id' | 'title' | 'operations'>,
): Schemas.CollectionResponse {
	return {
		organization_id: ORGANIZATION.id,
		slug: overrides.title.toLowerCase(),
		version: '1.0',
		description: '',
		base_url: 'https://api.test',
		source: { kind: 'url', url: 'https://api.test/openapi.json' },
		imported_at: '2026-11-06T09:30:00Z',
		security: [],
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

const PETS = collectionResponse({
	id: PETS_ID,
	title: 'Petstore',
	base_url: 'https://pets.test',
	operations: [
		{
			id: 'getPet',
			method: 'GET',
			path: '/pets/{id}',
			summary: 'Lire un animal',
			tag: 'pets',
			parameters: [
				{
					name: 'id',
					location: 'path',
					required: true,
					description: '',
					schema: { type: 'string' },
				},
			],
			request_body: null,
			response: null,
			response_status: '200',
		},
		{
			id: 'createPet',
			method: 'POST',
			path: '/pets',
			summary: 'Créer un animal',
			tag: 'pets',
			parameters: [],
			request_body: {
				schema: {
					type: 'object',
					required: ['name'],
					properties: { name: { type: 'string' } },
				},
			},
			response: null,
			response_status: '201',
		},
	],
})

const SHOP = collectionResponse({
	id: SHOP_ID,
	title: 'Boutique',
	operations: [
		{
			id: 'listOrders',
			method: 'GET',
			path: '/orders',
			summary: 'Lister les commandes',
			tag: 'orders',
			parameters: [],
			request_body: null,
			response: null,
			response_status: '200',
		},
	],
})

function textField(name: string): Schemas.FieldResponse {
	return {
		expression: false,
		kind: 'Text',
		label: name,
		name,
		required: false,
		secret: false,
		visible_when: null,
	}
}

const HTTP_DESCRIPTOR: Schemas.ConnectorDescriptorResponse = {
	kind: 'http.request',
	family: 'http',
	label: 'Requête HTTP',
	version: 1,
	auth: 'None',
	fields: [textField('method'), textField('url')],
	branches: [],
	output_example: null,
}

const OPERATION_DESCRIPTOR: Schemas.ConnectorDescriptorResponse = {
	kind: 'collection.operation',
	family: 'http',
	label: "Opération d'une collection",
	version: 1,
	auth: { Optional: ['bearer_token'] },
	fields: [],
	branches: [],
	output_example: null,
}

function descriptorOf(
	kind: string,
	family: string,
	label: string,
): Schemas.ConnectorDescriptorResponse {
	return {
		kind,
		family,
		label,
		version: 1,
		auth: 'None',
		fields: [],
		branches: [],
		output_example: null,
	}
}

const CATALOGUE: Schemas.ConnectorDescriptorResponse[] = [
	descriptorOf('odoo.create_invoice', 'odoo', 'Créer une facture Odoo'),
	HTTP_DESCRIPTOR,
	OPERATION_DESCRIPTOR,
	descriptorOf('flow.condition', 'flow', 'Condition'),
	descriptorOf(
		'mestier.task_recurrence.extend_horizon',
		'mestier',
		'Étendre l’horizon de récurrence',
	),
	descriptorOf('mestier.support.add_badge', 'mestier', 'Ajouter un badge'),
	descriptorOf('mestier.customer.create', 'mestier', 'Créer un client'),
]

function apiError(status: number, message: string) {
	return Object.assign(new Error(message), { status })
}

async function renderCreator({
	hasTrigger = true,
	descriptors = CATALOGUE,
	configure = () => {},
}: {
	hasTrigger?: boolean
	descriptors?: Schemas.ConnectorDescriptorResponse[]
	configure?: (api: ReturnType<typeof installFakeTanstackApi>) => void
} = {}) {
	const onCreate = vi.fn<(choice: CreatorChoice) => void>()
	const api = installFakeTanstackApi()
	api.mockGet(COLLECTIONS_PATH, () => ({
		data: [summaryOf(PETS), summaryOf(SHOP)],
		pagination: null,
	}))
	api.mockGet(COLLECTION_PATH, (params) => {
		const id = (params as { path: { collection_id: string } }).path
			.collection_id
		if (id === PETS_ID) return { data: PETS, pagination: null }
		if (id === SHOP_ID) return { data: SHOP, pagination: null }
		throw apiError(404, 'Resource not found')
	})
	configure(api)
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false, retryDelay: 0 } },
	})
	await renderWithRouter(
		<QueryClientProvider client={queryClient}>
			<OrganizationListProvider organizations={[ORGANIZATION]}>
				<ActiveOrganizationProvider activeOrganization={ORGANIZATION}>
					<NodeCreatorFeature
						organizationSlug="atelier-bois"
						descriptors={descriptors}
						hasTrigger={hasTrigger}
						onCreate={onCreate}
						onClose={vi.fn()}
					/>
				</ActiveOrganizationProvider>
			</OrganizationListProvider>
		</QueryClientProvider>,
	)
	return {
		api,
		onCreate,
		panel: screen.getByRole('complementary', { name: 'Créateur de nœuds' }),
	}
}

function rowsOf(panel: HTMLElement): string[] {
	return within(panel)
		.getAllByRole('listitem')
		.map((item) => item.textContent ?? '')
}

function detailCalls(api: ReturnType<typeof installFakeTanstackApi>) {
	return api.calls.filter((call) => call.path === COLLECTION_PATH)
}

describe('NodeCreatorFeature — the catalogue it offers', () => {
	it('lists the prototype categories in order, then the trigger entry, with no Odoo or empty category', async () => {
		const { panel } = await renderCreator()

		expect(rowsOf(panel)).toEqual([
			'Actions MestierAgir dans Mestier : projets, tickets, devis, discussions',
			'CollectionsAppeler une API importée depuis une spec OpenAPI',
			'LogiqueBrancher, fusionner ou mettre en pause le workflow',
			'OutilsExécuter du code, envoyer un e-mail, appeler une API',
		])
		expect(
			within(panel).getByRole('button', { name: /Ajouter un déclencheur/ }),
		).toBeDefined()
		expect(within(panel).queryByText('Odoo')).toBeNull()
		expect(within(panel).queryByText('Validation humaine')).toBeNull()
	})

	it('shows each type with its own description, the Odoo ones among the tools', async () => {
		const user = userEvent.setup()
		const { panel } = await renderCreator()

		await user.click(within(panel).getByRole('button', { name: /Outils/ }))

		expect(rowsOf(panel)).toEqual([
			'Requête HTTPAppelle une API externe',
			'Créer une facture OdooCrée une facture client dans Odoo',
		])
	})

	it('never offers the internal horizon extension', async () => {
		const user = userEvent.setup()
		const { panel } = await renderCreator()

		await user.click(
			within(panel).getByRole('button', { name: /Actions Mestier/ }),
		)

		expect(rowsOf(panel)).toEqual([
			'Créer un clientCrée un client dans Mestier',
			'Ajouter un badge au ticketAjoute un badge de couleur à un ticket',
		])
	})

	it('starts on the triggers when the workflow has none, event first', async () => {
		const { panel } = await renderCreator({ hasTrigger: false })

		expect(
			within(panel).getByText('Qu’est-ce qui déclenche ce workflow ?'),
		).toBeDefined()
		expect(rowsOf(panel)).toEqual([
			'Event MestierSe déclenche quand un événement se produit dans Mestier',
			'ClicSe déclenche quand vous cliquez sur « Tester le workflow »',
		])
	})

	it('searches the description as well as the label, grouped by category', async () => {
		const user = userEvent.setup()
		const { panel } = await renderCreator()

		await user.type(
			within(panel).getByRole('searchbox', { name: 'Rechercher un nœud' }),
			'couleur',
		)

		expect(
			within(panel).getByRole('region', { name: 'Actions Mestier' }),
		).toBeDefined()
		await waitFor(() => {
			expect(rowsOf(panel)).toEqual([
				'Ajouter un badge au ticketAjoute un badge de couleur à un ticket',
			])
		})
	})

	it('does not offer the collection operation as a node of its own, in a category or in a search', async () => {
		const user = userEvent.setup()
		const { panel } = await renderCreator()

		await user.click(within(panel).getByRole('button', { name: /Outils/ }))
		expect(within(panel).queryByText(/Opération de collection/)).toBeNull()
		await user.click(
			within(panel).getByRole('button', { name: 'Revenir aux catégories' }),
		)
		await user.type(
			within(panel).getByRole('searchbox', { name: 'Rechercher un nœud' }),
			'opération',
		)

		await waitFor(() => {
			expect(within(panel).queryByText(/Chargement|Recherche dans/)).toBeNull()
		})
		expect(
			within(panel).queryByRole('button', { name: /Opération de collection/ }),
		).toBeNull()
	})

	it('hides the collections when the server knows no collection operation', async () => {
		const { panel } = await renderCreator({
			descriptors: [
				HTTP_DESCRIPTOR,
				descriptorOf('flow.condition', 'flow', 'Si'),
			],
		})

		expect(
			within(panel).queryByRole('button', { name: /Collections/ }),
		).toBeNull()
	})
})

describe('NodeCreatorFeature — the collections', () => {
	it('lists the collections of the organization, without reading their operations yet', async () => {
		const user = userEvent.setup()
		const { panel, api } = await renderCreator()

		await user.click(within(panel).getByRole('button', { name: /Collections/ }))

		expect(
			await within(panel).findByRole('button', { name: /Petstore/ }),
		).toBeDefined()
		expect(rowsOf(panel)).toEqual([
			'Petstore2 opérations',
			'Boutique1 opération',
		])
		expect(detailCalls(api)).toHaveLength(0)
	})

	it('says so while the collections load', async () => {
		const user = userEvent.setup()
		const { panel } = await renderCreator({
			configure: (api) =>
				api.mockGet(COLLECTIONS_PATH, () => new Promise(() => {})),
		})

		await user.click(within(panel).getByRole('button', { name: /Collections/ }))

		expect(
			await within(panel).findByText('Chargement des collections…'),
		).toBeDefined()
		expect(
			within(panel).queryByText('Aucune collection importée pour le moment.'),
		).toBeNull()
	})

	it('says when the collections cannot be read', async () => {
		const user = userEvent.setup()
		const { panel } = await renderCreator({
			configure: (api) =>
				api.mockGet(COLLECTIONS_PATH, () => {
					throw apiError(403, 'Forbidden')
				}),
		})

		await user.click(within(panel).getByRole('button', { name: /Collections/ }))

		expect(
			await within(panel).findByText(
				'Vous n’avez pas la permission de voir les collections.',
			),
		).toBeDefined()
	})

	it('sends to the collections page when no collection has been imported', async () => {
		const user = userEvent.setup()
		const { panel } = await renderCreator({
			configure: (api) =>
				api.mockGet(COLLECTIONS_PATH, () => ({ data: [], pagination: null })),
		})

		await user.click(within(panel).getByRole('button', { name: /Collections/ }))

		expect(
			(
				await within(panel).findByRole('link', {
					name: 'Importer une collection',
				})
			).getAttribute('href'),
		).toBe('/o/atelier-bois/automatisation/collections')
	})

	it('reads the operations of the collection picked, and of that one only', async () => {
		const user = userEvent.setup()
		const { panel, api } = await renderCreator()

		await user.click(within(panel).getByRole('button', { name: /Collections/ }))
		await user.click(
			await within(panel).findByRole('button', { name: /Petstore/ }),
		)

		expect(
			await within(panel).findByRole('button', { name: /Lire un animal/ }),
		).toBeDefined()
		expect(rowsOf(panel)).toEqual([
			'GETLire un animal/pets/{id}',
			'POSTCréer un animal/pets',
		])
		expect(within(panel).getByText('https://pets.test')).toBeDefined()
		expect(detailCalls(api).map((call) => call.params)).toEqual([
			{ path: { organization_id: 'org-1', collection_id: PETS_ID } },
		])
	})

	it('says when the collection has gone', async () => {
		const user = userEvent.setup()
		const { panel } = await renderCreator({
			configure: (api) =>
				api.mockGet(COLLECTION_PATH, () => {
					throw apiError(404, 'Resource not found')
				}),
		})

		await user.click(within(panel).getByRole('button', { name: /Collections/ }))
		await user.click(
			await within(panel).findByRole('button', { name: /Petstore/ }),
		)

		expect(
			await within(panel).findByText('Cette collection n’est plus disponible.'),
		).toBeDefined()
	})

	it('says the collection could not be read, not that it is gone, on any other failure', async () => {
		const user = userEvent.setup()
		const { panel } = await renderCreator({
			configure: (api) =>
				api.mockGet(COLLECTION_PATH, () => {
					throw apiError(500, 'Internal server error')
				}),
		})

		await user.click(within(panel).getByRole('button', { name: /Collections/ }))
		await user.click(
			await within(panel).findByRole('button', { name: /Petstore/ }),
		)

		expect(
			await within(panel).findByText('Impossible de lire cette collection.'),
		).toBeDefined()
		expect(within(panel).queryByText(/n’est plus disponible/)).toBeNull()
	})

	it('creates a node that holds the reference of the operation, not its url', async () => {
		const user = userEvent.setup()
		const { panel, onCreate } = await renderCreator()

		await user.click(within(panel).getByRole('button', { name: /Collections/ }))
		await user.click(
			await within(panel).findByRole('button', { name: /Petstore/ }),
		)
		await user.click(
			await within(panel).findByRole('button', { name: /Créer un animal/ }),
		)

		expect(onCreate).toHaveBeenCalledTimes(1)
		expect(onCreate).toHaveBeenCalledWith({
			type: 'connector',
			descriptor: OPERATION_DESCRIPTOR,
			preset: {
				name: 'Créer un animal',
				description: 'POST /pets',
				operation: { collection_id: PETS_ID, operation_id: 'createPet' },
				config: { body: { name: '' } },
			},
		})
		expect(JSON.stringify(onCreate.mock.calls)).not.toContain('pets.test')
	})

	it('finds an operation of any collection from the search, reading them only then', async () => {
		const user = userEvent.setup()
		const { panel, api } = await renderCreator()
		expect(detailCalls(api)).toHaveLength(0)

		await user.type(
			within(panel).getByRole('searchbox', { name: 'Rechercher un nœud' }),
			'commandes',
		)

		const results = await within(panel).findByRole('region', {
			name: 'Opérations de collections',
		})
		expect(within(results).getByText('Lister les commandes')).toBeDefined()
		expect(within(results).getByText('Boutique · /orders')).toBeDefined()
		expect(detailCalls(api)).toHaveLength(2)
	})

	it('creates the node of an operation found by the search', async () => {
		const user = userEvent.setup()
		const { panel, onCreate } = await renderCreator()

		await user.type(
			within(panel).getByRole('searchbox', { name: 'Rechercher un nœud' }),
			'commandes',
		)
		await user.click(
			await within(panel).findByRole('button', {
				name: /Lister les commandes/,
			}),
		)

		expect(onCreate).toHaveBeenCalledWith({
			type: 'connector',
			descriptor: OPERATION_DESCRIPTOR,
			preset: {
				name: 'Lister les commandes',
				description: 'GET /orders',
				operation: { collection_id: SHOP_ID, operation_id: 'listOrders' },
				config: {},
			},
		})
	})
})
