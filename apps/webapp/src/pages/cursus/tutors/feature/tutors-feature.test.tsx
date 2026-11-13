import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeAll, beforeEach, describe, expect, it, vi } from 'vitest'
import {
	TutorsFeature,
	type TutorsFeatureProps,
} from '#/pages/cursus/tutors/feature/tutors-feature'
import {
	callsTo,
	chooseOption,
	createQueryClient,
	type FakeApi,
	installFakeTanstackApi,
	installRadixPolyfills,
	MEMBERS,
	MEMBERS_PATH,
	Providers,
	page,
} from '#/pages/support/feature/support-test-kit'

const BASE = '/api/v1/organizations/{organization_id}/cursus'
const PARTICIPANTS = `${BASE}/participants`
const HOST_COMPANIES = `${BASE}/host-companies`
const SCHOOL_TUTORS = `${BASE}/school-tutors`
const SCHOOL_TUTOR = `${BASE}/school-tutors/{school_tutor_id}`
const ARCHIVE = `${BASE}/school-tutors/{school_tutor_id}/archive`

const STAMPS = {
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

const ALIX = {
	id: 'st-1',
	organization_id: 'org-1',
	member_id: 'member-1',
	member_name: 'Alix Martin',
	member_email: 'alix@example.fr',
	job_title: 'Formatrice',
	open_triad_count: 2,
	archived_at: null,
	...STAMPS,
}

const COMPANY_TUTOR = {
	id: 'p-1',
	organization_id: 'org-1',
	last_name: 'Roux',
	first_name: 'Paul',
	email: 'paul@example.fr',
	status: 'active',
	apprentice_id: null,
	company_tutors: [],
	...STAMPS,
}

const MANAGER = ['VIEW_CURSUS', 'MANAGE_CURSUS_PARTICIPANTS']

let api: FakeApi

beforeAll(installRadixPolyfills)

beforeEach(() => {
	api = installFakeTanstackApi()
	api.mockGet(PARTICIPANTS, () => page([COMPANY_TUTOR] as never, 7))
	api.mockGet(HOST_COMPANIES, () => page([] as never))
	api.mockGet(SCHOOL_TUTORS, () => page([ALIX] as never, 3))
	api.mockGet(MEMBERS_PATH, () => ({ data: MEMBERS }))
})

function renderTutors(
	permissions: string[],
	overrides: Partial<TutorsFeatureProps> = {},
) {
	render(
		<Providers queryClient={createQueryClient(permissions)}>
			<TutorsFeature
				tab="ecole"
				status={undefined}
				hostCompanyId={undefined}
				onTabChange={vi.fn()}
				onFiltersChange={vi.fn()}
				onOpenParticipant={vi.fn()}
				{...overrides}
			/>
		</Providers>,
	)
}

function schoolQueries() {
	return callsTo(api, 'get', SCHOOL_TUTORS).map(
		(call) => (call.params as { query: Record<string, unknown> }).query,
	)
}

describe('Tutors tabs', () => {
	it('shows both totals and switches tab', { timeout: 15_000 }, async () => {
		const user = userEvent.setup()
		const onTabChange = vi.fn()
		renderTutors(['VIEW_CURSUS'], { tab: 'entreprise', onTabChange })

		expect(await screen.findByText('7 tuteurs entreprise')).toBeTruthy()
		await user.click(await screen.findByRole('tab', { name: 'École · 3' }))
		expect(screen.getByRole('tab', { name: 'Entreprise · 7' })).toBeTruthy()
		expect(onTabChange).toHaveBeenCalledWith('ecole')
	})
})

describe('School tutors tab', () => {
	it('lists the school tutors with their four columns', async () => {
		renderTutors(['VIEW_CURSUS'])

		expect(await screen.findByText('Alix Martin')).toBeTruthy()
		expect(screen.getByText('3 tuteurs école')).toBeTruthy()
		for (const header of ['Nom', 'Fonction', 'Trinômes suivis', 'Statut']) {
			expect(screen.getByRole('columnheader', { name: header })).toBeTruthy()
		}
		const row = screen.getByText('Alix Martin').closest('tr') as HTMLElement
		expect(within(row).getByText('Formatrice')).toBeTruthy()
		expect(within(row).getByText('2')).toBeTruthy()
		expect(within(row).getByText('Actif')).toBeTruthy()
		expect(schoolQueries()).toContainEqual(
			expect.objectContaining({ archived: 'active', page: 1, per_page: 25 }),
		)
	})

	it('sends the search as q and the archive filter', {
		timeout: 15_000,
	}, async () => {
		const user = userEvent.setup()
		renderTutors(['VIEW_CURSUS'])
		await screen.findByText('Alix Martin')

		await user.click(screen.getByPlaceholderText('Rechercher un nom'))
		await user.paste('ali')
		await waitFor(() =>
			expect(schoolQueries().at(-1)).toMatchObject({
				q: 'ali',
				archived: 'active',
			}),
		)
		await user.click(screen.getByRole('radio', { name: 'Archivés' }))
		await waitFor(() =>
			expect(schoolQueries().at(-1)).toMatchObject({
				q: 'ali',
				archived: 'archived',
			}),
		)
	})

	it('edits the job title and archives from the row menu', {
		timeout: 15_000,
	}, async () => {
		const user = userEvent.setup()
		api.mockMutation('patch', SCHOOL_TUTOR, () => ({ data: ALIX }))
		api.mockMutation('post', ARCHIVE, () => ({ data: ALIX }))
		renderTutors(MANAGER)

		await user.click(
			await screen.findByRole('button', { name: 'Actions Alix Martin' }),
		)
		await user.click(
			await screen.findByRole('menuitem', { name: 'Modifier la fonction' }),
		)
		const field = (await screen.findByLabelText('Fonction')) as HTMLInputElement
		await user.clear(field)
		await user.paste('Référente')
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))
		await waitFor(() =>
			expect(callsTo(api, 'patch', SCHOOL_TUTOR)).toHaveLength(1),
		)
		expect(callsTo(api, 'patch', SCHOOL_TUTOR)[0]?.params).toEqual({
			path: { organization_id: 'org-1', school_tutor_id: 'st-1' },
			body: { job_title: 'Référente' },
		})

		await user.click(
			await screen.findByRole('button', { name: 'Actions Alix Martin' }),
		)
		await user.click(await screen.findByRole('menuitem', { name: 'Archiver' }))
		await waitFor(() => expect(callsTo(api, 'post', ARCHIVE)).toHaveLength(1))
	})
})

