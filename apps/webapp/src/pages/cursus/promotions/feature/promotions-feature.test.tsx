import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeAll, describe, expect, it, vi } from 'vitest'
import { PromotionsListFeature } from '#/pages/cursus/promotions/feature/promotions-list-feature'
import {
	callsTo,
	chooseOption,
	createQueryClient,
	installFakeTanstackApi,
	installRadixPolyfills,
	Providers,
} from '#/pages/support/feature/support-test-kit'

const PROMOTIONS_PATH =
	'/api/v1/organizations/{organization_id}/cursus/promotions'
const TRAININGS_PATH =
	'/api/v1/organizations/{organization_id}/cursus/trainings'

beforeAll(installRadixPolyfills)

function setup(permissions: string[]) {
	const api = installFakeTanstackApi()
	api.mockGet(PROMOTIONS_PATH, () => ({
		data: [
			{
				id: 'promo-1',
				organization_id: 'org-1',
				training_id: 'training-1',
				label: 'CAP 2026',
				start_year: 2026,
				end_year: 2028,
				created_at: '2026-01-01T00:00:00Z',
				updated_at: '2026-01-01T00:00:00Z',
			},
		],
	}))
	api.mockGet(TRAININGS_PATH, () => ({
		data: [
			{
				id: 'training-1',
				organization_id: 'org-1',
				establishment_id: 'est-1',
				name: 'CAP Menuisier',
				created_at: '2026-01-01T00:00:00Z',
				updated_at: '2026-01-01T00:00:00Z',
			},
		],
	}))
	api.mockMutation('post', PROMOTIONS_PATH, () => ({
		data: { id: 'promo-2' },
	}))
	const onOpenPromotion = vi.fn()
	render(
		<Providers queryClient={createQueryClient(permissions)}>
			<PromotionsListFeature
				archived={false}
				trainingId={undefined}
				onFiltersChange={vi.fn()}
				onOpenPromotion={onOpenPromotion}
			/>
		</Providers>,
	)
	return { api, onOpenPromotion }
}

describe('PromotionsListFeature', () => {
	it('lists promotions with their training name', async () => {
		setup(['VIEW_CURSUS'])

		expect(await screen.findByText('CAP 2026')).toBeDefined()
		expect(
			await screen.findByText('CAP Menuisier', { selector: 'td' }),
		).toBeDefined()
	})

	it('has no creation button without MANAGE_CURSUS_CATALOG', async () => {
		setup(['VIEW_CURSUS'])
		await screen.findByText('CAP 2026')

		expect(
			screen.queryByRole('button', { name: /Nouvelle promotion/ }),
		).toBeNull()
	})

	it('creates a promotion with the typed payload', {
		timeout: 15_000,
	}, async () => {
		const user = userEvent.setup()
		const { api } = setup(['VIEW_CURSUS', 'MANAGE_CURSUS_CATALOG'])
		await screen.findByText('CAP 2026')

		await user.click(screen.getByRole('button', { name: /Nouvelle promotion/ }))
		await chooseOption(
			user,
			await screen.findByRole('combobox', { name: /Formation/ }),
			'CAP Menuisier',
		)
		await user.click(screen.getByLabelText(/Libellé/))
		await user.paste('CAP 2027')
		await user.click(screen.getByLabelText(/Année de début/))
		await user.paste('2027')
		await user.click(screen.getByLabelText(/Année de fin/))
		await user.paste('2029')
		await user.click(screen.getByRole('button', { name: 'Créer la promotion' }))

		await waitFor(() =>
			expect(callsTo(api, 'post', PROMOTIONS_PATH)).toHaveLength(1),
		)
		expect(callsTo(api, 'post', PROMOTIONS_PATH)[0]?.params).toEqual({
			path: { organization_id: 'org-1' },
			body: {
				training_id: 'training-1',
				label: 'CAP 2027',
				start_year: 2027,
				end_year: 2029,
			},
		})
	})
})
