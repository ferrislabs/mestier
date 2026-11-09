import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeAll, beforeEach, describe, expect, it } from 'vitest'
import { EstablishmentsPageFeature } from '#/pages/cursus/catalog/feature/establishments-page-feature'
import {
	callsTo,
	createQueryClient,
	type FakeApi,
	installFakeTanstackApi,
	installRadixPolyfills,
	Providers,
	page,
} from '#/pages/support/feature/support-test-kit'

const PATH = '/api/v1/organizations/{organization_id}/cursus/establishments'

let api: FakeApi

beforeAll(installRadixPolyfills)

beforeEach(() => {
	api = installFakeTanstackApi()
	api.mockGet(PATH, () =>
		page([
			{
				id: 'e-1',
				organization_id: 'org-1',
				name: 'CFA du Bâtiment',
				is_main_site: false,
				created_at: '2026-01-01T00:00:00Z',
				updated_at: '2026-01-01T00:00:00Z',
			},
		] as never),
	)
})

function renderPage(permissions: string[]) {
	render(
		<Providers queryClient={createQueryClient(permissions)}>
			<EstablishmentsPageFeature />
		</Providers>,
	)
}

describe('EstablishmentsPageFeature', () => {
	it('creates an establishment with the typed payload', {
		timeout: 15_000,
	}, async () => {
		const user = userEvent.setup()
		api.mockMutation('post', PATH, () => ({ data: {} }))
		renderPage(['VIEW_CURSUS', 'MANAGE_CURSUS_CATALOG'])

		await user.click(
			await screen.findByRole('button', { name: 'Nouvel établissement' }),
		)
		await user.click(screen.getByLabelText('Nom'))
		await user.paste('CFA du Bâtiment')
		await user.click(screen.getByLabelText('UAI'))
		await user.paste('0690001A')
		await user.click(screen.getByLabelText('Site principal'))
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		await waitFor(() => expect(callsTo(api, 'post', PATH)).toHaveLength(1))
		expect(callsTo(api, 'post', PATH)[0]?.params).toEqual({
			path: { organization_id: 'org-1' },
			body: {
				name: 'CFA du Bâtiment',
				uai: '0690001A',
				siret: null,
				address: null,
				is_main_site: true,
			},
		})
	})

	it('lists the établissements and hides the create button without the manage bit', async () => {
		renderPage(['VIEW_CURSUS'])

		expect(await screen.findByText('CFA du Bâtiment')).toBeTruthy()
		expect(screen.getByRole('heading', { name: 'Établissements' })).toBeTruthy()
		expect(
			screen.queryByRole('button', { name: 'Nouvel établissement' }),
		).toBeNull()
		expect(
			screen.queryByRole('button', { name: 'Archiver CFA du Bâtiment' }),
		).toBeNull()
	})

	it('offers create and archive actions with the manage bit', async () => {
		renderPage(['VIEW_CURSUS', 'MANAGE_CURSUS_CATALOG'])

		expect(
			await screen.findByRole('button', { name: 'Nouvel établissement' }),
		).toBeTruthy()
		expect(
			await screen.findByRole('button', { name: 'Archiver CFA du Bâtiment' }),
		).toBeTruthy()
	})
})
