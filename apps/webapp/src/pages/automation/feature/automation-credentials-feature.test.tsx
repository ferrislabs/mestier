import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import {
	AutomationCredentialsFeature,
	type AutomationCredentialsFeatureProps,
} from '#/pages/automation/feature/automation-credentials-feature'
import { credential, KINDS } from '#/pages/automation/test/credential-fixtures'
import {
	type FakeApi,
	installFakeTanstackApi,
} from '#/pages/automation/test/fake-tanstack-api'
import { seedPermissionsCacheForOrganization } from '#/test/with-permissions'

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

const KINDS_PATH =
	'/api/v1/organizations/{organization_id}/automation/credential-kinds'
const CREDENTIALS_PATH =
	'/api/v1/organizations/{organization_id}/automation/credentials'
const CREDENTIAL_PATH =
	'/api/v1/organizations/{organization_id}/automation/credentials/{credential_id}'
const ROTATE_PATH =
	'/api/v1/organizations/{organization_id}/automation/credentials/{credential_id}/rotate'
const OAUTH_START_PATH =
	'/api/v1/organizations/{organization_id}/automation/credentials/{credential_id}/oauth/start'
const OAUTH_DISCONNECT_PATH =
	'/api/v1/organizations/{organization_id}/automation/credentials/{credential_id}/oauth/disconnect'

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

const GENERATED = credential({
	id: 'sig-1',
	name: 'Signature sortante',
	kind: 'bearer_token',
	origin: 'generated',
	secrets_set: ['token'],
})

const GOOGLE = credential({
	id: 'google',
	name: 'Agenda Google',
	kind: 'oauth2_authorization_code',
	status: 'not_connected',
	data: {
		authorization_url: 'https://auth.exemple.fr/authorize',
		token_url: 'https://auth.exemple.fr/token',
		client_id: 'abc',
		pkce: 'yes',
	},
	secrets_set: ['client_secret'],
})

const STRIPE = credential({
	id: 'stripe',
	name: 'API tarifs',
	description: 'Clé du fournisseur de tarifs.',
	kind: 'http_header',
	data: { header_name: 'X-API-Key' },
	secrets_set: ['header_value'],
	used_by_workflows: 2,
})

function wireDefaults(api: FakeApi) {
	api.mockGet(KINDS_PATH, () => ({ data: KINDS, pagination: null }))
	api.mockGet(CREDENTIALS_PATH, () => ({ data: [], pagination: null }))
}

function renderFeature(
	configure: (api: FakeApi) => void,
	{
		permissions = ['VIEW_AUTOMATION', 'MANAGE_AUTOMATION'],
		props = {},
	}: {
		permissions?: string[]
		props?: AutomationCredentialsFeatureProps
	} = {},
) {
	const api = installFakeTanstackApi()
	wireDefaults(api)
	configure(api)

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

	function Page(pageProps: AutomationCredentialsFeatureProps) {
		return (
			<Providers>
				<AutomationCredentialsFeature {...pageProps} />
			</Providers>
		)
	}

	const view = render(<Page {...props} />)

	return Object.assign(api, {
		showOutcome: (nextProps: AutomationCredentialsFeatureProps) =>
			view.rerender(<Page {...nextProps} />),
	})
}

function callsTo(api: FakeApi, method: string, path: string) {
	return api.calls.filter(
		(call) => call.method === method && call.path === path,
	)
}

function bodyOf(call: { params: unknown } | undefined) {
	return (call?.params as { body: unknown } | undefined)?.body
}

function refusal(message: string, status: number, code?: string) {
	return Object.assign(new Error(message), {
		status,
		data: { code, message, status },
	})
}

async function openRow(user: ReturnType<typeof userEvent.setup>, name: string) {
	await user.click(await screen.findByRole('button', { name }))
	return screen.findByRole('dialog')
}

async function startCreating(
	user: ReturnType<typeof userEvent.setup>,
	typeName: RegExp,
) {
	await user.click(
		await screen.findByRole('button', { name: 'Nouvelle identification' }),
	)
	await user.click(await screen.findByRole('button', { name: typeName }))
}

