import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeAll, beforeEach, describe, expect, it, vi } from 'vitest'
import { HostCompaniesListFeature } from '#/pages/cursus/host-companies/feature/host-companies-list-feature'
import {
	callsTo,
	createQueryClient,
	type FakeApi,
	installFakeTanstackApi,
	installRadixPolyfills,
	Providers,
	page,
} from '#/pages/support/feature/support-test-kit'

const HOST_COMPANIES =
	'/api/v1/organizations/{organization_id}/cursus/host-companies'

const COMPANY = {
	id: 'hc-1',
	organization_id: 'org-1',
	name: 'Menuiserie Roux',
	siret: '12345678900011',
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

let api: FakeApi

beforeAll(installRadixPolyfills)

beforeEach(() => {
	api = installFakeTanstackApi()
	api.mockGet(HOST_COMPANIES, () => page([COMPANY] as never))
})

function renderFeature(permissions: string[], onOpen = vi.fn()) {
	render(
		<Providers queryClient={createQueryClient(permissions)}>
			<HostCompaniesListFeature
				includeArchived={false}
				onIncludeArchivedChange={vi.fn()}
				onOpenHostCompany={onOpen}
			/>
		</Providers>,
	)
}

describe('HostCompaniesListFeature', () => {
	it('creates a host company with the typed payload', async () => {
		const user = userEvent.setup()
		const onOpen = vi.fn()
		api.mockMutation('post', HOST_COMPANIES, () => ({ data: COMPANY }))
		renderFeature(['VIEW_CURSUS', 'MANAGE_CURSUS_TRIADS'], onOpen)

		await user.click(
			await screen.findByRole('button', { name: 'Nouvelle entreprise' }),
		)
		await user.type(screen.getByLabelText('Raison sociale'), 'Charpente Blanc')
		await user.type(screen.getByLabelText('SIRET'), '98765432100022')
		await user.type(screen.getByLabelText('Effectif'), '8')
		await user.click(screen.getByRole('button', { name: "Créer l'entreprise" }))

		await waitFor(() =>
			expect(callsTo(api, 'post', HOST_COMPANIES)).toHaveLength(1),
		)
		expect(callsTo(api, 'post', HOST_COMPANIES)[0]?.params).toEqual({
			path: { organization_id: 'org-1' },
			body: {
				name: 'Charpente Blanc',
				siret: '98765432100022',
				naf_code: null,
				legal_form: null,
				director_name: null,
				employee_count: 8,
				address: null,
			},
		})
		await waitFor(() => expect(onOpen).toHaveBeenCalledWith('hc-1'))
	})

	it('has no creation button with only the read bit', async () => {
		renderFeature(['VIEW_CURSUS'])

		expect(await screen.findByText('Menuiserie Roux')).toBeTruthy()
		expect(
			screen.queryByRole('button', { name: 'Nouvelle entreprise' }),
		).toBeNull()
	})

	it('has no creation button with the participants bit only', async () => {
		renderFeature(['VIEW_CURSUS', 'MANAGE_CURSUS_PARTICIPANTS'])

		expect(await screen.findByText('Menuiserie Roux')).toBeTruthy()
		expect(
			screen.queryByRole('button', { name: 'Nouvelle entreprise' }),
		).toBeNull()
	})
})
