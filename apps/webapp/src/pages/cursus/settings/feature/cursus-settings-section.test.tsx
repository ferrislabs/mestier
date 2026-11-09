import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeAll, beforeEach, describe, expect, it } from 'vitest'
import { CursusSettingsSection } from '#/pages/cursus/settings/cursus-settings-section'
import {
	callsTo,
	createQueryClient,
	type FakeApi,
	installFakeTanstackApi,
	installRadixPolyfills,
	Providers,
	page,
} from '#/pages/support/feature/support-test-kit'

const BASE = '/api/v1/organizations/{organization_id}/cursus'
const ESTABLISHMENTS = `${BASE}/establishments`
const TRAININGS = `${BASE}/trainings`
const CLOSURE_REASONS = `${BASE}/closure-reasons`

let api: FakeApi

beforeAll(installRadixPolyfills)

beforeEach(() => {
	api = installFakeTanstackApi()
	api.mockGet(ESTABLISHMENTS, () => page([] as never))
	api.mockGet(TRAININGS, () => page([] as never))
	api.mockGet(CLOSURE_REASONS, () => page([] as never))
})

function renderSection(permissions: string[]) {
	render(
		<Providers queryClient={createQueryClient(permissions)}>
			<CursusSettingsSection />
		</Providers>,
	)
}

describe('CursusSettingsSection', () => {
	it('creates an establishment with the typed payload', {
		timeout: 15_000,
	}, async () => {
		const user = userEvent.setup()
		api.mockMutation('post', ESTABLISHMENTS, () => ({ data: {} }))
		renderSection(['VIEW_CURSUS', 'MANAGE_CURSUS_CATALOG'])

		await user.click(
			await screen.findByRole('button', { name: 'Nouvel établissement' }),
		)
		await user.click(screen.getByLabelText('Nom'))
		await user.paste('CFA du Bâtiment')
		await user.click(screen.getByLabelText('UAI'))
		await user.paste('0690001A')
		await user.click(screen.getByLabelText('Site principal'))
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		await waitFor(() =>
			expect(callsTo(api, 'post', ESTABLISHMENTS)).toHaveLength(1),
		)
		expect(callsTo(api, 'post', ESTABLISHMENTS)[0]?.params).toEqual({
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

	it('creates a closure reason', async () => {
		const user = userEvent.setup()
		api.mockMutation('post', CLOSURE_REASONS, () => ({ data: {} }))
		renderSection(['VIEW_CURSUS', 'MANAGE_CURSUS_CATALOG'])

		await user.click(
			await screen.findByRole('button', { name: 'Nouveau motif' }),
		)
		await user.click(screen.getByLabelText('Libellé'))
		await user.paste('Rupture à l’amiable')
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		await waitFor(() =>
			expect(callsTo(api, 'post', CLOSURE_REASONS)).toHaveLength(1),
		)
		expect(callsTo(api, 'post', CLOSURE_REASONS)[0]?.params).toMatchObject({
			body: { label: 'Rupture à l’amiable' },
		})
	})

	it('shows the three blocks without any write button to a reader', async () => {
		renderSection(['VIEW_CURSUS'])

		expect(await screen.findByText('Établissements')).toBeTruthy()
		expect(screen.getByText('Formations')).toBeTruthy()
		expect(screen.getByText('Motifs de clôture')).toBeTruthy()
		expect(
			screen.queryByRole('button', { name: 'Nouvel établissement' }),
		).toBeNull()
		expect(
			screen.queryByRole('button', { name: 'Nouvelle formation' }),
		).toBeNull()
		expect(screen.queryByRole('button', { name: 'Nouveau motif' })).toBeNull()
	})

	it('shows nothing of the catalog without the read bit', () => {
		renderSection([])

		expect(screen.queryByText('Établissements')).toBeNull()
	})
})