describe('AutomationCredentialsFeature — the table', () => {
	it('shows the description, the status, the account and the usage the server reports', async () => {
		renderFeature((api) => {
			api.mockGet(CREDENTIALS_PATH, () => ({
				data: [
					STRIPE,
					credential({
						id: 'old',
						name: 'Ancien jeton',
						status: 'expired',
					}),
					GOOGLE,
					credential({
						id: 'connected',
						name: 'Agenda perso',
						kind: 'oauth2_authorization_code',
						account: 'a@exemple.fr',
					}),
				],
				pagination: null,
			}))
		})

		expect(await screen.findByText('Utilisée par 2 workflows')).toBeDefined()
		expect(screen.getByText('Clé du fournisseur de tarifs.')).toBeDefined()
		expect(screen.getByText('Expirée')).toBeDefined()
		expect(screen.getAllByText('Compte non connecté')).toHaveLength(2)
		expect(screen.getAllByText('Valide')).toHaveLength(2)
		expect(screen.getByText('Connecté : a@exemple.fr')).toBeDefined()
		expect(screen.getAllByText('Non utilisée')).toHaveLength(3)
		expect(screen.getByText('En-tête X-API-Key')).toBeDefined()
	})

	it('lists the most recently created identification first', async () => {
		renderFeature((api) => {
			api.mockGet(CREDENTIALS_PATH, () => ({
				data: [
					credential({
						id: 'old',
						name: 'Ancienne',
						created_at: '2026-01-01T00:00:00Z',
					}),
					credential({
						id: 'new',
						name: 'Récente',
						created_at: '2026-08-01T00:00:00Z',
					}),
					credential({
						id: 'mid',
						name: 'Intermédiaire',
						created_at: '2026-04-01T00:00:00Z',
					}),
				],
				pagination: null,
			}))
		})

		await screen.findByText('Récente')
		const names = screen
			.getAllByRole('row')
			.slice(1)
			.map((row) => within(row).getAllByRole('button')[0]?.textContent)

		expect(names).toEqual(['Récente', 'Intermédiaire', 'Ancienne'])
	})

	it('names the type as the server does, and never shows the technical code', async () => {
		renderFeature((api) => {
			api.mockGet(CREDENTIALS_PATH, () => ({
				data: [
					credential({ id: 'a', name: 'Un', kind: 'bearer_token' }),
					credential({ id: 'b', name: 'Deux', kind: 'http_header' }),
					credential({ id: 'c', name: 'Trois', kind: 'odoo_api' }),
				],
				pagination: null,
			}))
		})

		await screen.findByText('Un')
		expect(screen.getByText('Bearer')).toBeDefined()
		expect(screen.getByText('Clé d’API dans un en-tête')).toBeDefined()
		expect(screen.getByText('Clé d’API Odoo')).toBeDefined()
		expect(screen.queryByText('bearer_token')).toBeNull()
	})

	it('shows a loading state, not the codes, while the kinds are not there', async () => {
		renderFeature((api) => {
			api.mockGet(KINDS_PATH, () => new Promise(() => {}))
			api.mockGet(CREDENTIALS_PATH, () => ({
				data: [GENERATED],
				pagination: null,
			}))
		})

		expect(await screen.findByText('Chargement…')).toBeDefined()
		expect(screen.queryByText('bearer_token')).toBeNull()
		expect(screen.queryByText('Signature sortante')).toBeNull()
	})

	it('explains a failed list load in French', async () => {
		renderFeature((api) => {
			api.mockGet(CREDENTIALS_PATH, () => {
				throw refusal('HTTP 500: Internal Server Error', 500)
			})
		})

		expect(
			await screen.findByText(
				'Une erreur est survenue. Réessayez dans un instant.',
			),
		).toBeDefined()
	})

	it('hides every mutation without MANAGE_AUTOMATION', async () => {
		renderFeature(
			(api) => {
				api.mockGet(CREDENTIALS_PATH, () => ({
					data: [GENERATED],
					pagination: null,
				}))
			},
			{ permissions: ['VIEW_AUTOMATION'] },
		)

		await screen.findByText('Signature sortante')
		expect(
			screen.queryByRole('button', { name: 'Nouvelle identification' }),
		).toBeNull()
		expect(
			screen.queryByRole('button', { name: 'Signature sortante' }),
		).toBeNull()
	})
})

