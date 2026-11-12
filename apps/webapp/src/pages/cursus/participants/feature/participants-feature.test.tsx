import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeAll, beforeEach, describe, expect, it, vi } from 'vitest'
import { ParticipantDetailFeature } from '#/pages/cursus/participants/feature/participant-detail-feature'
import { ParticipantsListFeature } from '#/pages/cursus/participants/feature/participants-list-feature'
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
const PARTICIPANTS = `${BASE}/participants`
const PARTICIPANT = `${BASE}/participants/{participant_id}`
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

describe('ParticipantsListFeature', () => {
	it('creates an apprentice with the apprentice block', {
		timeout: 15_000,
	}, async () => {
		const user = userEvent.setup()
		const onOpenParticipant = vi.fn()
		api.mockMutation('post', PARTICIPANTS, () => ({ data: PARTICIPANT_ROW }))
		renderWith(
			['VIEW_CURSUS', 'MANAGE_CURSUS_PARTICIPANTS'],
			<ParticipantsListFeature
				kind={undefined}
				status={undefined}
				onFiltersChange={vi.fn()}
				onOpenParticipant={onOpenParticipant}
			/>,
		)

		await user.click(
			await screen.findByRole('button', { name: 'Nouveau participant' }),
		)
		await user.click(screen.getByLabelText('Nom'))
		await user.paste('Dupont')
		await user.click(screen.getByLabelText('Prénom'))
		await user.paste('Jo')
		await user.click(screen.getByLabelText('E-mail'))
		await user.paste('jo@example.fr')
		await user.click(screen.getByLabelText('Code INE'))
		await user.paste('123456789AB')
		await user.click(
			screen.getByRole('button', { name: 'Créer le participant' }),
		)

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
					address: null,
				},
			},
		})
		await waitFor(() => expect(onOpenParticipant).toHaveBeenCalledWith('p-1'))
	})

	it('has no creation button without the manage bit', async () => {
		renderWith(
			['VIEW_CURSUS'],
			<ParticipantsListFeature
				kind={undefined}
				status={undefined}
				onFiltersChange={vi.fn()}
				onOpenParticipant={vi.fn()}
			/>,
		)

		expect(await screen.findByText('Léa Martin')).toBeTruthy()
		expect(
			screen.queryByRole('button', { name: 'Nouveau participant' }),
		).toBeNull()
	})

	it('passes the kind and status filters to the list query', async () => {
		renderWith(
			['VIEW_CURSUS'],
			<ParticipantsListFeature
				kind="company_tutor"
				status="pending"
				onFiltersChange={vi.fn()}
				onOpenParticipant={vi.fn()}
			/>,
		)

		await screen.findByText('Léa Martin')
		expect(
			(callsTo(api, 'get', PARTICIPANTS)[0]?.params as { query: object }).query,
		).toMatchObject({ kind: 'company_tutor', status: 'pending' })
	})
})

describe('ParticipantsListFeature search', () => {
	it('counts the participants from the pagination total', async () => {
		api.mockGet(PARTICIPANTS, () =>
			page(
				[
					{ ...PARTICIPANT_ROW, apprentice_id: 'a-1', company_tutors: [] },
				] as never,
				58,
			),
		)
		renderWith(
			['VIEW_CURSUS'],
			<ParticipantsListFeature
				kind={undefined}
				status={undefined}
				onFiltersChange={vi.fn()}
				onOpenParticipant={vi.fn()}
			/>,
		)

		expect(await screen.findByText('58 participants')).toBeTruthy()
	})

	it('sends the typed search as q from the first page', {
		timeout: 15_000,
	}, async () => {
		const user = userEvent.setup()
		renderWith(
			['VIEW_CURSUS'],
			<ParticipantsListFeature
				kind="apprentice"
				status={undefined}
				onFiltersChange={vi.fn()}
				onOpenParticipant={vi.fn()}
			/>,
		)
		await screen.findByText('Léa Martin')

		await user.click(
			screen.getByPlaceholderText('Rechercher un nom, un e-mail'),
		)
		await user.paste('lea')

		await waitFor(() =>
			expect(
				(callsTo(api, 'get', PARTICIPANTS).at(-1)?.params as { query: object })
					.query,
			).toMatchObject({ q: 'lea', kind: 'apprentice', page: 1 }),
		)
	})
})

describe('ParticipantsListFeature records', () => {
	it('shows the kind and host company from the list without detail calls', async () => {
		api.mockGet(PARTICIPANTS, () =>
			page([
				{
					...PARTICIPANT_ROW,
					apprentice_id: null,
					company_tutors: [
						{
							id: 't-1',
							participant_id: 'p-1',
							host_company_id: 'hc-1',
							host_company_name: 'Menuiserie Roux',
							job_title: null,
							created_at: '2026-01-01T00:00:00Z',
							updated_at: '2026-01-01T00:00:00Z',
						},
					],
				},
			] as never),
		)
		renderWith(
			['VIEW_CURSUS'],
			<ParticipantsListFeature
				kind={undefined}
				status={undefined}
				onFiltersChange={vi.fn()}
				onOpenParticipant={vi.fn()}
			/>,
		)

		expect(await screen.findByText('Tuteur entreprise')).toBeTruthy()
		expect(screen.getByText('Menuiserie Roux')).toBeTruthy()
		expect(callsTo(api, 'get', PARTICIPANT)).toHaveLength(0)
	})
})

describe('ParticipantDetailFeature', () => {
	it('shows the invitation link once with its expiry', async () => {
		const user = userEvent.setup()
		api.mockMutation('post', INVITATION, () => ({
			data: { token: 'tok-123', expires_at: '2026-12-01T00:00:00Z' },
		}))
		renderWith(
			['VIEW_CURSUS', 'MANAGE_CURSUS_PARTICIPANTS'],
			<ParticipantDetailFeature participantId="p-1" />,
		)

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
