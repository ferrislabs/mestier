import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeAll, describe, expect, it } from 'vitest'
import { SupportIntegrationFeature } from '#/pages/support/feature/support-integration-feature'
import { installRadixPolyfills } from '#/pages/support/feature/support-test-kit'

beforeAll(installRadixPolyfills)

async function sendPayload(
	user: ReturnType<typeof userEvent.setup>,
	payload: string,
) {
	const field = screen.getByLabelText('Corps de la requête')
	await user.clear(field)
	await user.click(field)
	await user.paste(payload)
	await user.click(screen.getByRole('button', { name: 'Envoyer la requête' }))
}

describe('SupportIntegrationFeature', () => {
	it('lists the mocked keys', () => {
		render(<SupportIntegrationFeature />)

		expect(
			screen.getAllByText('Application partenaires').length,
		).toBeGreaterThan(0)
	})

	it('reveals the secret of a freshly created key', async () => {
		const user = userEvent.setup()
		render(<SupportIntegrationFeature />)

		await user.click(screen.getByRole('button', { name: 'Nouvelle clé' }))
		await user.type(
			screen.getByLabelText('Nom de la plateforme'),
			'Site vitrine',
		)
		await user.click(screen.getByRole('button', { name: 'Créer la clé' }))

		expect(await screen.findByText('Clé « Site vitrine » créée')).toBeTruthy()
		expect(
			screen.getByRole('figure', { name: 'Secret de la clé' }).textContent,
		).toMatch(/^mst_live_[A-Za-z0-9]{32}$/)

		await user.click(screen.getByRole('button', { name: 'J’ai copié la clé' }))
		expect(screen.queryByText('Clé « Site vitrine » créée')).toBeNull()
	})

	it('answers 422 for a payload that is not JSON', async () => {
		const user = userEvent.setup()
		render(<SupportIntegrationFeature />)

		await sendPayload(user, 'pas du json')

		expect((await screen.findByRole('alert')).textContent).toContain(
			'JSON invalide.',
		)
	})

	it('answers 422 with the missing fields', async () => {
		const user = userEvent.setup()
		render(<SupportIntegrationFeature />)

		await sendPayload(user, '{}')

		expect((await screen.findByRole('alert')).textContent).toContain(
			'subject est obligatoire.',
		)
	})

	it('answers 201 for a new external id, then 200 for the same one', async () => {
		const user = userEvent.setup()
		render(<SupportIntegrationFeature />)
		const payload = JSON.stringify({
			subject: 'Volet bloqué',
			body: 'Il ne remonte plus.',
			requester: { email: 'claire@fontaine.fr' },
			external_id: 'EC-FEATURE-1',
		})

		await sendPayload(user, payload)
		expect(await screen.findByText(/ticket #\d+ créé/)).toBeTruthy()

		await sendPayload(user, payload)
		await waitFor(() =>
			expect(screen.getByText(/ce external_id existe déjà/)).toBeTruthy(),
		)
	})
})