describe('AutomationCredentialsFeature — picking a type', () => {
	it('groups the types by the categories of the server and finds one by what it does', async () => {
		const user = userEvent.setup()
		renderFeature(() => {})

		await user.click(
			await screen.findByRole('button', { name: 'Nouvelle identification' }),
		)

		expect(screen.getByRole('region', { name: 'Requêtes HTTP' })).toBeDefined()
		expect(
			screen.getByRole('region', { name: 'Webhooks entrants' }),
		).toBeDefined()
		expect(screen.getByRole('region', { name: 'E-mail' })).toBeDefined()
		expect(
			screen.getByRole('region', { name: 'Serveurs et bases de données' }),
		).toBeDefined()
		expect(screen.getByRole('region', { name: 'Autres' })).toBeDefined()

		const search = screen.getByRole('searchbox', {
			name: 'Rechercher un type d’identification',
		})
		await user.type(search, 'mutuelle')
		expect(
			screen.getByRole('button', { name: /Certificat client/ }),
		).toBeDefined()
		expect(screen.queryByRole('button', { name: /Bearer/ })).toBeNull()

		await user.clear(search)
		await user.type(search, 'http_header')
		expect(screen.getByText(/Aucun type ne correspond/)).toBeDefined()
	})

	it('shows a loading state, not an empty picker, before the kinds are there', async () => {
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockGet(KINDS_PATH, () => new Promise(() => {}))
		})

		await user.click(
			await screen.findByRole('button', { name: 'Nouvelle identification' }),
		)

		const dialog = await screen.findByRole('dialog')
		expect(within(dialog).getByText('Chargement…')).toBeDefined()
		expect(within(dialog).queryByText(/Aucun type ne correspond/)).toBeNull()
	})
})