describe('New tutor', () => {
	it('declares a member as school tutor, offering only undeclared members', {
		timeout: 20_000,
	}, async () => {
		const user = userEvent.setup()
		api.mockMutation('post', SCHOOL_TUTORS, () => ({ data: ALIX }))
		renderTutors(MANAGER)

		await user.click(
			await screen.findByRole('button', { name: 'Nouveau tuteur' }),
		)
		expect(
			screen.getByRole('radio', { name: /Tuteur entreprise/ }),
		).toBeTruthy()
		await user.click(screen.getByRole('radio', { name: /Tuteur école/ }))
		await user.click(screen.getByRole('button', { name: 'Suivant' }))
		expect(
			screen.getByRole('heading', { name: 'Nouveau tuteur école' }),
		).toBeTruthy()
		await waitFor(() =>
			expect(schoolQueries()).toContainEqual(
				expect.objectContaining({ archived: 'all' }),
			),
		)

		await user.click(screen.getByRole('combobox', { name: 'Membre' }))
		expect(
			await screen.findByRole('option', { name: 'Jo Durand' }),
		).toBeTruthy()
		expect(screen.queryByRole('option', { name: 'Alix Martin' })).toBeNull()
		await user.click(screen.getByRole('option', { name: 'Jo Durand' }))
		await user.click(screen.getByLabelText('Fonction'))
		await user.paste('Formateur')
		await user.click(screen.getByRole('button', { name: 'Créer le tuteur' }))

		await waitFor(() =>
			expect(callsTo(api, 'post', SCHOOL_TUTORS)).toHaveLength(1),
		)
		expect(callsTo(api, 'post', SCHOOL_TUTORS)[0]?.params).toEqual({
			path: { organization_id: 'org-1' },
			body: { member_id: 'member-2', job_title: 'Formateur' },
		})
	})

	it('explains a conflict on an already declared member', {
		timeout: 20_000,
	}, async () => {
		const user = userEvent.setup()
		api.mockMutation('post', SCHOOL_TUTORS, () => {
			throw Object.assign(new Error('Conflict'), {
				status: 409,
				data: { code: 'E_CONFLICT' },
			})
		})
		renderTutors(MANAGER)

		await user.click(
			await screen.findByRole('button', { name: 'Nouveau tuteur' }),
		)
		await user.click(screen.getByRole('radio', { name: /Tuteur école/ }))
		await user.click(screen.getByRole('button', { name: 'Suivant' }))
		await chooseOption(
			user,
			screen.getByRole('combobox', { name: 'Membre' }),
			'Jo Durand',
		)
		await user.click(screen.getByRole('button', { name: 'Créer le tuteur' }))

		expect((await screen.findByRole('alert')).textContent).toBe(
			'Ce membre est déjà déclaré comme tuteur école.',
		)
	})

	it('goes on to the company tutor steps', { timeout: 15_000 }, async () => {
		const user = userEvent.setup()
		renderTutors(MANAGER, { tab: 'entreprise' })

		await user.click(
			await screen.findByRole('button', { name: 'Nouveau tuteur' }),
		)
		await user.click(screen.getByRole('button', { name: 'Suivant' }))

		expect(
			document.querySelector('[aria-current="step"]')?.textContent,
		).toContain('Identité')
		expect(screen.getByLabelText('Nom')).toBeTruthy()
		await user.click(screen.getByRole('button', { name: 'Précédent' }))
		expect(
			screen.getByRole('radio', { name: /Tuteur entreprise/ }),
		).toBeTruthy()
	})
})
