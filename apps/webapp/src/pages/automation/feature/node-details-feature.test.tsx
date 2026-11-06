import { screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { TooltipProvider } from '#/components/ui/tooltip'
import type { CredentialKind } from '#/hooks/use-automation'
import { NodeDetailsFeature } from '#/pages/automation/feature/node-details-feature'
import type { CollectionRead } from '#/pages/automation/lib/collection-node'
import type { Collection } from '#/pages/automation/lib/collections'
import {
	connectorNode,
	type EditorEdge,
	type EditorLookup,
	type EditorNode,
	triggerNode,
} from '#/pages/automation/lib/editor-nodes'
import type { NodeIssue } from '#/pages/automation/lib/node-issues'
import type { UpstreamSources } from '#/pages/automation/lib/upstream-entries'
import {
	type DryRunPasses,
	dryRunPassesByConnector,
} from '#/pages/automation/lib/workflow-runs'
import {
	credential as credentialFixture,
	HTTP_HEADER,
	KINDS,
	ODOO_API,
} from '#/pages/automation/test/credential-fixtures'
import type { TestRun } from '#/pages/automation/ui/variables-panel'
import { renderWithRouter } from '#/test/render-with-router'

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

const NODE: EditorNode = {
	id: 'c1',
	type: 'automation',
	position: { x: 0, y: 0 },
	data: {
		kind: 'connector',
		label: 'Envoyer l’e-mail',
		subtitle: '',
		disabled: false,
		typeLabel: 'Envoyer un e-mail',
		branches: [],
		errors: [],
		runStatus: 'failed',
		connector: { id: 'c1', kind: 'mail.send', version: 1, config: {} },
	},
}

const FAILED_STEP: Schemas.RunStepResponse = {
	id: 's1',
	connector_id: 'c1',
	iteration_path: '',
	attempts: 1,
	status: 'failed',
	created_at: '2026-08-05T00:00:01Z',
	error: 'SMTP expiré',
}

describe('NodeDetailsFeature', () => {
	it('sends a failed node to the identifications', async () => {
		await renderWithRouter(
			<NodeDetailsFeature
				node={NODE}
				nodes={[NODE]}
				edges={[]}
				organizationSlug="atelier-bois"
				sources={{
					descriptors: new Map(),
					events: [],
					lastRun: null,
					runSteps: new Map([['c1', FAILED_STEP]]),
				}}
				collectionReads={new Map()}
				credentialKinds={KINDS}
				credentials={[]}
				issues={[]}
				dryRunSteps={null}
				testRun={{ running: false, start: vi.fn() }}
				onUpdateConnector={vi.fn()}
				onUpdateTriggerKind={vi.fn()}
				onUpdateDetails={vi.fn()}
				onCreateCredential={vi.fn()}
				onEvaluateExpression={vi.fn()}
				onClose={vi.fn()}
				onNavigate={vi.fn()}
			/>,
		)

		const dialog = await screen.findByRole('dialog')
		expect(
			within(dialog)
				.getByRole('link', { name: 'Ouvrir les identifications' })
				.getAttribute('href'),
		).toBe('/o/atelier-bois/automatisation/identifications')
	})
})

function descriptor(
	kind: string,
	family: string,
	overrides: Partial<Schemas.ConnectorDescriptorResponse> = {},
): Schemas.ConnectorDescriptorResponse {
	return {
		auth: 'None',
		branches: [],
		family,
		fields: [],
		kind,
		label: kind,
		output_example: { ok: true },
		version: 1,
		...overrides,
	}
}

function jsonField(name: string, label: string): Schemas.FieldResponse {
	return {
		expression: true,
		kind: 'Json',
		label,
		name,
		required: false,
		secret: false,
		visible_when: null,
	}
}

const OPERATION_DESCRIPTOR = descriptor('collection.operation', 'http', {
	auth: { Optional: ['bearer_token', 'http_basic', 'http_header'] },
	label: "Opération d'une collection",
	fields: [
		jsonField('path', 'Paramètres de chemin'),
		jsonField('query', 'Paramètres de requête'),
		jsonField('headers', 'En-têtes'),
		jsonField('body', 'Corps'),
	],
	output_example: {
		status: 200,
		headers: { 'content-type': 'application/json' },
		body: {},
	},
})

const LOOKUP: EditorLookup = {
	descriptors: new Map([
		[
			'flow.condition',
			descriptor('flow.condition', 'flow', {
				branches: ['Then', 'Else'],
				fields: [
					{
						expression: true,
						kind: 'Text',
						label: 'Condition',
						name: 'predicate',
						required: true,
						secret: false,
						visible_when: null,
					},
				],
			}),
		],
		[
			'http.request',
			descriptor('http.request', 'http', {
				auth: { Optional: ['bearer_token'] },
				fields: [
					{
						expression: false,
						kind: { Select: { options: [{ value: 'GET', label: 'GET' }] } },
						label: 'Méthode',
						name: 'method',
						required: true,
						secret: false,
						visible_when: null,
					},
					{
						expression: true,
						kind: 'Text',
						label: 'URL',
						name: 'url',
						required: true,
						secret: false,
						visible_when: null,
					},
				],
			}),
		],
		['collection.operation', OPERATION_DESCRIPTOR],
	]),
	events: [],
}

const SOURCES: UpstreamSources = {
	descriptors: LOOKUP.descriptors,
	events: [],
	lastRun: null,
	runSteps: new Map(),
}

const AT = { x: 0, y: 0 }

function edge(source: string, target: string, handle?: string): EditorEdge {
	return {
		id: `${source}->${target}:${handle ?? ''}`,
		source,
		target,
		sourceHandle: handle,
	}
}

interface OpenOptions {
	onUpdateConnector?: (patch: unknown) => void
	onUpdateTriggerKind?: (kind: Schemas.TriggerKindDto) => void
	onCreateCredential?: (
		body: Schemas.CreateCredentialRequest,
	) => Promise<Schemas.CredentialResponse & { secret: unknown }>
	credentials?: Schemas.CredentialResponse[]
	credentialKinds?: CredentialKind[]
	events?: Schemas.EventDescriptorResponse[]
	reads?: ReadonlyMap<string, CollectionRead>
	issues?: readonly NodeIssue[]
	runSteps?: Map<string, Schemas.RunStepResponse>
	lastRun?: UpstreamSources['lastRun']
	dryRunSteps?: DryRunPasses | null
	testRun?: TestRun | null
}

async function openNode(
	nodeId: string,
	nodes: EditorNode[],
	edges: EditorEdge[],
	handlers: OpenOptions = {},
) {
	const node = nodes.find((candidate) => candidate.id === nodeId)
	if (!node) throw new Error(`no node ${nodeId}`)
	await renderWithRouter(
		<TooltipProvider>
			<NodeDetailsFeature
				node={node}
				nodes={nodes}
				edges={edges}
				organizationSlug="atelier-bois"
				sources={{
					...SOURCES,
					...(handlers.events ? { events: handlers.events } : {}),
					...(handlers.runSteps ? { runSteps: handlers.runSteps } : {}),
					...(handlers.lastRun ? { lastRun: handlers.lastRun } : {}),
				}}
				collectionReads={handlers.reads ?? new Map()}
				credentialKinds={handlers.credentialKinds ?? KINDS}
				credentials={handlers.credentials ?? []}
				issues={handlers.issues ?? []}
				dryRunSteps={handlers.dryRunSteps ?? null}
				testRun={
					handlers.testRun === undefined
						? { running: false, start: vi.fn() }
						: handlers.testRun
				}
				onUpdateConnector={handlers.onUpdateConnector ?? vi.fn()}
				onUpdateTriggerKind={handlers.onUpdateTriggerKind ?? vi.fn()}
				onUpdateDetails={vi.fn()}
				onCreateCredential={handlers.onCreateCredential ?? vi.fn()}
				onEvaluateExpression={vi.fn()}
				onClose={vi.fn()}
				onNavigate={vi.fn()}
			/>
		</TooltipProvider>,
	)
	return screen.findByRole('dialog')
}

function chain(): { nodes: EditorNode[]; edges: EditorEdge[] } {
	return {
		nodes: [
			triggerNode({ id: 't1', kind: 'Manual' }, AT, LOOKUP),
			connectorNode(
				{ id: 'c1', kind: 'flow.condition', version: 1, config: {} },
				AT,
				LOOKUP,
			),
			connectorNode(
				{
					id: 'c2',
					kind: 'http.request',
					version: 1,
					config: { method: 'POST', url: 'https://a' },
					name: 'Appeler le fournisseur',
				},
				{ x: 300, y: 0 },
				LOOKUP,
			),
		],
		edges: [edge('t1', 'c1'), edge('c1', 'c2', 'Then')],
	}
}

describe('NodeDetailsFeature — what the header says of the node', () => {
	it('shows the label of the type in the badge', async () => {
		const { nodes, edges } = chain()
		const renamed = nodes.map((node) =>
			node.id === 'c1'
				? { ...node, data: { ...node.data, label: 'Vérifier le montant' } }
				: node,
		)

		const dialog = await openNode('c1', renamed, edges)

		expect(within(dialog).getByText('Si')).toBeDefined()
	})

	it('describes a node with no description by its type and its subtitle, never its family', async () => {
		const { nodes, edges } = chain()
		const cleared = nodes.map((node) =>
			node.id === 'c1'
				? { ...node, data: { ...node.data, subtitle: '' } }
				: node,
		)

		const dialog = await openNode('c1', cleared, edges)

		const described = dialog.getAttribute('aria-describedby')
		expect(document.getElementById(described ?? '')?.textContent).toBe(
			'Si · Condition',
		)
	})
})

describe('NodeDetailsFeature — the neighbours', () => {
	it('names the branch of a next node in French, and none for a previous one', async () => {
		const { nodes, edges } = chain()

		const dialog = await openNode('c1', nodes, edges)

		expect(
			within(dialog).getByRole('button', {
				name: 'Ouvrir le nœud suivant : Appeler le fournisseur (branche vrai)',
			}),
		).toBeDefined()
		expect(
			within(dialog).getByRole('button', {
				name: 'Ouvrir le nœud précédent : Clic',
			}),
		).toBeDefined()
	})

	it('names the branch the data comes from in the variables', async () => {
		const { nodes, edges } = chain()

		const dialog = await openNode('c2', nodes, edges)

		expect(
			within(dialog).getByText(/entrée directe, branche vrai/),
		).toBeDefined()
	})
})

describe('NodeDetailsFeature — the parameters', () => {
	it('leaves the parameters of the manual trigger empty', async () => {
		const { nodes, edges } = chain()

		const dialog = await openNode('t1', nodes, edges)

		const parameters = within(dialog).getByRole('region', {
			name: 'Paramètres',
		})
		expect(parameters.textContent).toBe('Paramètres')
		expect(within(parameters).queryByRole('button')).toBeNull()
		expect(within(parameters).queryByRole('textbox')).toBeNull()
	})

	it('seeds the defaults of a node that has none yet', async () => {
		const onUpdateConnector = vi.fn()
		const nodes = [
			triggerNode({ id: 't1', kind: 'Manual' }, AT, LOOKUP),
			connectorNode(
				{ id: 'c1', kind: 'http.request', version: 1, config: {} },
				AT,
				LOOKUP,
			),
		]

		await openNode('c1', nodes, [edge('t1', 'c1')], { onUpdateConnector })

		expect(onUpdateConnector).toHaveBeenCalledWith({
			config: { method: 'GET' },
		})
	})
})

describe('NodeDetailsFeature — what the server says of the name', () => {
	const REUSED =
		'connector `c1` reuses the name `Appeler le fournisseur`, already given to another node'

	function withErrors(
		errors: { field: string | null; message: string }[],
		kind: 'connector' | 'trigger' = 'connector',
	) {
		const { nodes, edges } = chain()
		return {
			edges,
			nodes: nodes.map((node) =>
				node.id === (kind === 'trigger' ? 't1' : 'c1')
					? { ...node, data: { ...node.data, errors } }
					: node,
			),
		}
	}

	it('says under the name, in French, that another node already has it', async () => {
		const { nodes, edges } = withErrors([{ field: 'name', message: REUSED }])

		const dialog = await openNode('c1', nodes, edges)

		const alert = within(dialog).getByRole('alert')
		expect(alert.textContent).toBe('Un autre nœud porte déjà ce nom.')
		expect(within(dialog).queryByText(REUSED)).toBeNull()
		const heading = alert.parentElement
		expect(
			within(heading as HTMLElement).getByLabelText('Nom du nœud', {
				exact: false,
			}),
		).toBeDefined()
	})

	it('shows a message it does not know as the server sent it', async () => {
		const { nodes, edges } = withErrors([
			{ field: 'name', message: 'nom refusé par le serveur' },
		])

		const dialog = await openNode('c1', nodes, edges)

		expect(within(dialog).getByRole('alert').textContent).toBe(
			'nom refusé par le serveur',
		)
	})

	it('puts the description error under the description', async () => {
		const { nodes, edges } = withErrors([
			{
				field: 'description',
				message: 'connector `c1` has a description longer than 500 characters',
			},
		])

		const dialog = await openNode('c1', nodes, edges)

		expect(within(dialog).getByRole('alert').textContent).toBe(
			'La description ne peut pas dépasser 500 caractères.',
		)
	})

	it('shows it once for a trigger, not again in the alert of its parameters', async () => {
		const { nodes, edges } = withErrors(
			[
				{
					field: 'name',
					message:
						'trigger `t1` reuses the name `Clic`, already given to another node',
				},
			],
			'trigger',
		)

		const dialog = await openNode('t1', nodes, edges)

		expect(within(dialog).getAllByRole('alert')).toHaveLength(1)
		expect(
			within(dialog).getAllByText('Un autre nœud porte déjà ce nom.'),
		).toHaveLength(1)
	})

	it('leaves the error of a connector field under that field, not under the name', async () => {
		const { nodes, edges } = chain()
		const named = nodes.map((node) =>
			node.id === 'c2'
				? {
						...node,
						data: {
							...node.data,
							errors: [{ field: 'url', message: 'URL manquante' }],
						},
					}
				: node,
		)

		const dialog = await openNode('c2', named, edges)

		expect(within(dialog).queryByText('URL manquante')).not.toBeNull()
		expect(
			within(dialog).queryByText('Un autre nœud porte déjà ce nom.'),
		).toBeNull()
	})
})

const COLLECTION_ID = '6f1f6a9e-5f0c-4a43-9d77-1d9d0a3b1c11'

const PETS: Collection = {
	id: COLLECTION_ID,
	slug: 'petstore',
	title: 'Petstore',
	version: '2.0',
	description: '',
	baseUrl: 'https://pets.test',
	source: { kind: 'url', url: 'https://pets.test/spec.json' },
	importedAt: '2026-11-06T09:30:00Z',
	security: [{ name: 'bearerAuth', kind: { type: 'http', scheme: 'bearer' } }],
	operations: [
		{
			id: 'updatePet',
			method: 'PUT',
			path: '/pets/{id}',
			summary: 'Modifier un animal',
			tag: 'pets',
			parameters: [
				{
					name: 'id',
					location: 'path',
					required: true,
					description: 'Identifiant',
					schema: { type: 'string' },
				},
				{
					name: 'limit',
					location: 'query',
					required: false,
					description: '',
					schema: { type: 'integer' },
				},
				{
					name: 'sort',
					location: 'query',
					required: true,
					description: '',
					schema: { type: 'string', enum: ['name', 'age'] },
				},
				{
					name: 'state',
					location: 'query',
					required: false,
					description: '',
					schema: { type: 'string', enum: ['open', 'closed'] },
				},
				{
					name: 'X-Tenant',
					location: 'header',
					required: true,
					description: '',
					schema: { type: 'string' },
				},
			],
			requestBody: {
				schema: {
					type: 'object',
					required: ['name'],
					properties: {
						name: { type: 'string' },
						age: { type: 'integer' },
					},
				},
			},
			response: { type: 'object', example: { id: 7, name: 'Rex' } },
			responseStatus: '200',
		},
	],
	schemas: {},
	history: [],
}

function foundReads(...collections: Collection[]) {
	return new Map<string, CollectionRead>(
		collections.map((item) => [item.id, { status: 'found', collection: item }]),
	)
}

function readsAs(status: 'loading' | 'notFound' | 'error') {
	return new Map<string, CollectionRead>([[COLLECTION_ID, { status }]])
}

function operationChain(
	config: Record<string, unknown>,
	extra: Partial<Schemas.PlacedConnectorDto> = {},
	errors: { field: string | null; message: string }[] = [],
) {
	const base = chain()
	const operationNode = connectorNode(
		{
			id: 'c2',
			kind: 'collection.operation',
			version: 1,
			config,
			name: 'Modifier un animal',
			description: 'PUT /pets/{id}',
			operation: { collection_id: COLLECTION_ID, operation_id: 'updatePet' },
			...extra,
		},
		{ x: 300, y: 0 },
		LOOKUP,
		errors,
	)
	return {
		edges: base.edges,
		nodes: base.nodes.map((node) => (node.id === 'c2' ? operationNode : node)),
	}
}

const SAVED = {
	path: { id: '5' },
	query: { sort: 'name' },
	headers: { 'X-Tenant': 'acme' },
	body: { name: 'Rex', age: null },
}

async function openOperation(
	config: Record<string, unknown> = SAVED,
	options: OpenOptions = {},
	extra: Partial<Schemas.PlacedConnectorDto> = {},
	errors: { field: string | null; message: string }[] = [],
) {
	const { nodes, edges } = operationChain(config, extra, errors)
	const dialog = await openNode('c2', nodes, edges, {
		reads: foundReads(PETS),
		...options,
	})
	return within(dialog)
}

function lastConfig(onUpdateConnector: ReturnType<typeof vi.fn>) {
	return (
		onUpdateConnector.mock.calls.at(-1)?.[0] as {
			config: Record<string, unknown>
		}
	).config
}

describe('NodeDetailsFeature — a node made from a collection operation', () => {
	it('heads the parameters with the operation, its summary and a link to its collection', async () => {
		const dialog = await openOperation()

		const parameters = dialog.getByRole('region', { name: 'Paramètres' })
		expect(within(parameters).getByText('PUT')).toBeDefined()
		expect(within(parameters).getByText('/pets/{id}')).toBeDefined()
		expect(within(parameters).getByText('Modifier un animal')).toBeDefined()
		expect(
			within(parameters)
				.getByRole('link', { name: 'Petstore' })
				.getAttribute('href'),
		).toBe(`/o/atelier-bois/automatisation/collections/${COLLECTION_ID}`)
		expect(within(parameters).getByText(/https:\/\/pets\.test/)).toBeDefined()
	})

	it('names it after the type in the badge, not as a generic http request', async () => {
		const dialog = await openOperation()

		expect(dialog.getByText('Opération de collection')).toBeDefined()
		expect(dialog.queryByText('Requête HTTP')).toBeNull()
	})

	it('offers one field per parameter of the spec, and none of the generic ones', async () => {
		const dialog = await openOperation()

		for (const label of ['id', 'limit', 'sort', 'state', 'X-Tenant']) {
			expect(dialog.getByLabelText(label), label).toBeDefined()
		}
		expect(dialog.getByLabelText('id').getAttribute('placeholder')).toBe(
			'Obligatoire',
		)
		expect(dialog.getByText('Chemin · obligatoire · Identifiant')).toBeDefined()
		expect(dialog.queryByText('Paramètres de chemin')).toBeNull()
		expect(dialog.queryByText('Paramètres de requête')).toBeNull()
		expect(dialog.queryByLabelText('Corps')).toBeNull()
		expect(dialog.getByText('Corps de la requête')).toBeDefined()
	})

	it('shows the value saved for a parameter in its field', async () => {
		const dialog = await openOperation()

		expect((dialog.getByLabelText('id') as HTMLInputElement).value).toBe('5')
		expect((dialog.getByLabelText('X-Tenant') as HTMLInputElement).value).toBe(
			'acme',
		)
		expect(dialog.getByLabelText('sort').textContent).toBe('name')
		expect(dialog.getByLabelText('state').textContent).toBe('Non renseigné')
	})

	it('tells what the spec declares under the identification', async () => {
		const dialog = await openOperation()

		expect(
			dialog.getByText(
				'La spec déclare : bearerAuth (http bearer). Aucune identification n’est appliquée automatiquement : choisissez celle à utiliser.',
			),
		).toBeDefined()
	})

	it('says the spec declares no authentication when it declares none', async () => {
		const dialog = await openOperation(SAVED, {
			reads: foundReads({ ...PETS, security: [] }),
		})

		expect(
			dialog.getByText(
				'La spec ne déclare pas d’authentification. Aucune identification n’est appliquée automatiquement : choisissez celle à utiliser.',
			),
		).toBeDefined()
	})

	it('writes a typed value under its place in the config, keeping the rest', async () => {
		const user = userEvent.setup()
		const onUpdateConnector = vi.fn()
		const dialog = await openOperation(SAVED, { onUpdateConnector })

		await user.type(dialog.getByLabelText('id'), '7')

		expect(lastConfig(onUpdateConnector)).toEqual({
			...SAVED,
			path: { id: '57' },
		})
	})

	it('writes a number as a number, and lets an expression through', async () => {
		const user = userEvent.setup()
		const onUpdateConnector = vi.fn()
		const dialog = await openOperation(SAVED, { onUpdateConnector })

		await user.type(dialog.getByLabelText('limit'), '25')
		expect(lastConfig(onUpdateConnector).query).toEqual({
			sort: 'name',
			limit: 25,
		})

		await user.clear(dialog.getByLabelText('limit'))
		await user.type(dialog.getByLabelText('limit'), '{{{{ trigger.n }}')
		expect(lastConfig(onUpdateConnector).query).toEqual({
			sort: 'name',
			limit: '{{ trigger.n }}',
		})
	})

	it('drops the entry of a parameter once its field is emptied, and the group when nothing is left', async () => {
		const user = userEvent.setup()
		const onUpdateConnector = vi.fn()
		const dialog = await openOperation(SAVED, { onUpdateConnector })

		await user.clear(dialog.getByLabelText('id'))

		expect(lastConfig(onUpdateConnector)).toEqual({
			query: SAVED.query,
			headers: SAVED.headers,
			body: SAVED.body,
		})
	})

	it('picks a value of a choice, and leaves an optional one unset again', async () => {
		const user = userEvent.setup()
		const onUpdateConnector = vi.fn()
		const dialog = await openOperation(SAVED, { onUpdateConnector })

		await user.click(dialog.getByLabelText('state'))
		await user.click(await screen.findByRole('option', { name: 'open' }))
		expect(lastConfig(onUpdateConnector).query).toEqual({
			sort: 'name',
			state: 'open',
		})
	})

	it('leaves an optional choice out of the config when set back to not filled in', async () => {
		const user = userEvent.setup()
		const onUpdateConnector = vi.fn()
		const dialog = await openOperation(
			{ ...SAVED, query: { sort: 'name', state: 'open' } },
			{ onUpdateConnector },
		)

		await user.click(dialog.getByLabelText('state'))
		await user.click(
			await screen.findByRole('option', { name: 'Non renseigné' }),
		)

		expect(lastConfig(onUpdateConnector).query).toEqual({ sort: 'name' })
	})

	it('derives the body fields from the schema, one typed row per key', async () => {
		const dialog = await openOperation()

		expect(
			(dialog.getByLabelText('Clé du name') as HTMLInputElement).value,
		).toBe('name')
		expect(
			(dialog.getByLabelText('Valeur de name') as HTMLInputElement).value,
		).toBe('Rex')
		expect(dialog.getByLabelText('Type de age').textContent).toBe('Nombre')
		expect(
			(dialog.getByLabelText('Valeur de age') as HTMLInputElement).value,
		).toBe('')
	})

	it('writes the body into the config next to the parameters', async () => {
		const user = userEvent.setup()
		const onUpdateConnector = vi.fn()
		const dialog = await openOperation(SAVED, { onUpdateConnector })

		await user.type(dialog.getByLabelText('Valeur de age'), '4')

		expect(lastConfig(onUpdateConnector)).toEqual({
			...SAVED,
			body: { name: 'Rex', age: 4 },
		})
	})

	it('shows the contract of the operation, request and response', async () => {
		const user = userEvent.setup()
		const dialog = await openOperation()

		expect(dialog.getByText('Contrat de l’opération')).toBeDefined()
		const contract = document.querySelector('[role="tabpanel"]')
		expect(contract?.textContent).toContain('export interface UpdatePetRequest')
		expect(contract?.textContent).toContain('export type UpdatePetResponse')

		await user.click(dialog.getByRole('tab', { name: 'Exemple de réponse' }))
		expect(document.body.textContent).toContain('"name": "Rex"')
	})

	it('shows the example of the response in the shape the node outputs, before any run', async () => {
		const dialog = await openOperation()

		const output = dialog.getByRole('region', { name: 'Sortie' })
		expect(output.textContent).toContain('Exemple de réponse 200 de')
		expect(output.textContent).toContain('"status": 200')
		expect(output.textContent).toContain('"content-type": "application/json"')
		expect(output.textContent).toContain('"body"')
		expect(output.textContent).toContain('"name": "Rex"')
	})

	it('shows what the node really output once it ran', async () => {
		const dialog = await openOperation(SAVED, {
			runSteps: new Map([
				[
					'c2',
					{
						...FAILED_STEP,
						connector_id: 'c2',
						status: 'succeeded',
						error: null,
						output: { status: 200, headers: {}, body: { id: 9 } },
					},
				],
			]),
		})

		const output = dialog.getByRole('region', { name: 'Sortie' })
		expect(output.textContent).not.toContain('Exemple de réponse')
		expect(output.textContent).toContain('1 élément')
		expect(output.textContent).toContain('body.id')
	})

	it('puts what the server said under the field it names', async () => {
		const dialog = await openOperation(SAVED, {}, {}, [
			{
				field: 'path.id',
				message: 'connector `c2` is missing required parameter `path.id`',
			},
			{
				field: 'body.name',
				message: 'connector `c2` sets parameter `body.name` to a bad value',
			},
			{
				field: 'operation',
				message: 'connector `c2` references unknown collection `x`',
			},
		])

		const alerts = dialog
			.getAllByRole('alert')
			.map((alert) => alert.textContent)
		expect(alerts).toEqual(
			expect.arrayContaining([
				'connector `c2` is missing required parameter `path.id`',
				'connector `c2` sets parameter `body.name` to a bad value',
				'connector `c2` references unknown collection `x`',
			]),
		)
		const underId = dialog
			.getByText('connector `c2` is missing required parameter `path.id`')
			.closest('.grid')
		expect(within(underId as HTMLElement).getByLabelText('id')).toBeDefined()
		const underBody = dialog
			.getByText('connector `c2` sets parameter `body.name` to a bad value')
			.closest('.grid')
		expect(
			within(underBody as HTMLElement).getByText('Corps de la requête'),
		).toBeDefined()
	})

	it('puts the issues of the node under the field they concern', async () => {
		const dialog = await openOperation(
			{ ...SAVED, query: { sort: 'name', legacy: '1' } },
			{
				issues: [
					{
						parameter: 'query.legacy',
						message:
							'Paramètre « legacy » inconnu de la spec 2.0 : il a peut-être été retiré.',
					},
					{
						parameter: 'body',
						message: 'Champ requis « name » absent du corps (spec 2.0).',
					},
				],
			},
		)

		const stale = dialog
			.getByText(
				'Paramètre « legacy » inconnu de la spec 2.0 : il a peut-être été retiré.',
			)
			.closest('.grid')
		expect(
			(
				within(stale as HTMLElement).getByLabelText(
					'legacy',
				) as HTMLInputElement
			).value,
		).toBe('1')
		expect(dialog.getByText('Absent de la spec')).toBeDefined()
		expect(
			dialog.getByText('Champ requis « name » absent du corps (spec 2.0).'),
		).toBeDefined()
	})

	it('says the operation cannot be found once the collection is known to be gone, without any field', async () => {
		const dialog = await openOperation(SAVED, {
			reads: readsAs('notFound'),
		})

		expect(
			dialog.getByText(
				'Opération introuvable : la collection a été supprimée ou modifiée.',
			),
		).toBeDefined()
		expect(dialog.queryByLabelText('id')).toBeNull()
		expect(dialog.queryByText('Contrat de l’opération')).toBeNull()
	})

	it('says the operation is gone when the collection no longer has it', async () => {
		const dialog = await openOperation(SAVED, {
			reads: foundReads({ ...PETS, operations: [] }),
		})

		expect(
			dialog.getByText(
				'Opération introuvable : la collection a été supprimée ou modifiée.',
			),
		).toBeDefined()
		expect(dialog.queryByLabelText('id')).toBeNull()
	})

	it('says the collection could not be read, not that it was deleted, on any other failure', async () => {
		const dialog = await openOperation(SAVED, { reads: readsAs('error') })

		expect(
			dialog.getByText('Impossible de lire la collection de cette opération.'),
		).toBeDefined()
		expect(dialog.queryByText(/introuvable/)).toBeNull()
		expect(dialog.queryByText(/supprimée/)).toBeNull()
		expect(dialog.queryByLabelText('id')).toBeNull()
		expect(dialog.queryByText('Contrat de l’opération')).toBeNull()
	})

	it('says it is loading rather than gone while the collection loads', async () => {
		const dialog = await openOperation(SAVED, { reads: readsAs('loading') })

		expect(dialog.getByText('Chargement de l’opération…')).toBeDefined()
		expect(dialog.queryByText(/introuvable/)).toBeNull()
	})

	it('waits too while the collection has not been asked for yet', async () => {
		const dialog = await openOperation(SAVED, { reads: new Map() })

		expect(dialog.getByText('Chargement de l’opération…')).toBeDefined()
	})
})

describe('NodeDetailsFeature — the dry run', () => {
	function dryStep(
		connectorId: string,
		patch: Partial<Schemas.DryRunStepResponse> = {},
	): Schemas.DryRunStepResponse {
		return {
			connector_id: connectorId,
			iteration_path: '',
			status: 'succeeded',
			input: null,
			output: null,
			error: null,
			...patch,
		}
	}

	function stepsOf(...steps: Schemas.DryRunStepResponse[]) {
		return dryRunPassesByConnector(steps)
	}

	const FILLED = stepsOf(
		dryStep('c2', {
			input: { url: 'https://fournisseur.test/commandes' },
			output: { numero: 'CMD-77' },
		}),
	)

	async function openExecutionTab(
		nodeId: string,
		options: OpenOptions = {},
	): Promise<HTMLElement> {
		const { nodes, edges } = chain()
		const dialog = await openNode(nodeId, nodes, edges, options)
		await userEvent.click(
			within(dialog).getByRole('radio', { name: 'Exécution' }),
		)
		return dialog
	}

	it('shows the input of the step in the input column and its output in the output column', async () => {
		const dialog = await openExecutionTab('c2', { dryRunSteps: FILLED })

		const input = within(dialog).getByRole('region', { name: 'Entrée' })
		expect(input.textContent).toContain('https://fournisseur.test/commandes')
		const output = within(dialog).getByRole('region', { name: 'Sortie' })
		expect(output.textContent).toContain('CMD-77')
	})

	it('tells the status of the step in the header', async () => {
		const dialog = await openExecutionTab('c2', { dryRunSteps: FILLED })

		expect(within(dialog).getByText('Succès')).toBeDefined()
	})

	it('shows the error of a failed step in the output column, with what the node was called with', async () => {
		const dialog = await openExecutionTab('c2', {
			dryRunSteps: stepsOf(
				dryStep('c2', {
					status: 'failed',
					input: { url: 'https://fournisseur.test/commandes' },
					error: 'connectors.c1.output.body.nope est introuvable',
				}),
			),
		})

		const output = within(dialog).getByRole('region', { name: 'Sortie' })
		expect(output.textContent).toContain('Erreur dans ce nœud')
		expect(output.textContent).toContain(
			'connectors.c1.output.body.nope est introuvable',
		)
		expect(
			within(dialog).getByRole('region', { name: 'Entrée' }).textContent,
		).toContain('https://fournisseur.test/commandes')
		expect(within(dialog).getByText('Échec')).toBeDefined()
	})

	it('says a node the test did not reach was not reached, in both columns', async () => {
		const dialog = await openExecutionTab('c2', {
			dryRunSteps: stepsOf(dryStep('c2', { status: 'skipped' })),
		})

		expect(
			within(within(dialog).getByRole('region', { name: 'Entrée' })).getByText(
				'Ce nœud n’a pas été atteint',
			),
		).toBeDefined()
		expect(
			within(within(dialog).getByRole('region', { name: 'Sortie' })).getByText(
				'Ce nœud n’a pas été exécuté',
			),
		).toBeDefined()
		expect(within(dialog).getByText('Non appelé')).toBeDefined()
	})

	it('shows a node left unreached by a failure as not reached, with no status of its own', async () => {
		const { nodes, edges } = chain()
		const withThird = [
			...nodes,
			connectorNode(
				{ id: 'c3', kind: 'http.request', version: 1, config: {} },
				{ x: 600, y: 0 },
				LOOKUP,
			),
		]
		const dialog = await openNode(
			'c3',
			withThird,
			[...edges, edge('c2', 'c3')],
			{
				dryRunSteps: dryRunPassesByConnector([
					dryStep('c1'),
					dryStep('c2', { status: 'failed', error: 'URL introuvable' }),
					dryStep('c3', { status: 'skipped' }),
				]),
			},
		)
		await userEvent.click(
			within(dialog).getByRole('radio', { name: 'Exécution' }),
		)

		expect(
			within(within(dialog).getByRole('region', { name: 'Entrée' })).getByText(
				'Ce nœud n’a pas été atteint',
			),
		).toBeDefined()
		expect(within(dialog).getByText('Non appelé')).toBeDefined()
	})

	it('tells a node outside the path of the test from a node the test skipped, once the test is over', async () => {
		const dialog = await openExecutionTab('c2', { dryRunSteps: stepsOf() })

		const input = within(dialog).getByRole('region', { name: 'Entrée' })
		expect(
			within(input).getByText('Ce nœud est hors du chemin du test'),
		).toBeDefined()
		expect(within(input).queryByText('Ce nœud n’a pas été atteint')).toBeNull()
		expect(within(input).queryByText('En attente du test')).toBeNull()
	})

	it('waits for the test to reach the node while it is still running', async () => {
		const dialog = await openExecutionTab('c2', {
			dryRunSteps: stepsOf(),
			testRun: { running: true, start: vi.fn() },
		})

		const input = within(dialog).getByRole('region', { name: 'Entrée' })
		expect(within(input).getByText('En attente du test')).toBeDefined()
		expect(within(input).queryByText('Ce nœud n’a pas été atteint')).toBeNull()
		expect(
			within(input).queryByText('Ce nœud est hors du chemin du test'),
		).toBeNull()
	})

	it('says the node received nothing when the step carries no input', async () => {
		const dialog = await openExecutionTab('c2', {
			dryRunSteps: stepsOf(dryStep('c2', { output: { ok: true } })),
		})

		expect(
			within(within(dialog).getByRole('region', { name: 'Entrée' })).getByText(
				'Aucune donnée en entrée',
			),
		).toBeDefined()
	})

	it('shows a disabled node as receiving what it lets pass', async () => {
		const { nodes, edges } = chain()
		const disabled = nodes.map((node) =>
			node.id === 'c2'
				? { ...node, data: { ...node.data, disabled: true } }
				: node,
		)
		const dialog = await openNode('c2', disabled, edges, {
			dryRunSteps: stepsOf(dryStep('c2', { output: { passe: 'oui' } })),
		})
		await userEvent.click(
			within(dialog).getByRole('radio', { name: 'Exécution' }),
		)

		expect(
			within(dialog).getByRole('region', { name: 'Entrée' }).textContent,
		).toContain('oui')
	})

	it('keeps the empty slots and the button before any test', async () => {
		const start = vi.fn()
		const dialog = await openExecutionTab('c2', {
			testRun: { running: false, start },
		})

		expect(
			within(dialog).getByText('Aucune exécution pour le moment'),
		).toBeDefined()
		expect(
			within(within(dialog).getByRole('region', { name: 'Sortie' })).getByText(
				'Ce nœud n’a pas été exécuté',
			),
		).toBeDefined()
		await userEvent.click(
			within(dialog).getByRole('button', { name: 'Tester le workflow' }),
		)
		expect(start).toHaveBeenCalledTimes(1)
	})

	it('offers no test button to somebody who cannot run one', async () => {
		const dialog = await openExecutionTab('c2', { testRun: null })

		expect(
			within(dialog).queryByRole('button', { name: 'Tester le workflow' }),
		).toBeNull()
	})

	it('prefers the dry run to the last real run for this node, and gives the real run back without it', async () => {
		const realStep: Schemas.RunStepResponse = {
			...FAILED_STEP,
			connector_id: 'c2',
			status: 'succeeded',
			error: null,
			output: { numero: 'REEL-1' },
		}
		const withDry = await openExecutionTab('c2', {
			runSteps: new Map([['c2', realStep]]),
			dryRunSteps: FILLED,
		})
		const output = within(withDry).getByRole('region', { name: 'Sortie' })
		expect(output.textContent).toContain('CMD-77')
		expect(output.textContent).not.toContain('REEL-1')
	})

	it('shows the last real run in the output column when no test has run', async () => {
		const realStep: Schemas.RunStepResponse = {
			...FAILED_STEP,
			connector_id: 'c2',
			status: 'succeeded',
			error: null,
			output: { numero: 'REEL-1' },
		}
		const dialog = await openExecutionTab('c2', {
			runSteps: new Map([['c2', realStep]]),
		})

		expect(
			within(dialog).getByRole('region', { name: 'Sortie' }).textContent,
		).toContain('REEL-1')
	})

	it('leaves the output of a trigger to the last real run: a dry run has no trigger payload', async () => {
		const { nodes, edges } = chain()
		const dialog = await openNode('t1', nodes, edges, {
			lastRun: { triggerPayload: { quote_id: 'q-real' }, connectorOutputs: {} },
			dryRunSteps: FILLED,
		})

		expect(
			within(dialog).getByRole('region', { name: 'Sortie' }).textContent,
		).toContain('q-real')
	})

	describe('a node of a loop', () => {
		const PASSES = stepsOf(
			dryStep('c2', {
				iteration_path: 'c1[0]',
				input: { url: 'https://un.test' },
				output: { numero: 'PASSAGE-A' },
			}),
			dryStep('c2', {
				iteration_path: 'c1[1]',
				input: { url: 'https://deux.test' },
				output: { numero: 'PASSAGE-B' },
			}),
			dryStep('c2', {
				iteration_path: 'c1[2]',
				input: { url: 'https://trois.test' },
				output: { numero: 'PASSAGE-C' },
			}),
		)

		function columns(dialog: HTMLElement) {
			return {
				input: within(dialog).getByRole('region', { name: 'Entrée' }),
				output: within(dialog).getByRole('region', { name: 'Sortie' }),
			}
		}

		it('shows the last pass first, and which one it is', async () => {
			const dialog = await openExecutionTab('c2', { dryRunSteps: PASSES })

			const { input, output } = columns(dialog)
			expect(input.textContent).toContain('https://trois.test')
			expect(output.textContent).toContain('PASSAGE-C')
			expect(within(input).getByText('Passage 3 sur 3')).toBeDefined()
			expect(within(output).getByText('Passage 3 sur 3')).toBeDefined()
		})

		it('steps back to an earlier pass in both columns, and forward again', async () => {
			const user = userEvent.setup()
			const dialog = await openExecutionTab('c2', { dryRunSteps: PASSES })

			await user.click(
				within(columns(dialog).input).getByRole('button', {
					name: 'Passage précédent',
				}),
			)

			expect(columns(dialog).input.textContent).toContain('https://deux.test')
			expect(columns(dialog).output.textContent).toContain('PASSAGE-B')
			expect(columns(dialog).output.textContent).not.toContain('PASSAGE-C')
			expect(
				within(columns(dialog).output).getByText('Passage 2 sur 3'),
			).toBeDefined()

			await user.click(
				within(columns(dialog).output).getByRole('button', {
					name: 'Passage précédent',
				}),
			)
			expect(columns(dialog).input.textContent).toContain('https://un.test')
			expect(
				(
					within(columns(dialog).output).getByRole('button', {
						name: 'Passage précédent',
					}) as HTMLButtonElement
				).disabled,
			).toBe(true)

			await user.click(
				within(columns(dialog).input).getByRole('button', {
					name: 'Passage suivant',
				}),
			)
			expect(columns(dialog).output.textContent).toContain('PASSAGE-B')
		})

		it('shows the status of the pass chosen', async () => {
			const user = userEvent.setup()
			const dialog = await openExecutionTab('c2', {
				dryRunSteps: stepsOf(
					dryStep('c2', { iteration_path: 'c1[0]', output: 1 }),
					dryStep('c2', {
						iteration_path: 'c1[1]',
						status: 'failed',
						error: 'boom',
					}),
				),
			})
			expect(within(dialog).getByText('Échec')).toBeDefined()

			await user.click(
				within(columns(dialog).output).getByRole('button', {
					name: 'Passage précédent',
				}),
			)

			expect(within(dialog).getByText('Succès')).toBeDefined()
		})

		it('offers no choice of pass to a node that ran once', async () => {
			const dialog = await openExecutionTab('c2', { dryRunSteps: FILLED })

			expect(within(dialog).queryByText(/Passage \d+ sur/)).toBeNull()
			expect(
				within(dialog).queryByRole('button', { name: 'Passage précédent' }),
			).toBeNull()
		})
	})
})

function placedCredential(
	id: string,
	name: string,
	kind: string,
	status: Schemas.CredentialStatusResponse = 'valid',
): Schemas.CredentialResponse {
	return credentialFixture({ id, name, kind, status })
}

const SCHEDULE_NODE = triggerNode(
	{
		id: 't1',
		kind: {
			Schedule: { frequency: 'Daily', time: '09:00', timezone: 'Europe/Paris' },
		},
	},
	AT,
	LOOKUP,
)

function webhookNode(
	rule: Partial<
		Extract<Schemas.TriggerKindDto, { Webhook: unknown }>['Webhook']
	> = {},
	errors: EditorNode['data']['errors'] = [],
): EditorNode {
	const node = triggerNode(
		{
			id: 't1',
			kind: {
				Webhook: {
					method: 'POST',
					path: 'paiement-recu',
					credential_id: null,
					contract: [
						{ name: 'facture', kind: 'Text', required: true, example: 'FA-1' },
						{
							name: 'montant',
							kind: 'Number',
							required: false,
							example: '2340',
						},
					],
					on_invalid: 'Reject',
					...rule,
				},
			},
		},
		AT,
		LOOKUP,
	)
	return { ...node, data: { ...node.data, errors } }
}

describe('NodeDetailsFeature — a scheduler', () => {
	it('reads the rule in the header, under the type badge', async () => {
		const dialog = await openNode('t1', [SCHEDULE_NODE], [])

		expect(within(dialog).getAllByText('Planificateur').length).toBeGreaterThan(
			0,
		)
		const described = dialog.getAttribute('aria-describedby')
		expect(document.getElementById(described ?? '')?.textContent).toBe(
			'tous les jours à 9:00',
		)
	})

	it('shows the fields of the schedule in the parameters column', async () => {
		const dialog = await openNode('t1', [SCHEDULE_NODE], [])

		const parameters = within(dialog).getByRole('region', {
			name: 'Paramètres',
		})
		expect(
			within(parameters).getByRole('combobox', { name: 'Fréquence' }),
		).toBeDefined()
		expect(
			within(parameters).getByLabelText('Heure de déclenchement'),
		).toBeDefined()
		expect(
			within(parameters).getByRole('combobox', { name: 'Fuseau horaire' }),
		).toBeDefined()
		expect(
			within(parameters).queryByRole('combobox', { name: 'Jour' }),
		).toBeNull()
	})

	it('explains in the input column what the workflow receives, key by key', async () => {
		const dialog = await openNode('t1', [SCHEDULE_NODE], [])

		const input = within(dialog).getByRole('region', { name: 'Entrée' })
		for (const key of ['scheduled_for', 'horodatage', 'jour', 'fuseau'])
			expect(within(input).getAllByText(key)).toHaveLength(2)
		expect(
			within(input).getByRole('tab', { name: 'Exemple de payload' }),
		).toBeDefined()
	})

	it('sends the new rule to the graph, and the weekday along with a weekly frequency', async () => {
		const user = userEvent.setup()
		const onUpdateTriggerKind = vi.fn()
		const dialog = await openNode('t1', [SCHEDULE_NODE], [], {
			onUpdateTriggerKind,
		})

		await user.click(
			within(dialog).getByRole('combobox', { name: 'Fréquence' }),
		)
		await user.click(
			await screen.findByRole('option', { name: 'Toutes les semaines' }),
		)

		expect(onUpdateTriggerKind).toHaveBeenCalledWith({
			Schedule: {
				frequency: 'Weekly',
				time: '09:00',
				timezone: 'Europe/Paris',
				weekday: 'Monday',
			},
		})
	})

	it('shows the issue of the client and the error of the server on the field they name', async () => {
		const node: EditorNode = {
			...SCHEDULE_NODE,
			data: {
				...SCHEDULE_NODE.data,
				errors: [
					{
						field: 'timezone',
						message: 'trigger `t1` has the unknown timezone `Mars/Olympus`',
					},
				],
			},
		}

		const dialog = await openNode('t1', [node], [], {
			issues: [
				{
					parameter: 'time',
					message: 'Indiquez une heure au format HH:MM, par exemple 09:00.',
				},
			],
		})

		const parameters = within(dialog).getByRole('region', {
			name: 'Paramètres',
		})
		expect(
			within(parameters)
				.getAllByRole('alert')
				.map((alert) => alert.textContent),
		).toEqual([
			'Indiquez une heure au format HH:MM, par exemple 09:00.',
			'Ce fuseau horaire n’existe pas.',
		])
	})
})

describe('NodeDetailsFeature — a webhook', () => {
	it('shows the request it expects: the method and the URL, the contract and an example', async () => {
		const user = userEvent.setup()
		const dialog = await openNode('t1', [webhookNode()], [])

		const input = within(dialog).getByRole('region', { name: 'Entrée' })
		expect(within(input).getByText('Requête attendue')).toBeDefined()
		expect(
			within(input).getByText(
				(_, element) =>
					element?.tagName === 'P' &&
					element.textContent ===
						`POST ${window.location.origin}/webhooks/paiement-recu`,
			),
		).toBeDefined()
		expect(within(input).getByText(/PaiementRecuBody/)).toBeDefined()
		expect(within(input).getByText(/montant\?/)).toBeDefined()

		await user.click(
			within(input).getByRole('tab', { name: 'Exemple de requête' }),
		)
		expect(within(input).getByText(/FA-1/)).toBeDefined()
		expect(within(input).getByText(/2340/)).toBeDefined()
	})

	it('follows the method, the path and the contract that are edited', async () => {
		const dialog = await openNode(
			't1',
			[
				webhookNode({
					method: 'PUT',
					path: 'stripe/retour',
					contract: [
						{ name: 'ref', kind: 'Boolean', required: true, example: '' },
					],
				}),
			],
			[],
		)

		const input = within(dialog).getByRole('region', { name: 'Entrée' })
		expect(
			within(input).getByText(
				(_, element) =>
					element?.tagName === 'P' &&
					element.textContent ===
						`PUT ${window.location.origin}/webhooks/stripe/retour`,
			),
		).toBeDefined()
		expect(within(input).getByText(/StripeRetourBody/)).toBeDefined()
		expect(input.textContent).toContain('ref: boolean')
	})

	it('shows the fields of the webhook in the parameters column', async () => {
		const dialog = await openNode('t1', [webhookNode()], [])

		const parameters = within(dialog).getByRole('region', {
			name: 'Paramètres',
		})
		expect(
			within(parameters).getByRole('combobox', { name: 'Méthode' }),
		).toBeDefined()
		expect(
			(
				within(parameters).getByPlaceholderText(
					'stripe/paiement-recu',
				) as HTMLInputElement
			).value,
		).toBe('paiement-recu')
		expect(
			within(parameters).getByRole('combobox', { name: 'Identification' }),
		).toBeDefined()
		expect(within(parameters).getByLabelText('Clé facture')).toBeDefined()
		expect(
			within(parameters).getByRole('combobox', {
				name: 'Requête non conforme au contrat',
			}),
		).toBeDefined()
	})

	it('offers only the identifications a webhook can use', async () => {
		const user = userEvent.setup()
		const dialog = await openNode('t1', [webhookNode()], [], {
			credentials: [
				placedCredential('c1', 'Clé Stripe', 'http_header'),
				placedCredential('c2', 'Compte basic', 'http_basic'),
				placedCredential('jeton', 'Jeton', 'bearer_token'),
				placedCredential('c4', 'Odoo prod', 'odoo_api'),
			],
		})

		await user.click(
			within(dialog).getByRole('combobox', { name: 'Identification' }),
		)

		expect(
			screen.getAllByRole('option').map((option) => option.textContent),
		).toEqual(['Aucune identification', 'Clé Stripe', 'Compte basic', 'Jeton'])
		expect(
			within(dialog).getByText(
				'Types acceptés : Basic Auth, Bearer, Clé d’API dans un en-tête, Signature HMAC.',
			),
		).toBeDefined()
	})

	it('flags the identification a webhook points at when it is expired', async () => {
		const dialog = await openNode(
			't1',
			[webhookNode({ credential_id: 'c1' })],
			[],
			{
				credentials: [
					placedCredential('c1', 'Clé Stripe', 'http_header', 'expired'),
				],
			},
		)

		expect(
			within(dialog).getByText(
				'Cette identification est expirée. Renouvelez-la depuis Identifications.',
			),
		).toBeDefined()
	})

	it('offers no identification to a webhook while the kinds are not known', async () => {
		const dialog = await openNode('t1', [webhookNode()], [], {
			credentials: [placedCredential('c1', 'Clé Stripe', 'http_header')],
			credentialKinds: [],
		})

		expect(
			within(dialog)
				.getByRole('combobox', { name: 'Identification' })
				.hasAttribute('disabled'),
		).toBe(true)
	})

	it('saves the chosen identification on the trigger', async () => {
		const user = userEvent.setup()
		const onUpdateTriggerKind = vi.fn()
		const dialog = await openNode('t1', [webhookNode()], [], {
			onUpdateTriggerKind,
			credentials: [placedCredential('c1', 'Clé Stripe', 'http_header')],
		})

		await user.click(
			within(dialog).getByRole('combobox', { name: 'Identification' }),
		)
		await user.click(await screen.findByRole('option', { name: 'Clé Stripe' }))

		expect(onUpdateTriggerKind).toHaveBeenCalledWith({
			Webhook: expect.objectContaining({
				credential_id: 'c1',
				path: 'paiement-recu',
			}),
		})
	})

	it('warns under the selector while no identification is chosen, and no longer once there is one', async () => {
		const warning =
			'Sans identification, toute personne qui connaît cette URL peut déclencher ce workflow.'
		const credentials = [placedCredential('c1', 'Clé Stripe', 'http_header')]
		const { unmount } = await renderOpened(webhookNode(), { credentials })
		expect(screen.getByText(warning)).toBeDefined()
		unmount()

		await renderOpened(webhookNode({ credential_id: 'c1' }), { credentials })
		expect(screen.queryByText(warning)).toBeNull()
	})

	it('shows the issues of the path next to the URL', async () => {
		const dialog = await openNode('t1', [webhookNode({ path: '' })], [], {
			issues: [
				{
					parameter: 'path',
					message: 'Indiquez la fin de l’URL après /webhooks/.',
				},
			],
		})

		const parameters = within(dialog).getByRole('region', {
			name: 'Paramètres',
		})
		expect(within(parameters).getByRole('alert').textContent).toBe(
			'Indiquez la fin de l’URL après /webhooks/.',
		)
	})

	it('puts what the server says on the field it names: the path, the identification, a key', async () => {
		const node = webhookNode({ credential_id: 'c9' }, [
			{
				field: 'path',
				message:
					'trigger `t1` uses a webhook path another workflow already holds',
			},
			{
				field: 'credential',
				message: 'trigger `t1` references unknown credential `c9`',
			},
			{
				field: 'contract.facture',
				message: 'trigger `t1` names the contract key `facture` more than once',
			},
			{ field: 'priority', message: 'quelque chose d’autre' },
		])

		const dialog = await openNode('t1', [node], [])

		const parameters = within(dialog).getByRole('region', {
			name: 'Paramètres',
		})
		const alerts = within(parameters)
			.getAllByRole('alert')
			.map((alert) => alert.textContent)
		expect(alerts).toContain('quelque chose d’autre')
		expect(alerts).toContain('Ce chemin est déjà utilisé.')
		expect(alerts).toContain('Cette identification n’existe plus.')
		expect(alerts).toContain('Clé en double à ce niveau.')
	})

	it('creates an identification of a type a webhook accepts, and selects it', async () => {
		const user = userEvent.setup()
		const onUpdateTriggerKind = vi.fn()
		const onCreateCredential = vi.fn().mockResolvedValue({
			...placedCredential('neuve', 'Clé du webhook', 'http_header'),
			secret: null,
		})
		const dialog = await openNode('t1', [webhookNode()], [], {
			onUpdateTriggerKind,
			onCreateCredential,
			credentialKinds: [HTTP_HEADER, ODOO_API],
		})

		await user.click(
			within(dialog).getByRole('button', { name: 'Créer une identification' }),
		)
		await user.click(within(dialog).getByRole('combobox', { name: 'Type' }))
		expect(
			screen.getAllByRole('option').map((option) => option.textContent),
		).toEqual(['Clé d’API dans un en-tête'])
		await user.click(screen.getByRole('option'))
		await user.type(within(dialog).getByLabelText('Nom'), 'Clé du webhook')
		await user.type(within(dialog).getByLabelText(/Valeur/), 'secret')
		await user.click(within(dialog).getByRole('button', { name: 'Créer' }))

		await waitFor(() => {
			expect(onUpdateTriggerKind).toHaveBeenCalledWith({
				Webhook: expect.objectContaining({ credential_id: 'neuve' }),
			})
		})
		expect(onCreateCredential).toHaveBeenCalledWith(
			expect.objectContaining({ kind: 'http_header', name: 'Clé du webhook' }),
		)
	})
})

async function renderOpened(node: EditorNode, handlers: OpenOptions = {}) {
	const view = await renderWithRouter(
		<TooltipProvider>
			<NodeDetailsFeature
				node={node}
				nodes={[node]}
				edges={[]}
				organizationSlug="atelier-bois"
				sources={SOURCES}
				collectionReads={new Map()}
				credentialKinds={handlers.credentialKinds ?? KINDS}
				credentials={handlers.credentials ?? []}
				issues={handlers.issues ?? []}
				dryRunSteps={null}
				testRun={{ running: false, start: vi.fn() }}
				onUpdateConnector={vi.fn()}
				onUpdateTriggerKind={handlers.onUpdateTriggerKind ?? vi.fn()}
				onUpdateDetails={vi.fn()}
				onCreateCredential={handlers.onCreateCredential ?? vi.fn()}
				onEvaluateExpression={vi.fn()}
				onClose={vi.fn()}
				onNavigate={vi.fn()}
			/>
		</TooltipProvider>,
	)
	await screen.findByRole('dialog')
	return view
}