describe('AutomationCredentialsFeature — creation', () => {
	it('builds the form from the descriptor: defaults, placeholders, labels, hints', async () => {
		const user = userEvent.setup()
		renderFeature(() => {})

		await startCreating(user, /Clé d’API dans un en-tête/)

		expect(
			(screen.getByLabelText('Nom de l’en-tête') as HTMLInputElement).value,
		).toBe('X-API-Key')
		expect(screen.getByLabelText('Valeur')).toHaveProperty('type', 'password')
		expect(screen.getByLabelText('Nom')).toHaveProperty(
			'placeholder',
			'Ex. Clé d’API dans un en-tête – fournisseur',
		)
	})

	it('sends the name, the description and the data of the kind, defaults included', async () => {
		const user = userEvent.setup()
		const api = renderFeature((fake) => {
			fake.mockMutation('post', CREDENTIALS_PATH, () => ({
				data: { ...STRIPE, secret: null },
			}))
		})

		await startCreating(user, /Clé d’API dans un en-tête/)
		await user.type(screen.getByLabelText('Nom'), '  API tarifs ')
		await user.type(screen.getByLabelText(/Description/), 'Pour les devis')
		await user.type(screen.getByLabelText('Valeur'), 'sk-123')
		await user.click(
			screen.getByRole('button', { name: 'Créer l’identification' }),
		)

		await waitFor(() => {
			expect(callsTo(api, 'post', CREDENTIALS_PATH)).toHaveLength(1)
		})
		expect(bodyOf(callsTo(api, 'post', CREDENTIALS_PATH)[0])).toEqual({
			kind: 'http_header',
			name: 'API tarifs',
			description: 'Pour les devis',
			origin: 'supplied',
			data: { header_name: 'X-API-Key', header_value: 'sk-123' },
		})
		await waitFor(() => {
			expect(screen.queryByRole('dialog')).toBeNull()
		})
		expect(screen.queryByText('Secret généré')).toBeNull()
	})

	it('hides the fields a choice does not call for and sends only the visible ones', async () => {
		const user = userEvent.setup()
		const api = renderFeature((fake) => {
			fake.mockMutation('post', CREDENTIALS_PATH, () => ({
				data: { ...STRIPE, secret: null },
			}))
		})

		await startCreating(user, /^SSH/)
		expect(screen.getByLabelText(/Clé privée/)).toBeDefined()
		await user.click(screen.getByRole('combobox', { name: 'Méthode' }))
		await user.click(
			await screen.findByRole('option', { name: 'Mot de passe' }),
		)
		expect(screen.queryByLabelText(/Clé privée/)).toBeNull()

		await user.type(screen.getByLabelText('Nom'), 'Serveur')
		await user.type(screen.getByLabelText('Hôte'), 'srv.exemple.fr')
		await user.type(screen.getByLabelText('Utilisateur'), 'deploy')
		await user.type(screen.getByLabelText('Mot de passe'), 'pw')
		await user.click(
			screen.getByRole('button', { name: 'Créer l’identification' }),
		)

		await waitFor(() => {
			expect(callsTo(api, 'post', CREDENTIALS_PATH)).toHaveLength(1)
		})
		expect(bodyOf(callsTo(api, 'post', CREDENTIALS_PATH)[0])).toMatchObject({
			kind: 'ssh',
			data: {
				host: 'srv.exemple.fr',
				port: 22,
				username: 'deploy',
				method: 'password',
				password: 'pw',
			},
		})
	})

	it('requires every field of the descriptor before posting', async () => {
		const user = userEvent.setup()
		const api = renderFeature(() => {})

		await startCreating(user, /^Bearer/)
		await user.type(screen.getByLabelText('Nom'), 'Sans token')
		await user.click(
			screen.getByRole('button', { name: 'Créer l’identification' }),
		)

		expect(screen.getAllByText('Champ obligatoire.').length).toBeGreaterThan(0)
		expect(callsTo(api, 'post', CREDENTIALS_PATH)).toHaveLength(0)
	})

	it('reveals a generated secret exactly once after creation', async () => {
		const user = userEvent.setup()
		const api = renderFeature((fake) => {
			fake.mockMutation('post', CREDENTIALS_PATH, () => ({
				data: { ...GENERATED, secret: 'c2VjcmV0LWJ5dGVz' },
			}))
		})

		await startCreating(user, /^Bearer/)
		await user.type(screen.getByLabelText('Nom'), 'Signature sortante')
		await user.click(screen.getByRole('combobox', { name: 'Origine' }))
		await user.click(
			screen.getByRole('option', { name: /Générée par Mestier/ }),
		)
		await user.click(
			screen.getByRole('button', { name: 'Créer l’identification' }),
		)

		expect(await screen.findByText('Secret généré')).toBeDefined()
		expect(screen.getByDisplayValue('c2VjcmV0LWJ5dGVz')).toBeDefined()
		expect(bodyOf(callsTo(api, 'post', CREDENTIALS_PATH)[0])).toEqual({
			kind: 'bearer_token',
			name: 'Signature sortante',
			description: '',
			origin: 'generated',
			data: undefined,
		})
	})

	it('keeps the dialog open on the new account-based identification so that it can be connected', async () => {
		const user = userEvent.setup()
		renderFeature((fake) => {
			fake.mockMutation('post', CREDENTIALS_PATH, () => ({
				data: { ...GOOGLE, secret: null },
			}))
		})

		await startCreating(user, /^OAuth2/)
		await user.type(screen.getByLabelText('Nom'), 'Agenda Google')
		await user.type(
			screen.getByLabelText('URL d’autorisation'),
			'https://auth.exemple.fr/authorize',
		)
		await user.type(
			screen.getByLabelText('URL du jeton'),
			'https://auth.exemple.fr/token',
		)
		await user.type(screen.getByLabelText('Client ID'), 'abc')
		await user.type(screen.getByLabelText('Client secret'), 'shh')
		await user.click(
			screen.getByRole('button', { name: 'Créer l’identification' }),
		)

		const section = await screen.findByRole('region', {
			name: 'Compte autorisé',
		})
		expect(
			within(section).getByRole('button', { name: 'Connecter un compte' }),
		).toBeDefined()
		expect(screen.getByRole('dialog')).toBeDefined()
	})

	it('puts a name already taken under the name, in French', async () => {
		const user = userEvent.setup()
		renderFeature((fake) => {
			fake.mockMutation('post', CREDENTIALS_PATH, () => {
				throw refusal(
					'Conflict: Une identification de ce type porte déjà ce nom.',
					409,
					'E_CONFLICT',
				)
			})
		})

		await startCreating(user, /^Bearer/)
		await user.type(screen.getByLabelText('Nom'), 'Doublon')
		await user.type(screen.getByLabelText('Jeton'), 'shh')
		await user.click(
			screen.getByRole('button', { name: 'Créer l’identification' }),
		)

		expect(
			await screen.findByText(
				'Une identification de ce type porte déjà ce nom.',
			),
		).toBeDefined()
		expect(screen.getByLabelText('Nom').getAttribute('aria-invalid')).toBe(
			'true',
		)
		expect(screen.queryByText(/Conflict/)).toBeNull()
	})

	it('puts the refusal that names a field under that field and lets the user retry', async () => {
		const user = userEvent.setup()
		const api = renderFeature((fake) => {
			fake.mockMutation('post', CREDENTIALS_PATH, () => {
				throw refusal(
					'Validation failed: Le champ « Jeton » ne peut pas valoir null.',
					400,
					'E_VALIDATION',
				)
			})
		})

		await startCreating(user, /^Bearer/)
		await user.type(screen.getByLabelText('Nom'), 'X')
		await user.type(screen.getByLabelText('Jeton'), 'shh')
		await user.click(
			screen.getByRole('button', { name: 'Créer l’identification' }),
		)

		expect(
			await screen.findByText('Le champ « Jeton » ne peut pas valoir null.'),
		).toBeDefined()
		expect(screen.getByLabelText('Jeton').getAttribute('aria-invalid')).toBe(
			'true',
		)

		await user.click(
			screen.getByRole('button', { name: 'Créer l’identification' }),
		)
		await waitFor(() => {
			expect(callsTo(api, 'post', CREDENTIALS_PATH)).toHaveLength(2)
		})
	})
})

