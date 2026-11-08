import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { TooltipProvider } from '#/components/ui/tooltip'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import { resetRhStore } from '#/pages/hr/feature/mocks/rh-store'
import { renderWithRouter } from '#/test/render-with-router'
import { MemberDetailFeature } from './member-detail-feature'

const MEMBERS_PATH = '/api/v1/organizations/{organization_id}/members'
const ABSENCES_PATH = '/api/v1/organizations/{organization_id}/absences'
const PROFILES_PATH = '/api/v1/organizations/{organization_id}/member-profiles'
const PERMISSIONS_PATH =
	'/api/v1/organizations/{organization_id}/members/me/permissions'

const ORGANIZATION = {
	id: 'org-1',
	name: 'Atelier Bois & Co',
	slug: 'atelier-bois',
}

function member(id: string, firstName: string, lastName: string) {
	return {
		id,
		organization_id: 'org-1',
		last_name: lastName,
		first_name: firstName,
		display_name: `${lastName} ${firstName}`,
		account: null,
		joined_at: null,
		created_at: '2026-01-01T00:00:00Z',
	}
}

const MEMBERS = [
	member('u-1', 'Alix', 'Nova'),
	member('u-2', 'Bao', 'Tran'),
	member('u-3', 'Cleo', 'Dupont'),
]

type Profile = {
	member_id: string
	organization_id: string
	work_email: string | null
	personal_email: string | null
	mobile: string | null
	landline: string | null
	address: {
		street: string
		complement: string
		postal_code: string | null
		city: string
		country: string
	}
	job_title: string | null
	manager_member_id: string | null
	work_permit_end_date: string | null
	archived_at: string | null
}

function profile(memberId: string, patch: Partial<Profile> = {}): Profile {
	return {
		member_id: memberId,
		organization_id: 'org-1',
		work_email: `${memberId}@atelier.fr`,
		personal_email: null,
		mobile: null,
		landline: null,
		address: {
			street: '',
			complement: '',
			postal_code: null,
			city: '',
			country: 'France',
		},
		job_title: null,
		manager_member_id: null,
		work_permit_end_date: null,
		archived_at: null,
		...patch,
	}
}

interface Write {
	method: string
	path: string
	params: { path?: Record<string, string>; body?: Profile }
}

let profiles: Profile[]
let writes: Write[]
let refuse: { status: number; message: string } | null

function installFakeApi() {
	const fakeApi = {
		get(path: string, params: unknown) {
			const p = (params ?? {}) as { path?: unknown; query?: unknown }
			const queryKey = [{ _id: path, path: p.path, query: p.query }]
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						if (path === MEMBERS_PATH)
							return { data: MEMBERS, pagination: null }
						if (path === ABSENCES_PATH) return { data: [], pagination: null }
						if (path === PROFILES_PATH)
							return { data: structuredClone(profiles), pagination: null }
						if (path === PERMISSIONS_PATH)
							return {
								data: { permissions: ['MANAGE_MEMBERS'] },
								pagination: null,
							}
						throw new Error(`unmocked GET ${path}`)
					},
				},
			}
		},
		mutation(method: string, path: string) {
			const mutationKey = [{ method, path }]
			return {
				mutationKey,
				mutationOptions: {
					mutationKey,
					mutationFn: async (params: Write['params']) => {
						writes.push({ method, path, params })
						if (refuse && path.endsWith('/profile'))
							throw Object.assign(new Error(refuse.message), {
								status: refuse.status,
							})
						const id = params.path?.member_id ?? ''
						const others = profiles.filter((row) => row.member_id !== id)
						const current = profiles.find((row) => row.member_id === id)
						if (path.endsWith('/archive'))
							profiles = [
								...others,
								{
									...(current ?? profile(id)),
									archived_at: '2026-11-05T08:00:00Z',
								},
							]
						if (path.endsWith('/restore'))
							profiles = [
								...others,
								{ ...(current ?? profile(id)), archived_at: null },
							]
						if (path.endsWith('/profile') && params.body)
							profiles = [
								...others,
								{
									...(current ?? profile(id)),
									...params.body,
									member_id: id,
									organization_id: 'org-1',
								},
							]
						return { data: {} }
					},
				},
			}
		},
	}
	// biome-ignore lint/suspicious/noExplicitAny: test-only fake, shape matches TanstackQueryApiClient's used surface
	;(window as any).tanstackApi = fakeApi
}

function renderDetail(memberId: string) {
	installFakeApi()
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	return renderWithRouter(
		<QueryClientProvider client={queryClient}>
			<OrganizationListProvider organizations={[ORGANIZATION as never]}>
				<ActiveOrganizationProvider activeOrganization={ORGANIZATION as never}>
					<TooltipProvider>
						<MemberDetailFeature
							memberId={memberId}
							tab="identite"
							month={null}
							onChange={() => undefined}
						/>
					</TooltipProvider>
				</ActiveOrganizationProvider>
			</OrganizationListProvider>
		</QueryClientProvider>,
	)
}

