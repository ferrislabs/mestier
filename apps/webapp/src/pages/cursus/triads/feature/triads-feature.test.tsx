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
const CLOSURE_REASONS_PATH = `${BASE}/closure-reasons`
const CONTRACT_CLOSE_PATH = `${BASE}/contracts/{contract_id}/close`
const SCHOOL_TUTORS_PATH = `${BASE}/school-tutors`

const SCHOOL_TUTOR = {
	id: 'st-1',
	organization_id: 'org-1',
	member_id: 'member-1',
	member_name: 'Alix Martin',
	job_title: null,
	open_triad_count: 0,
	archived_at: null,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

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
				apprentice_id: 'apprentice-9',
				company_tutors: [],
				...STAMPS,
			},
		],
	}))
	api.mockGet(MEMBERS_PATH, () => ({ data: MEMBERS }))
	api.mockGet(SCHOOL_TUTORS_PATH, () => ({ data: [SCHOOL_TUTOR] }))
}

function renderCreation(onDeclareSchoolTutor = vi.fn()) {
	render(
		<Providers
			queryClient={createQueryClient(['VIEW_CURSUS', 'MANAGE_CURSUS_TRIADS'])}
		>
			<TriadsListFeature
				promotionId={undefined}
				status="open"
				onFiltersChange={vi.fn()}
				onOpenTriad={vi.fn()}
				onDeclareSchoolTutor={onDeclareSchoolTutor}
			/>
		</Providers>,
	)
}

describe('TriadsListFeature', () => {
	it('creates a triad with the apprentice record id and the chosen tutor', {
		timeout: 30_000,
	}, async () => {
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
					status="open"
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
			screen.getByRole('combobox', { name: /Tuteur école/ }),
			'Alix Martin',
		)
		expect(
			callsTo(api, 'get', SCHOOL_TUTORS_PATH).map(
				(call) => (call.params as { query: object }).query,
			),
		).toContainEqual(expect.objectContaining({ archived: 'active' }))
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

	it('offers only the active school tutors and a way to declare one', {
		timeout: 15_000,
	}, async () => {
		const user = userEvent.setup()
		const api = installFakeTanstackApi()
		installListReads(api)
		const onDeclareSchoolTutor = vi.fn()
		renderCreation(onDeclareSchoolTutor)

		await user.click(
			await screen.findByRole('button', { name: /Nouveau trinôme/ }),
		)
		await user.click(screen.getByRole('combobox', { name: /Tuteur école/ }))
		expect(
			await screen.findByRole('option', { name: 'Alix Martin' }),
		).toBeTruthy()
		expect(screen.queryByRole('option', { name: 'Jo Durand' })).toBeNull()
		await user.keyboard('{Escape}')
		await user.click(
			screen.getByRole('button', { name: 'Déclarer un tuteur école' }),
		)
		expect(onDeclareSchoolTutor).toHaveBeenCalled()
	})

	it('shows a refused school tutor under the field', {
		timeout: 30_000,
	}, async () => {
		const user = userEvent.setup()
		const api = installFakeTanstackApi()
		installListReads(api)
		api.mockMutation('post', TRIADS_PATH, () => {
			throw Object.assign(new Error('not a school tutor'), {
				status: 422,
				data: { code: 'E_NOT_A_SCHOOL_TUTOR' },
			})
		})
		renderCreation()

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
			screen.getByRole('combobox', { name: /Tuteur école/ }),
			'Alix Martin',
		)
		await user.click(screen.getByRole('button', { name: 'Créer le trinôme' }))

		expect(
			await screen.findByText("Ce membre n'est pas un tuteur école actif."),
		).toBeTruthy()
		expect(screen.queryByRole('alert')).toBeNull()
	})

	it('has no creation button without MANAGE_CURSUS_TRIADS', async () => {
		const api = installFakeTanstackApi()
		installListReads(api)
		render(
			<Providers queryClient={createQueryClient(['VIEW_CURSUS'])}>
				<TriadsListFeature
					promotionId={undefined}
					status="open"
					onFiltersChange={vi.fn()}
					onOpenTriad={vi.fn()}
				/>
			</Providers>,
		)

		await screen.findByText('Aucun trinôme ne correspond.')
		expect(screen.queryByRole('button', { name: /Nouveau trinôme/ })).toBeNull()
	})
})

function triadQueries(api: ReturnType<typeof installFakeTanstackApi>) {
	return callsTo(api, 'get', TRIADS_PATH).map(
		(call) => (call.params as { query: Record<string, unknown> }).query,
	)
}

describe('TriadsListFeature filters', () => {
	function renderList(status: 'open' | 'closed' | 'all') {
		const api = installFakeTanstackApi()
		installListReads(api)
		render(
			<Providers queryClient={createQueryClient(['VIEW_CURSUS'])}>
				<TriadsListFeature
					promotionId={undefined}
					status={status}
					onFiltersChange={vi.fn()}
					onOpenTriad={vi.fn()}
				/>
			</Providers>,
		)
		return api
	}

	it('lists the open triads by default and counts them', async () => {
		const api = renderList('open')

		expect(await screen.findByText('Aucun trinôme ouvert')).toBeTruthy()
		expect(triadQueries(api)[0]).toMatchObject({ closed: false })
	})

	it('drops the closed filter for every triad', async () => {
		const api = renderList('all')

		await screen.findByText('Aucun trinôme')
		expect(triadQueries(api)[0]).not.toHaveProperty('closed')
	})

	it('sends the typed search as q', { timeout: 15_000 }, async () => {
		const user = userEvent.setup()
		const api = renderList('open')
		await screen.findByText('Aucun trinôme ouvert')

		await user.click(
			screen.getByPlaceholderText(
				'Rechercher un apprenti, une entreprise, une promotion',
			),
		)
		await user.paste('Lea')

		await waitFor(() =>
			expect(triadQueries(api).at(-1)).toMatchObject({
				q: 'Lea',
				closed: false,
			}),
		)
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
					duration_months: 24,
					has_mission_sheet_file: false,
					tutors: [],
					...STAMPS,
				},
			],
			...STAMPS,
		},
	}
}

function renderDetail(
	permissions: string[],
	closed: boolean,
	tutor: { id: string; name: string } | null = null,
) {
	const api = installFakeTanstackApi()
	api.mockGet(TRIAD_PATH, () => {
		const detail = triadDetail(closed)
		return {
			data: {
				...detail.data,
				pedagogic_tutor_member_id: tutor?.id ?? null,
				pedagogic_tutor_name: tutor?.name ?? null,
			},
		}
	})
	api.mockGet(SCHOOL_TUTORS_PATH, () => ({ data: [SCHOOL_TUTOR] }))
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
	it('closes a contract with date, reason and note', {
		timeout: 15_000,
	}, async () => {
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
		await user.click(screen.getByLabelText('Note'))
		await user.paste('Départ')
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

	it('keeps an archived school tutor selected', {
		timeout: 15_000,
	}, async () => {
		const user = userEvent.setup()
		renderDetail(['VIEW_CURSUS', 'MANAGE_CURSUS_TRIADS'], false, {
			id: 'member-2',
			name: 'Jo Durand',
		})

		expect(await screen.findByText('Tuteur école')).toBeTruthy()
		const select = await screen.findByRole('combobox', {
			name: /Tuteur école/,
		})
		await waitFor(() => expect(select.textContent).toContain('Jo Durand'))
		await user.click(select)
		expect(
			await screen.findByRole('option', { name: 'Alix Martin' }),
		).toBeTruthy()
		expect(screen.getByRole('option', { name: 'Jo Durand' })).toBeTruthy()
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