describe('AutomationCredentialsFeature — editing', () => {
	it('prefills the readable values and shows the secrets that are set as stored', async () => {
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockGet(CREDENTIALS_PATH, () => ({
				data: [STRIPE],
				pagination: null,
			}))
		})

		const dialog = await openRow(user, 'API tarifs')

		expect(
			(within(dialog).getByLabelText('Nom de l’en-tête') as HTMLInputElement)
				.value,
		).toBe('X-API-Key')
		expect(
			(within(dialog).getByLabelText(/Description/) as HTMLTextAreaElement)
				.value,
		).toBe('Clé du fournisseur de tarifs.')
		expect(
			within(dialog).getByText('Enregistré et chiffré — non affichable'),
		).toBeDefined()
		expect(
			within(dialog).getByText(
				'Modifiez son nom, sa description ou ses paramètres de connexion.',
			),
		).toBeDefined()
		expect(within(dialog).queryByText(/Laissez les champs vides/)).toBeNull()
	})

	it('sends only the secret that was replaced, not the others', async () => {
		const user = userEvent.setup()
		const api = renderFeature((fake) => {
			fake.mockGet(CREDENTIALS_PATH, () => ({
				data: [GOOGLE],
				pagination: null,
			}))
			fake.mockMutation('patch', CREDENTIAL_PATH, () => ({ data: GOOGLE }))
		})

		const dialog = await openRow(user, 'Agenda Google')
		await user.click(within(dialog).getByRole('button', { name: 'Remplacer' }))
		await user.type(within(dialog).getByLabelText('Client secret'), 'nouveau')
		await user.click(
			within(dialog).getByRole('button', { name: 'Enregistrer' }),
		)

		await waitFor(() => {
			expect(callsTo(api, 'patch', CREDENTIAL_PATH)).toHaveLength(1)
		})
		expect(bodyOf(callsTo(api, 'patch', CREDENTIAL_PATH)[0])).toEqual({
			name: 'Agenda Google',
			description: '',
			data: { client_secret: 'nouveau' },
		})
		await waitFor(() => {
			expect(screen.queryByRole('dialog')).toBeNull()
		})
	})

	it('sends only the readable field that changed, and no data at all when nothing changed', async () => {
		const user = userEvent.setup()
		const api = renderFeature((fake) => {
			fake.mockGet(CREDENTIALS_PATH, () => ({
				data: [GOOGLE],
				pagination: null,
			}))
			fake.mockMutation('patch', CREDENTIAL_PATH, () => ({ data: GOOGLE }))
		})

		let dialog = await openRow(user, 'Agenda Google')
		await user.click(
			within(dialog).getByRole('button', { name: 'Enregistrer' }),
		)
		await waitFor(() => {
			expect(callsTo(api, 'patch', CREDENTIAL_PATH)).toHaveLength(1)
		})
		expect(bodyOf(callsTo(api, 'patch', CREDENTIAL_PATH)[0])).toEqual({
			name: 'Agenda Google',
			description: '',
			data: undefined,
		})

		await waitFor(() => {
			expect(screen.queryByRole('dialog')).toBeNull()
		})
		dialog = await openRow(user, 'Agenda Google')
		const clientId = within(dialog).getByLabelText('Client ID')
		await user.clear(clientId)
		await user.type(clientId, 'xyz')
		await user.click(
			within(dialog).getByRole('button', { name: 'Enregistrer' }),
		)

		await waitFor(() => {
			expect(callsTo(api, 'patch', CREDENTIAL_PATH)).toHaveLength(2)
		})
		expect(bodyOf(callsTo(api, 'patch', CREDENTIAL_PATH)[1])).toEqual({
			name: 'Agenda Google',
			description: '',
			data: { client_id: 'xyz' },
		})
	})

	it('shows under the secret the server asks to type again when an address moved', async () => {
		const user = userEvent.setup()
		renderFeature((fake) => {
			fake.mockGet(CREDENTIALS_PATH, () => ({
				data: [GOOGLE],
				pagination: null,
			}))
			fake.mockMutation('patch', CREDENTIAL_PATH, () => {
				throw refusal(
					'Validation failed: Modifier cette adresse exige de saisir de nouveau « Client secret ».',
					400,
					'E_VALIDATION',
				)
			})
		})

		const dialog = await openRow(user, 'Agenda Google')
		const tokenUrl = within(dialog).getByLabelText('URL du jeton')
		await user.clear(tokenUrl)
		await user.type(tokenUrl, 'https://autre.exemple.fr/token')
		await user.click(
			within(dialog).getByRole('button', { name: 'Enregistrer' }),
		)

		expect(
			await within(dialog).findByText(
				'Modifier cette adresse exige de saisir de nouveau « Client secret ».',
			),
		).toBeDefined()
		expect(screen.getByRole('dialog')).toBeDefined()
	})

	it('words the edit subtitle like the prototype', async () => {
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockGet(CREDENTIALS_PATH, () => ({
				data: [GENERATED],
				pagination: null,
			}))
		})

		const dialog = await openRow(user, 'Signature sortante')

		expect(
			within(dialog).getByText(
				'Modifiez son nom, sa description ou ses paramètres de connexion.',
			),
		).toBeDefined()
	})
})