describe('member profile bound to the API', () => {
	beforeEach(() => {
		resetRhStore()
		profiles = []
		writes = []
		refuse = null
	})

	afterEach(() => {
		resetRhStore()
	})

	it('spec_rh_1_5 shows the job title read from the API and defaults for a member without profile', async () => {
		profiles = [profile('u-2', { job_title: 'Chef de chantier' })]

		await renderDetail('u-2')
		expect(
			await screen.findByRole('heading', { name: 'Bao Tran' }),
		).toBeDefined()
		expect(await screen.findAllByText('Chef de chantier')).not.toHaveLength(0)
	})

	it('spec_rh_1_5 never shows the prototype job title for a member without profile', async () => {
		await renderDetail('u-1')

		expect(
			await screen.findByRole('heading', { name: 'Alix Nova' }),
		).toBeDefined()
		expect(screen.getByText('Poste non renseigné')).toBeDefined()
	})

	it('spec_rh_1_5 saves the identity form through the profile endpoint', async () => {
		const user = userEvent.setup()
		profiles = [profile('u-2')]
		await renderDetail('u-2')
		await screen.findByRole('heading', { name: 'Bao Tran' })

		await user.click(await screen.findByRole('button', { name: /Modifier/ }))
		const dialog = await screen.findByRole('dialog')
		await user.type(within(dialog).getByLabelText('Poste'), 'Menuisier')
		await user.type(within(dialog).getByLabelText('Mobile'), '06 12 34 56 78')
		await user.click(
			within(dialog).getByRole('button', { name: 'Enregistrer' }),
		)

		await waitFor(() =>
			expect(writes.some((write) => write.path.endsWith('/profile'))).toBe(
				true,
			),
		)
		const put = writes.find((write) => write.path.endsWith('/profile'))
		expect(put?.method).toBe('put')
		expect(put?.params.path).toEqual({
			organization_id: 'org-1',
			member_id: 'u-2',
		})
		expect(put?.params.body).toMatchObject({
			work_email: 'u-2@atelier.fr',
			personal_email: null,
			mobile: '06 12 34 56 78',
			job_title: 'Menuisier',
			manager_member_id: null,
		})
		await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull())
		expect(await screen.findAllByText('Menuisier')).not.toHaveLength(0)
	})

	it('spec_rh_1_5 keeps the dialog open and says why when the API refuses', async () => {
		const user = userEvent.setup()
		profiles = [profile('u-2')]
		refuse = { status: 422, message: 'Le mobile est invalide' }
		await renderDetail('u-2')
		await screen.findByRole('heading', { name: 'Bao Tran' })

		await user.click(await screen.findByRole('button', { name: /Modifier/ }))
		const dialog = await screen.findByRole('dialog')
		await user.type(within(dialog).getByLabelText('Poste'), 'Menuisier')
		await user.click(
			within(dialog).getByRole('button', { name: 'Enregistrer' }),
		)

		expect(
			await within(dialog).findByText('Le mobile est invalide'),
		).toBeDefined()
		expect(screen.queryByRole('dialog')).not.toBeNull()
	})

	it('spec_rh_1_5 archives and restores through the API, the badge follows the read', async () => {
		const user = userEvent.setup()
		profiles = [profile('u-2')]
		await renderDetail('u-2')
		await screen.findByRole('heading', { name: 'Bao Tran' })

		await user.click(await screen.findByRole('button', { name: /Archiver/ }))
		const dialog = await screen.findByRole('dialog')
		await user.click(within(dialog).getByRole('button', { name: 'Archiver' }))

		await waitFor(() =>
			expect(writes.some((write) => write.path.endsWith('/archive'))).toBe(
				true,
			),
		)
		expect(await screen.findByText('Archivé')).toBeDefined()

		const restore = await screen.findByRole('button', { name: /Réactiver/ })
		await waitFor(() => expect(restore.hasAttribute('disabled')).toBe(false))
		await user.click(restore)
		await waitFor(() =>
			expect(writes.some((write) => write.path.endsWith('/restore'))).toBe(
				true,
			),
		)
		await waitFor(() => expect(screen.queryByText('Archivé')).toBeNull())
	})
	it('spec_rh_1_5 moves the direct reports to the manager of the archived member', async () => {
		const user = userEvent.setup()
		profiles = [
			profile('u-1'),
			profile('u-2', { manager_member_id: 'u-1' }),
			profile('u-3', { manager_member_id: 'u-2' }),
		]
		await renderDetail('u-2')
		await screen.findByRole('heading', { name: 'Bao Tran' })

		await user.click(await screen.findByRole('button', { name: /Archiver/ }))
		const dialog = await screen.findByRole('dialog')
		await user.click(within(dialog).getByRole('button', { name: 'Archiver' }))

		await waitFor(() =>
			expect(
				writes.some(
					(write) =>
						write.path.endsWith('/profile') &&
						write.params.path?.member_id === 'u-3',
				),
			).toBe(true),
		)
		const moved = writes.find((write) => write.params.path?.member_id === 'u-3')
		expect(moved?.params.body?.manager_member_id).toBe('u-1')
		expect(writes[0]?.path.endsWith('/archive')).toBe(true)
	})
})
