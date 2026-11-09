import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeAll, describe, expect, it, vi } from 'vitest'
import { TriadDetailFeature } from '#/pages/cursus/triads/feature/triad-detail-feature'
import { TriadsListFeature } from '#/pages/cursus/triads/feature/triads-list-feature'
import {
	callsTo,
	chooseOption,
	createQueryClient,
	installFakeTanstackApi,
	installRadixPolyfills,
	MEMBERS,
	MEMBERS_PATH,
	Providers,
} from '#/pages/support/feature/support-test-kit'

const BASE = '/api/v1/organizations/{organization_id}/cursus'
const TRIADS_PATH = `${BASE}/triads`
const TRIAD_PATH = `${BASE}/triads/{triad_id}`
const PROMOTIONS_PATH = `${BASE}/promotions`
const PARTICIPANTS_PATH = `${BASE}/participants`
const PARTICIPANT_PATH = `${BASE}/participants/{participant_id}`
const CLOSURE_REASONS_PATH = `${BASE}/closure-reasons`
const CONTRACT_CLOSE_PATH = `${BASE}/contracts/{contract_id}/close`

const STAMPS = {
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

beforeAll(installRadixPolyfills)

function installListReads(api: ReturnType<typeof installFakeTanstackApi>) {
	api.mockGet(TRIADS_PATH, () => ({ data: [] }))
	api.mockGet(PROMOTIONS_PATH, () => ({
		data: [
			{
				id: 'promo-1',
				organization_id: 'org-1',
				training_id: 'training-1',
				label: 'CAP 2026',
				start_year: 2026,
				end_year: 2028,
				...STAMPS,
			},
		],
	}))
	api.mockGet(PARTICIPANTS_PATH, () => ({
		data: [
			{
				id: 'participant-1',
				organization_id: 'org-1',
				email: 'lea@example.fr',
				first_name: 'Lea',
				last_name: 'Martin',
				status: 'active',
				...STAMPS,
			},
		],
	}))
	api.mockGet(PARTICIPANT_PATH, () => ({
		data: {
			id: 'participant-1',
			organization_id: 'org-1',
			email: 'lea@example.fr',
			first_name: 'Lea',
			last_name: 'Martin',
			status: 'active',
			apprentice: {
				id: 'apprentice-9',
				participant_id: 'participant-1',
				...STAMPS,
			},
			company_tutors: [],
			...STAMPS,
		},
	}))
	api.mockGet(MEMBERS_PATH, () => ({ data: MEMBERS }))
}

describe('TriadsListFeature', () => {
	it('creates a triad with the apprentice record id and the chosen tutor', async () => {
		const user = userEvent.setup()
		const api = installFakeTanstackApi()
		installListReads(api)
		api.mockMutation('post', TRIADS_PATH, () => ({ data: { id: 'triad-new' } }))
		const onOpenTriad = vi.fn()
		render(
			<Providers
				queryClient={createQueryClient(['VIEW_CURSUS', 'MANAGE_CURSUS_TRIADS'])}
			>
				<TriadsListFeature
					promotionId={undefined}
					closed={undefined}
					onFiltersChange={vi.fn()}
					onOpenTriad={onOpenTriad}
				/>
			</Providers>,
		)

		await user.click(
			await screen.findByRole('button', { name: /Nouveau trinôme/ }),
		)
		await chooseOption(
			user,
			await screen.findByRole('combobox', { name: /Promotion/ }),
			'CAP 2026',
		)
		await chooseOption(
			user,
			await screen.findByRole('combobox', { name: /Apprenti/ }),
			'Lea Martin',
		)
		await chooseOption(
			user,
			screen.getByRole('combobox', { name: /Tuteur pédagogique/ }),
			'Alix Martin',
		)
		await user.click(screen.getByRole('button', { name: 'Créer le trinôme' }))

		await waitFor(() =>
			expect(callsTo(api, 'post', TRIADS_PATH)).toHaveLength(1),
		)
		expect(callsTo(api, 'post', TRIADS_PATH)[0]?.params).toEqual({
			path: { organization_id: 'org-1' },
			body: {
				apprentice_id: 'apprentice-9',
				promotion_id: 'promo-1',
				pedagogic_tutor_member_id: 'member-1',
			},
		})
		await waitFor(() => expect(onOpenTriad).toHaveBeenCalledWith('triad-new'))
	})

	it('has no creation button without MANAGE_CURSUS_TRIADS', async () => {
		const api = installFakeTanstackApi()
		installListReads(api)
		render(
			<Providers queryClient={createQueryClient(['VIEW_CURSUS'])}>
				<TriadsListFeature
					promotionId={undefined}
					closed={undefined}
					onFiltersChange={vi.fn()}
					onOpenTriad={vi.fn()}
				/>
			</Providers>,
		)

		await screen.findByText('Aucun trinôme')
		expect(screen.queryByRole('button', { name: /Nouveau trinôme/ })).toBeNull()
	})
})

function triadDetail(closed: boolean) {
	return {
		data: {
			id: 'triad-1',
			organization_id: 'org-1',
			apprentice_id: 'apprentice-9',
			promotion_id: 'promo-1',
			apprentice_name: 'Lea Martin',
			apprentice_email: 'lea@example.fr',
			training_name: 'CAP Menuisier',
			pedagogic_tutor_member_id: null,
			pedagogic_tutor_name: null,
			...(closed
				? { closed_at: '2027-01-01', closure_reason_label: 'Diplômé' }
				: {}),
			promotion: {
				id: 'promo-1',
				organization_id: 'org-1',
				training_id: 'training-1',
				label: 'CAP 2026',
				start_year: 2026,
				end_year: 2028,
				...STAMPS,
			},
			contracts: [
				{
					id: 'contract-1',
					organization_id: 'org-1',
					triad_id: 'triad-1',
					host_company_id: 'host-1',
					host_company_name: 'Atelier Bois',
					kind: 'PROFESSIONALIZATION',
					start_date: '2026-09-01',
					end_date: '2028-08-31',
					has_cerfa_file: false,
					has_mission_sheet_file: false,
					tutors: [],
					...STAMPS,
				},
			],
			...STAMPS,
		},
	}
}

function renderDetail(permissions: string[], closed: boolean) {
	const api = installFakeTanstackApi()
	api.mockGet(TRIAD_PATH, () => triadDetail(closed))
	api.mockGet(MEMBERS_PATH, () => ({ data: MEMBERS }))
	api.mockGet(CLOSURE_REASONS_PATH, () => ({
		data: [
			{
				id: 'reason-1',
				organization_id: 'org-1',
				label: 'Rupture',
				...STAMPS,
			},
		],
	}))
	api.mockMutation('post', CONTRACT_CLOSE_PATH, () => ({ data: {} }))
	render(
		<Providers queryClient={createQueryClient(permissions)}>
			<TriadDetailFeature
				triadId="triad-1"
				onBack={vi.fn()}
				onOpenPromotion={vi.fn()}
			/>
		</Providers>,
	)
	return api
}

describe('TriadDetailFeature', () => {
	it('closes a contract with date, reason and note', async () => {
		const user = userEvent.setup()
		const api = renderDetail(['VIEW_CURSUS', 'MANAGE_CURSUS_TRIADS'], false)

		await user.click(
			await screen.findByRole('button', { name: 'Clôturer le contrat' }),
		)
		const dialog = await screen.findByRole('dialog')
		const date = dialog.querySelector('#close-date') as HTMLInputElement
		await user.clear(date)
		await user.type(date, '2027-03-01')
		await chooseOption(
			user,
			screen.getByRole('combobox', { name: /Motif/ }),
			'Rupture',
		)
		await user.type(screen.getByLabelText('Note'), 'Départ')
		await user.click(
			screen
				.getAllByRole('button', { name: 'Clôturer le contrat' })
				.at(-1) as HTMLElement,
		)

		await waitFor(() =>
			expect(callsTo(api, 'post', CONTRACT_CLOSE_PATH)).toHaveLength(1),
		)
		expect(callsTo(api, 'post', CONTRACT_CLOSE_PATH)[0]?.params).toEqual({
			path: { organization_id: 'org-1', contract_id: 'contract-1' },
			body: {
				closed_at: '2027-03-01',
				closure_reason_id: 'reason-1',
				closure_note: 'Départ',
			},
		})
	})

	it('is read-only on a closed triad', async () => {
		renderDetail(['VIEW_CURSUS', 'MANAGE_CURSUS_TRIADS'], true)

		await screen.findByText('Atelier Bois')
		expect(screen.getByText(/Clôturé le 01\/01\/2027 · Diplômé/)).toBeDefined()
		expect(screen.queryByRole('button', { name: /Clôturer/ })).toBeNull()
		expect(
			screen.queryByRole('button', { name: /Ajouter un contrat/ }),
		).toBeNull()
		expect(screen.queryByRole('button', { name: /Modifier/ })).toBeNull()
	})
})