describe('AutomationCredentialsFeature — rotation', () => {
	it('reveals the freshly rotated secret exactly once', async () => {
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockGet(CREDENTIALS_PATH, () => ({
				data: [GENERATED],
				pagination: null,
			}))
			api.mockMutation('post', ROTATE_PATH, () => ({
				data: { ...GENERATED, secret: 'cm90YXRlZC1zZWNyZXQ' },
			}))
		})

		const dialog = await openRow(user, 'Signature sortante')
		await user.click(
			await within(dialog).findByRole('button', {
				name: 'Régénérer le secret',
			}),
		)

		expect(await screen.findByText('Secret généré')).toBeDefined()
		expect(screen.getByDisplayValue('cm90YXRlZC1zZWNyZXQ')).toBeDefined()
	})

	it('offers no rotation on a supplied credential', async () => {
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockGet(CREDENTIALS_PATH, () => ({
				data: [credential({ ...GENERATED, origin: 'supplied' })],
				pagination: null,
			}))
		})

		await openRow(user, 'Signature sortante')

		expect(
			screen.queryByRole('button', { name: 'Régénérer le secret' }),
		).toBeNull()
	})
})

describe('AutomationCredentialsFeature — deletion', () => {
	it('warns about the workflows that will fail, with their number, and deletes when confirmed', async () => {
		const user = userEvent.setup()
		const api = renderFeature((fake) => {
			fake.mockGet(CREDENTIALS_PATH, () => ({
				data: [STRIPE],
				pagination: null,
			}))
			fake.mockMutation('delete', CREDENTIAL_PATH, () => ({}))
		})

		await openRow(user, 'API tarifs')
		await user.click(await screen.findByRole('button', { name: 'Supprimer' }))

		expect(
			screen.getByText(/Utilisée par 2 workflows : leurs nœuds échoueront/),
		).toBeDefined()

		await user.click(
			screen.getByRole('button', { name: 'Supprimer définitivement' }),
		)

		await waitFor(() => {
			expect(callsTo(api, 'delete', CREDENTIAL_PATH)).toHaveLength(1)
		})
		expect(
			(callsTo(api, 'delete', CREDENTIAL_PATH)[0]?.params as { path: unknown })
				.path,
		).toEqual({ organization_id: 'org-1', credential_id: 'stripe' })
		await waitFor(() => {
			expect(screen.queryByRole('dialog')).toBeNull()
		})
	})

	it('says it is final when no workflow uses the identification', async () => {
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockGet(CREDENTIALS_PATH, () => ({
				data: [GENERATED],
				pagination: null,
			}))
		})

		await openRow(user, 'Signature sortante')
		await user.click(await screen.findByRole('button', { name: 'Supprimer' }))

		expect(screen.getByText('Cette action est définitive.')).toBeDefined()
	})

	it('keeps the dialog and the confirmation open when the deletion fails', async () => {
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockGet(CREDENTIALS_PATH, () => ({
				data: [STRIPE],
				pagination: null,
			}))
			api.mockMutation('delete', CREDENTIAL_PATH, () => {
				throw refusal('Resource not found', 404)
			})
		})

		await openRow(user, 'API tarifs')
		await user.click(await screen.findByRole('button', { name: 'Supprimer' }))
		await user.click(
			screen.getByRole('button', { name: 'Supprimer définitivement' }),
		)

		expect(
			await screen.findByText('Cette identification n’existe plus.'),
		).toBeDefined()
		expect(screen.getByRole('dialog')).toBeDefined()
		expect(
			screen.getByRole('button', { name: 'Supprimer définitivement' }),
		).toBeDefined()
	})

	it('does not show a previous refusal when the dialog is opened again', async () => {
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockGet(CREDENTIALS_PATH, () => ({
				data: [STRIPE],
				pagination: null,
			}))
			api.mockMutation('delete', CREDENTIAL_PATH, () => {
				throw refusal('Resource not found', 404)
			})
		})

		await openRow(user, 'API tarifs')
		await user.click(await screen.findByRole('button', { name: 'Supprimer' }))
		await user.click(
			screen.getByRole('button', { name: 'Supprimer définitivement' }),
		)
		await screen.findByText('Cette identification n’existe plus.')

		await user.keyboard('{Escape}')
		await waitFor(() => {
			expect(screen.queryByRole('dialog')).toBeNull()
		})
		await openRow(user, 'API tarifs')

		expect(screen.queryByText('Cette identification n’existe plus.')).toBeNull()
	})
})

