import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeAll, describe, expect, it, vi } from 'vitest'
import { installRadixPolyfills } from '#/pages/support/feature/support-test-kit'
import type { SupportIntegrationUIProps } from '#/pages/support/types'
import { SupportIntegrationUI } from '#/pages/support/ui/support-integration-ui'

beforeAll(installRadixPolyfills)

function renderIntegration(overrides: Partial<SupportIntegrationUIProps> = {}) {
	const props: SupportIntegrationUIProps = {
		origin: 'https://app.test',
		keys: [
			{
				id: 'key-1',
				name: 'Application partenaires',
				prefix: 'mst_live_a1b2',
				createdAt: '2026-10-02T09:30:00Z',
				lastUsedAt: null,
				revokedAt: null,
			},
			{
				id: 'key-2',
				name: 'Ancien site',
				prefix: 'mst_live_c3d4',
				createdAt: '2026-08-14T11:00:00Z',
				lastUsedAt: '2026-09-01T11:00:00Z',
				revokedAt: '2026-09-30T08:00:00Z',
			},
		],
		revealed: null,
		testResult: null,
		isSending: false,
		onCreateKey: vi.fn(),
		onDismissRevealed: vi.fn(),
		onRevokeKey: vi.fn(),
		onSendTest: vi.fn(),
		...overrides,
	}
	render(<SupportIntegrationUI {...props} />)
	return props
}

describe('SupportIntegrationUI — documentation', () => {
	it('titles the page and explains the integration', () => {
		renderIntegration()

		expect(screen.getByRole('heading', { level: 1 }).textContent).toBe(
			'Intégration API',
		)
		expect(
			screen.getByText(
				/Votre plateforme crée des tickets par une simple requête HTTP/,
			),
		).toBeTruthy()
	})

	it('shows the endpoint on the origin of the app', () => {
		renderIntegration()

		expect(screen.getByText('POST').parentElement?.textContent).toBe(
			'POSThttps://app.test/api/v1/support/tickets',
		)
	})

	it('builds the curl example with the prefix of the first live key', () => {
		renderIntegration()

		const example = screen.getByRole('figure', { name: 'Exemple de requête' })
		expect(example.textContent).toContain('Bearer mst_live_a1b2…')
	})

	it('falls back to a placeholder prefix without live key', () => {
		renderIntegration({ keys: [] })

		expect(
			screen.getByRole('figure', { name: 'Exemple de requête' }).textContent,
		).toContain('Bearer mst_live_xxxx…')
	})

	it('documents the payload fields and the response codes', () => {
		renderIntegration()

		expect(screen.getByText('requester.email')).toBeTruthy()
		expect(screen.getAllByText('obligatoire')).toHaveLength(3)
		expect(screen.getByText('201')).toBeTruthy()
		expect(screen.getByText('422')).toBeTruthy()
	})

	it('copies the example', async () => {
		const writeText = vi.fn().mockResolvedValue(undefined)
		Object.defineProperty(navigator, 'clipboard', {
			value: { writeText },
			configurable: true,
		})
		renderIntegration()

		await userEvent.click(screen.getByLabelText('Copier : Exemple de requête'))

		expect(writeText).toHaveBeenCalledTimes(1)
		expect(writeText.mock.calls[0][0]).toContain('curl -X POST')
	})
})

