import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeAll, beforeEach, describe, expect, it } from 'vitest'
import { TrainingsPageFeature } from '#/pages/cursus/catalog/feature/trainings-page-feature'
import {
	callsTo,
	createQueryClient,
	type FakeApi,
	installFakeTanstackApi,
	installRadixPolyfills,
	Providers,
	page,
} from '#/pages/support/feature/support-test-kit'

const PATH = '/api/v1/organizations/{organization_id}/cursus/trainings'

let api: FakeApi

beforeAll(installRadixPolyfills)

beforeEach(() => {
	api = installFakeTanstackApi()
	api.mockGet(PATH, () =>
		page([
			{
				id: 't-1',
				organization_id: 'org-1',
				name: 'CAP Menuiserie',
				created_at: '2026-01-01T00:00:00Z',
				updated_at: '2026-01-01T00:00:00Z',
			},
		] as never),
	)
})

function renderPage(permissions: string[]) {
	render(
		<Providers queryClient={createQueryClient(permissions)}>
			<TrainingsPageFeature />
		</Providers>,
	)
}

describe('TrainingsPageFeature', () => {
	it('creates a training without an establishment field or id', {
		timeout: 15_000,
	}, async () => {
		const user = userEvent.setup()
		api.mockMutation('post', PATH, () => ({ data: {} }))
		renderPage(['VIEW_CURSUS', 'MANAGE_CURSUS_CATALOG'])

		await user.click(
			await screen.findByRole('button', { name: 'Nouvelle formation' }),
		)
		expect(screen.queryByText('Établissement')).toBeNull()
		await user.click(screen.getByLabelText('Nom'))
		await user.paste('CAP Menuiserie')
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		await waitFor(() => expect(callsTo(api, 'post', PATH)).toHaveLength(1))
		expect(callsTo(api, 'post', PATH)[0]?.params).toEqual({
			path: { organization_id: 'org-1' },
			body: {
				name: 'CAP Menuiserie',
				rncp_code: null,
				rncp_label: null,
				level: null,
				degree_type: null,
			},
		})
	})

	it('lists the formations and hides the create button without the manage bit', async () => {
		renderPage(['VIEW_CURSUS'])

		expect(await screen.findByText('CAP Menuiserie')).toBeTruthy()
		expect(screen.getByRole('heading', { name: 'Formations' })).toBeTruthy()
		expect(
			screen.queryByRole('button', { name: 'Nouvelle formation' }),
		).toBeNull()
		expect(
			screen.queryByRole('button', { name: 'Archiver CAP Menuiserie' }),
		).toBeNull()
	})

	it('offers create and archive actions with the manage bit', async () => {
		renderPage(['VIEW_CURSUS', 'MANAGE_CURSUS_CATALOG'])

		expect(
			await screen.findByRole('button', { name: 'Nouvelle formation' }),
		).toBeTruthy()
		expect(
			await screen.findByRole('button', { name: 'Archiver CAP Menuiserie' }),
		).toBeTruthy()
	})
})