describe('AutomationCredentialsFeature — connecting an account', () => {
	const originalApiUrl = window.apiUrl

	afterEach(() => {
		window.apiUrl = originalApiUrl
	})

	it('starts the authorization with the callback of the API and the way back, then sends the browser to the provider', async () => {
		window.apiUrl = 'https://api.mestier.test'
		const user = userEvent.setup()
		const navigateTo = vi.fn()
		const api = renderFeature(
			(fake) => {
				fake.mockGet(CREDENTIALS_PATH, () => ({
					data: [GOOGLE],
					pagination: null,
				}))
				fake.mockMutation('post', OAUTH_START_PATH, () => ({
					data: {
						authorization_url: 'https://auth.exemple.fr/authorize?state=abc',
					},
				}))
			},
			{ props: { navigateTo } },
		)

		const dialog = await openRow(user, 'Agenda Google')
		await user.click(
			within(dialog).getByRole('button', { name: 'Connecter un compte' }),
		)

		await waitFor(() => {
			expect(navigateTo).toHaveBeenCalledWith(
				'https://auth.exemple.fr/authorize?state=abc',
			)
		})
		const call = callsTo(api, 'post', OAUTH_START_PATH)[0]
		expect(call?.params).toEqual({
			path: { organization_id: 'org-1', credential_id: 'google' },
			body: {
				redirect_uri: 'https://api.mestier.test/oauth/callback',
				return_to: `${window.location.origin}/o/atelier-bois/automatisation/identifications`,
			},
		})
	})

	it('asks the way back as a local path when the API answers on the origin of the page', async () => {
		window.apiUrl = ''
		const user = userEvent.setup()
		const api = renderFeature(
			(fake) => {
				fake.mockGet(CREDENTIALS_PATH, () => ({
					data: [GOOGLE],
					pagination: null,
				}))
				fake.mockMutation('post', OAUTH_START_PATH, () => ({
					data: { authorization_url: 'https://auth.exemple.fr/authorize' },
				}))
			},
			{ props: { navigateTo: vi.fn() } },
		)

		const dialog = await openRow(user, 'Agenda Google')
		await user.click(
			within(dialog).getByRole('button', { name: 'Connecter un compte' }),
		)

		await waitFor(() => {
			expect(callsTo(api, 'post', OAUTH_START_PATH)).toHaveLength(1)
		})
		expect(bodyOf(callsTo(api, 'post', OAUTH_START_PATH)[0])).toEqual({
			redirect_uri: `${window.location.origin}/oauth/callback`,
			return_to: '/o/atelier-bois/automatisation/identifications',
		})
	})

	it('says so, and does not leave the page, when the server cannot connect accounts', async () => {
		const user = userEvent.setup()
		const navigateTo = vi.fn()
		renderFeature(
			(fake) => {
				fake.mockGet(CREDENTIALS_PATH, () => ({
					data: [GOOGLE],
					pagination: null,
				}))
				fake.mockMutation('post', OAUTH_START_PATH, () => {
					throw refusal(
						'La connexion de comptes n’est pas configurée sur ce serveur : l’administrateur doit renseigner l’adresse publique de l’API.',
						409,
						'E_OAUTH_NOT_CONFIGURED',
					)
				})
			},
			{ props: { navigateTo } },
		)

		const dialog = await openRow(user, 'Agenda Google')
		await user.click(
			within(dialog).getByRole('button', { name: 'Connecter un compte' }),
		)

		expect(
			await screen.findByText(/n’est pas configurée sur ce serveur/),
		).toBeDefined()
		expect(navigateTo).not.toHaveBeenCalled()
	})

	it('disconnects a connected account and shows the credential as not connected again', async () => {
		const user = userEvent.setup()
		let connected = true
		const api = renderFeature((fake) => {
			fake.mockGet(CREDENTIALS_PATH, () => ({
				data: [
					connected
						? { ...GOOGLE, status: 'valid', account: 'a@exemple.fr' }
						: GOOGLE,
				],
				pagination: null,
			}))
			fake.mockMutation('post', OAUTH_DISCONNECT_PATH, () => {
				connected = false
				return { data: GOOGLE }
			})
		})

		const dialog = await openRow(user, 'Agenda Google')
		await user.click(
			within(dialog).getByRole('button', { name: 'Déconnecter' }),
		)

		await waitFor(() => {
			expect(callsTo(api, 'post', OAUTH_DISCONNECT_PATH)).toHaveLength(1)
		})
		const section = await within(dialog).findByRole('region', {
			name: 'Compte autorisé',
		})
		await waitFor(() => {
			expect(
				within(section).getByRole('button', { name: 'Connecter un compte' }),
			).toBeDefined()
		})
		expect(
			within(section).queryByRole('button', { name: 'Déconnecter' }),
		).toBeNull()
	})
})

