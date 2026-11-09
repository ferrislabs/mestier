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
const CLOSURE_REASONS = `${BASE}/closure-reasons`

let api: FakeApi

beforeAll(installRadixPolyfills)

beforeEach(() => {
	api = installFakeTanstackApi()
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

	it('shows only the closure reasons, read-only for a reader', async () => {
		renderSection(['VIEW_CURSUS'])

		expect(await screen.findByText('Motifs de clôture')).toBeTruthy()
		expect(screen.queryByText('Établissements')).toBeNull()
		expect(screen.queryByText('Formations')).toBeNull()
		expect(screen.queryByRole('button', { name: 'Nouveau motif' })).toBeNull()
	})

	it('shows nothing of the catalog without the read bit', () => {
		renderSection([])

		expect(screen.queryByText('Motifs de clôture')).toBeNull()
	})
})