describe('SupportIntegrationUI — clés d’API', () => {
	function keysCard() {
		return screen.getByRole('region', { name: 'Clés d’API' })
	}

	it('lists the keys, a revoked one included', () => {
		renderIntegration()

		expect(within(keysCard()).getByText('Application partenaires')).toBeTruthy()
		expect(within(keysCard()).getByText('Ancien site')).toBeTruthy()
		expect(within(keysCard()).getByText('Jamais')).toBeTruthy()
		expect(within(keysCard()).getByText('Révoquée')).toBeTruthy()
	})

	it('creates a key from its name', async () => {
		const props = renderIntegration()

		await userEvent.click(screen.getByRole('button', { name: 'Nouvelle clé' }))
		const create = screen.getByRole('button', { name: 'Créer la clé' })
		expect((create as HTMLButtonElement).disabled).toBe(true)
		await userEvent.type(
			screen.getByLabelText('Nom de la plateforme'),
			'Site vitrine',
		)
		await userEvent.click(create)

		expect(props.onCreateKey).toHaveBeenCalledWith('Site vitrine')
		expect(screen.queryByLabelText('Nom de la plateforme')).toBeNull()
		expect(screen.getByRole('button', { name: 'Nouvelle clé' })).toBeTruthy()
	})

	it('cancels the creation of a key', async () => {
		const props = renderIntegration()

		await userEvent.click(screen.getByRole('button', { name: 'Nouvelle clé' }))
		await userEvent.click(screen.getByRole('button', { name: 'Annuler' }))

		expect(screen.queryByLabelText('Nom de la plateforme')).toBeNull()
		expect(props.onCreateKey).not.toHaveBeenCalled()
	})

	it('shows the secret of a new key once, and dismisses it', async () => {
		const props = renderIntegration({
			revealed: { name: 'Site vitrine', secret: 'mst_live_secret' },
		})

		expect(screen.getByText('Clé « Site vitrine » créée')).toBeTruthy()
		expect(
			screen.getByRole('figure', { name: 'Secret de la clé' }).textContent,
		).toBe('mst_live_secret')
		await userEvent.click(
			screen.getByRole('button', { name: 'J’ai copié la clé' }),
		)

		expect(props.onDismissRevealed).toHaveBeenCalledTimes(1)
	})

	it('asks for a confirmation before revoking a key', async () => {
		const props = renderIntegration()

		await userEvent.click(screen.getByRole('button', { name: 'Révoquer' }))
		expect(props.onRevokeKey).not.toHaveBeenCalled()
		expect(screen.getByText('Les appels échoueront.')).toBeTruthy()
		await userEvent.click(
			within(keysCard()).getAllByRole('button', { name: 'Révoquer' })[0],
		)

		expect(props.onRevokeKey).toHaveBeenCalledWith('key-1')
	})

	it('backs out of a revocation', async () => {
		const props = renderIntegration()

		await userEvent.click(screen.getByRole('button', { name: 'Révoquer' }))
		await userEvent.click(
			within(keysCard()).getByRole('button', { name: 'Annuler' }),
		)

		expect(props.onRevokeKey).not.toHaveBeenCalled()
		expect(screen.queryByText('Les appels échoueront.')).toBeNull()
	})
})

describe('SupportIntegrationUI — essayer', () => {
	it('asks for a key first when there is none alive', () => {
		renderIntegration({ keys: [] })

		expect(screen.getByText('Créez d’abord une clé d’API.')).toBeTruthy()
		expect(
			screen.queryByRole('button', { name: 'Envoyer la requête' }),
		).toBeNull()
	})

	it('sends the payload with the first live key', async () => {
		const props = renderIntegration()

		await userEvent.click(
			screen.getByRole('button', { name: 'Envoyer la requête' }),
		)

		expect(props.onSendTest).toHaveBeenCalledTimes(1)
		const [keyId, payload] = vi.mocked(props.onSendTest).mock.calls[0]
		expect(keyId).toBe('key-1')
		expect(JSON.parse(payload).subject).toBe('Volet roulant bloqué')
	})

	it('sends an edited payload', async () => {
		const props = renderIntegration()
		const payload = screen.getByLabelText('Corps de la requête')

		await userEvent.clear(payload)
		await userEvent.click(payload)
		await userEvent.paste('{"subject":"a"}')
		await userEvent.click(
			screen.getByRole('button', { name: 'Envoyer la requête' }),
		)

		expect(props.onSendTest).toHaveBeenCalledWith('key-1', '{"subject":"a"}')
	})

	it('locks the send button while sending', () => {
		renderIntegration({ isSending: true })

		expect(
			(
				screen.getByRole('button', {
					name: 'Envoyer la requête',
				}) as HTMLButtonElement
			).disabled,
		).toBe(true)
	})

	it('reports an invalid payload', () => {
		renderIntegration({
			testResult: { kind: 'invalid', errors: ['subject est obligatoire.'] },
		})

		expect(screen.getByRole('alert').textContent).toContain('422')
		expect(screen.getByRole('alert').textContent).toContain(
			'subject est obligatoire.',
		)
	})

	it('reports a created ticket', () => {
		renderIntegration({ testResult: { kind: 'created', ticketNumber: 101 } })

		expect(screen.getAllByRole('status')[0].textContent).toContain('201')
		expect(
			screen.getByText(/ticket #101 créé, statut « Backlog »/),
		).toBeTruthy()
	})

	it('reports a duplicate', () => {
		renderIntegration({ testResult: { kind: 'duplicate', ticketNumber: 101 } })

		expect(
			screen.getByText(/ce external_id existe déjà, ticket #101 renvoyé/),
		).toBeTruthy()
	})
})