describe('AutomationCredentialsFeature — the way back from the provider', () => {
	it('welcomes a connected account, refreshes the list and clears the address', async () => {
		const onOauthOutcomeShown = vi.fn()
		const api = renderFeature((fake) => {
			fake.mockGet(CREDENTIALS_PATH, () => ({
				data: [GOOGLE],
				pagination: null,
			}))
		})
		await screen.findByText('Agenda Google')
		expect(callsTo(api, 'get', CREDENTIALS_PATH)).toHaveLength(1)

		api.showOutcome({ oauthOutcome: 'ok', onOauthOutcomeShown })

		expect(
			await screen.findByText(
				'Compte connecté. L’identification est prête à être utilisée.',
			),
		).toBeDefined()
		expect(onOauthOutcomeShown).toHaveBeenCalledOnce()
		await waitFor(() => {
			expect(callsTo(api, 'get', CREDENTIALS_PATH)).toHaveLength(2)
		})
	})

	it('shows the message at once when the page opens on the outcome', async () => {
		const onOauthOutcomeShown = vi.fn()
		renderFeature(() => {}, {
			props: { oauthOutcome: 'ok', onOauthOutcomeShown },
		})

		expect(await screen.findByText(/Compte connecté\./)).toBeDefined()
		expect(onOauthOutcomeShown).toHaveBeenCalledOnce()
	})

	it('tells a failed connection to be retried', async () => {
		renderFeature(() => {}, { props: { oauthOutcome: 'error' } })

		expect(await screen.findByRole('alert')).toHaveProperty(
			'textContent',
			expect.stringContaining('La connexion du compte a échoué'),
		)
	})

	it('lets the message be dismissed', async () => {
		const user = userEvent.setup()
		renderFeature(() => {}, { props: { oauthOutcome: 'ok' } })

		await screen.findByText(/Compte connecté/)
		await user.click(screen.getByRole('button', { name: 'Fermer le message' }))

		expect(screen.queryByText(/Compte connecté/)).toBeNull()
	})

	it('shows no message without an outcome', async () => {
		renderFeature(() => {})

		await screen.findByRole('button', { name: 'Nouvelle identification' })
		expect(screen.queryByText(/Compte connecté/)).toBeNull()
		expect(screen.queryByText(/a échoué/)).toBeNull()
	})
})
