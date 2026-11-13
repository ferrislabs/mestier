import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeAll, beforeEach, describe, expect, it, vi } from 'vitest'
import { ParticipantDetailFeature } from '#/pages/cursus/participants/feature/participant-detail-feature'
import {
	ParticipantsListFeature,
	type ParticipantsListFeatureProps,
} from '#/pages/cursus/participants/feature/participants-list-feature'
import {
	callsTo,
	chooseOption,
	createQueryClient,
	type FakeApi,
	installFakeTanstackApi,
	installRadixPolyfills,
	Providers,
	page,
} from '#/pages/support/feature/support-test-kit'

const BASE = '/api/v1/organizations/{organization_id}/cursus'
const PARTICIPANTS = `${BASE}/participants`
const PARTICIPANT = `${BASE}/participants/{participant_id}`
const COMPANY_TUTORS = `${BASE}/participants/{participant_id}/company-tutors`
const INVITATION = `${BASE}/participants/{participant_id}/invitation`
const HOST_COMPANIES = `${BASE}/host-companies`

const PARTICIPANT_ROW = {
	id: 'p-1',
	organization_id: 'org-1',
	last_name: 'Martin',
	first_name: 'Léa',
	email: 'lea@example.fr',
	status: 'pending',
	invitation_expires_at: null,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

const HOST_COMPANY = {
	id: 'hc-1',
	organization_id: 'org-1',
	name: 'Menuiserie Roux',
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

const TUTOR_RECORD = {
	id: 't-1',
	participant_id: 'p-1',
	host_company_id: 'hc-1',
	host_company_name: 'Menuiserie Roux',
	job_title: 'Chef d’atelier',
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

let api: FakeApi

beforeAll(installRadixPolyfills)

beforeEach(() => {
	api = installFakeTanstackApi()
	api.mockGet(PARTICIPANTS, () =>
		page([
			{ ...PARTICIPANT_ROW, apprentice_id: 'a-1', company_tutors: [] },
		] as never),
	)
	api.mockGet(PARTICIPANT, () => ({
		data: { ...PARTICIPANT_ROW, apprentice: null, company_tutors: [] },
	}))
	api.mockGet(HOST_COMPANIES, () => page([HOST_COMPANY] as never))
})

function renderWith(permissions: string[], ui: React.ReactElement) {
	render(
		<Providers queryClient={createQueryClient(permissions)}>{ui}</Providers>,
	)
}

function renderList(
	permissions: string[],
	overrides: Partial<ParticipantsListFeatureProps> = {},
) {
	renderWith(
		permissions,
		<ParticipantsListFeature
			kind="apprentice"
			status={undefined}
			hostCompanyId={undefined}
			onFiltersChange={vi.fn()}
			onOpenParticipant={vi.fn()}
			{...overrides}
		/>,
	)
}

function lastListQuery() {
	return (callsTo(api, 'get', PARTICIPANTS).at(-1)?.params as { query: object })
		.query
}

const MANAGER = ['VIEW_CURSUS', 'MANAGE_CURSUS_PARTICIPANTS']

describe('Apprentices list', () => {
	it('creates an apprentice in three steps without a tutor call', {
		timeout: 15_000,
	}, async () => {
		const user = userEvent.setup()
		const onOpenParticipant = vi.fn()
		api.mockMutation('post', PARTICIPANTS, () => ({ data: PARTICIPANT_ROW }))
		renderList(MANAGER, { onOpenParticipant })

		await user.click(
			await screen.findByRole('button', { name: 'Nouvel apprenti' }),
		)
		expect(
			screen.getByRole('heading', { name: 'Nouvel apprenti' }),
		).toBeTruthy()
		expect(screen.queryByRole('combobox', { name: /Type/ })).toBeNull()
		expect(screen.getByText('Apprentissage')).toBeTruthy()
		await user.click(screen.getByLabelText('Nom'))
		await user.paste('Dupont')
		await user.click(screen.getByLabelText('Prénom'))
		await user.paste('Jo')
		await user.click(screen.getByRole('button', { name: 'Suivant' }))
		await user.click(screen.getByLabelText('E-mail'))
		await user.paste('jo@example.fr')
		await user.click(screen.getByLabelText('Adresse'))
		await user.paste('1 rue Haute')
		await user.click(screen.getByRole('button', { name: 'Suivant' }))
		expect(callsTo(api, 'post', PARTICIPANTS)).toHaveLength(0)
		await user.click(screen.getByLabelText('Code INE'))
		await user.paste('123456789AB')
		await user.click(screen.getByRole('button', { name: "Créer l'apprenti" }))

		await waitFor(() =>
			expect(callsTo(api, 'post', PARTICIPANTS)).toHaveLength(1),
		)
		expect(callsTo(api, 'post', PARTICIPANTS)[0]?.params).toEqual({
			path: { organization_id: 'org-1' },
			body: {
				last_name: 'Dupont',
				first_name: 'Jo',
				email: 'jo@example.fr',
				phone: null,
				apprentice: {
					birth_date: null,
					ine_code: '123456789AB',
					address: '1 rue Haute',
				},
			},
		})
		expect(callsTo(api, 'post', COMPANY_TUTORS)).toHaveLength(0)
		await waitFor(() => expect(onOpenParticipant).toHaveBeenCalledWith('p-1'))
	})

	it('stays on the identity step while the name is empty', {
		timeout: 15_000,
	}, async () => {
		const user = userEvent.setup()
		renderList(MANAGER)

		await user.click(
			await screen.findByRole('button', { name: 'Nouvel apprenti' }),
		)
		await user.click(screen.getByLabelText('Prénom'))
		await user.paste('Jo')
		await user.click(screen.getByRole('button', { name: 'Suivant' }))

		expect(screen.getByRole('alert').textContent).toBe(
			'Le nom est obligatoire.',
		)
		expect(screen.queryByLabelText('E-mail')).toBeNull()
	})

	it('restarts at the first step with empty fields when reopened', {
		timeout: 15_000,
	}, async () => {
		const user = userEvent.setup()
		renderList(MANAGER)

		await user.click(
			await screen.findByRole('button', { name: 'Nouvel apprenti' }),
		)
		await user.click(screen.getByLabelText('Nom'))
		await user.paste('Dupont')
		await user.click(screen.getByRole('button', { name: 'Suivant' }))
		await user.click(screen.getByRole('button', { name: 'Précédent' }))
		expect((screen.getByLabelText('Nom') as HTMLInputElement).value).toBe(
			'Dupont',
		)
		await user.keyboard('{Escape}')
		await user.click(
			await screen.findByRole('button', { name: 'Nouvel apprenti' }),
		)

		expect((screen.getByLabelText('Nom') as HTMLInputElement).value).toBe('')
	})

	it('has no creation button without the manage bit', async () => {
		renderList(['VIEW_CURSUS'])

		expect(await screen.findByText('Léa Martin')).toBeTruthy()
		expect(screen.queryByRole('button', { name: 'Nouvel apprenti' })).toBeNull()
	})

	it('asks only for apprentices with the status filter', async () => {
		renderList(['VIEW_CURSUS'], { status: 'pending' })

		await screen.findByText('Léa Martin')
		expect(lastListQuery()).toMatchObject({
			kind: 'apprentice',
			status: 'pending',
			per_page: 25,
		})
		expect(lastListQuery()).not.toHaveProperty('host_company_id')
	})

	it('counts the apprentices from the pagination total', async () => {
		api.mockGet(PARTICIPANTS, () =>
			page(
				[
					{ ...PARTICIPANT_ROW, apprentice_id: 'a-1', company_tutors: [] },
				] as never,
				12,
			),
		)
		renderList(['VIEW_CURSUS'])

		expect(await screen.findByText('12 apprentis')).toBeTruthy()
	})

	it('sends the typed search as q from the first page', {
		timeout: 15_000,
	}, async () => {
		const user = userEvent.setup()
		renderList(['VIEW_CURSUS'])
		await screen.findByText('Léa Martin')

		await user.click(
			screen.getByPlaceholderText('Rechercher un nom, un e-mail'),
		)
		await user.paste('lea')

		await waitFor(() =>
			expect(lastListQuery()).toMatchObject({
				q: 'lea',
				kind: 'apprentice',
				page: 1,
			}),
		)
	})
})

describe('Tutors list', () => {
	beforeEach(() => {
		api.mockGet(PARTICIPANTS, () =>
			page(
				[
					{
						...PARTICIPANT_ROW,
						apprentice_id: null,
						company_tutors: [TUTOR_RECORD],
					},
				] as never,
				7,
			),
		)
	})

	it('asks only for tutors and shows their company and job', async () => {
		renderList(['VIEW_CURSUS'], { kind: 'company_tutor' })

		expect(await screen.findByText('7 tuteurs')).toBeTruthy()
		expect(lastListQuery()).toMatchObject({ kind: 'company_tutor' })
		expect(screen.getByText('Chef d’atelier')).toBeTruthy()
		expect(screen.getByText('Menuiserie Roux', { selector: 'td' })).toBeTruthy()
		expect(callsTo(api, 'get', PARTICIPANT)).toHaveLength(0)
	})

	it('filters by host company', { timeout: 15_000 }, async () => {
		const user = userEvent.setup()
		const onFiltersChange = vi.fn()
		renderList(['VIEW_CURSUS'], { kind: 'company_tutor', onFiltersChange })
		await screen.findByText('Léa Martin')

		await chooseOption(
			user,
			screen.getByRole('combobox', { name: 'Filtrer par entreprise' }),
			'Menuiserie Roux',
		)

		expect(onFiltersChange).toHaveBeenCalledWith({
			status: undefined,
			hostCompanyId: 'hc-1',
		})
	})

	it('sends the selected host company to the list query', async () => {
		renderList(['VIEW_CURSUS'], {
			kind: 'company_tutor',
			hostCompanyId: 'hc-1',
		})

		await screen.findByText('Léa Martin')
		expect(lastListQuery()).toMatchObject({
			kind: 'company_tutor',
			host_company_id: 'hc-1',
		})
	})

	it('needs the host company before creating the tutor', {
		timeout: 15_000,
	}, async () => {
		const user = userEvent.setup()
		api.mockMutation('post', PARTICIPANTS, () => ({ data: PARTICIPANT_ROW }))
		renderList(MANAGER, { kind: 'company_tutor' })

		await user.click(
			await screen.findByRole('button', { name: 'Nouveau tuteur' }),
		)
		expect(screen.getByRole('heading', { name: 'Nouveau tuteur' })).toBeTruthy()
		await user.click(screen.getByLabelText('Nom'))
		await user.paste('Roux')
		await user.click(screen.getByRole('button', { name: 'Suivant' }))
		expect(screen.queryByLabelText('Adresse')).toBeNull()
		await user.click(screen.getByLabelText('E-mail'))
		await user.paste('roux@example.fr')
		await user.click(screen.getByRole('button', { name: 'Suivant' }))

		expect(
			document.querySelector('[aria-current="step"]')?.textContent,
		).toContain('Entreprise')
		expect(screen.queryByLabelText('Code INE')).toBeNull()
		await user.click(screen.getByRole('button', { name: 'Créer le tuteur' }))
		expect(screen.getByRole('alert').textContent).toBe(
			"Choisissez l'entreprise d'accueil du tuteur.",
		)
		expect(callsTo(api, 'post', PARTICIPANTS)).toHaveLength(0)

		await chooseOption(
			user,
			screen.getByRole('combobox', { name: "Entreprise d'accueil" }),
			'Menuiserie Roux',
		)
		await user.click(screen.getByLabelText('Fonction'))
		await user.paste('Chef d’atelier')
		await user.click(screen.getByRole('button', { name: 'Créer le tuteur' }))

		await waitFor(() =>
			expect(callsTo(api, 'post', PARTICIPANTS)).toHaveLength(1),
		)
		expect(
			(callsTo(api, 'post', PARTICIPANTS)[0]?.params as { body: object }).body,
		).toEqual({
			last_name: 'Roux',
			first_name: null,
			email: 'roux@example.fr',
			phone: null,
			company_tutor: { host_company_id: 'hc-1', job_title: 'Chef d’atelier' },
		})
	})
})

describe('ParticipantDetailFeature', () => {
	it('shows the invitation link once with its expiry', async () => {
		const user = userEvent.setup()
		api.mockMutation('post', INVITATION, () => ({
			data: { token: 'tok-123', expires_at: '2026-12-01T00:00:00Z' },
		}))
		renderWith(MANAGER, <ParticipantDetailFeature participantId="p-1" />)

		await user.click(await screen.findByRole('button', { name: 'Inviter' }))

		const link = (await screen.findByLabelText(
			"Lien d'invitation",
		)) as HTMLInputElement
		expect(link.value).toBe(
			`${window.location.origin}/portail/invitation/tok-123`,
		)
		expect(screen.getByText(/Valable jusqu'au/)).toBeTruthy()
	})
})
