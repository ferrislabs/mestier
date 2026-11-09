import { render, screen, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { PortalInvitationFeature } from '#/pages/cursus/portal/feature/portal-invitation-feature'
import { PortalSpaceFeature } from '#/pages/cursus/portal/feature/portal-space-feature'
import { PortalSpacesFeature } from '#/pages/cursus/portal/feature/portal-spaces-feature'
import { PortalTriadFeature } from '#/pages/cursus/portal/feature/portal-triad-feature'
import {
	callsTo,
	createQueryClient,
	installFakeTanstackApi,
	Providers,
} from '#/pages/support/feature/support-test-kit'

const navigate = vi.fn()

vi.mock('@tanstack/react-router', async (importOriginal) => ({
	...(await importOriginal<typeof import('@tanstack/react-router')>()),
	useNavigate: () => navigate,
}))

const SPACES_PATH = '/api/v1/cursus/me/spaces'
const TRIADS_PATH = '/api/v1/organizations/{organization_id}/cursus/me/triads'
const TRIAD_PATH =
	'/api/v1/organizations/{organization_id}/cursus/me/triads/{triad_id}'
const ACCEPT_PATH = '/api/v1/cursus/invitations/{token}/accept'

const CENTRE = {
	organization_id: 'org-9',
	slug: 'ascor',
	name: 'Ascor',
	is_member: false,
}
const OWN = {
	organization_id: 'org-1',
	slug: 'atelier-bois',
	name: 'Atelier Bois',
	is_member: true,
}

function renderWithProviders(ui: React.ReactNode) {
	return render(<Providers queryClient={createQueryClient([])}>{ui}</Providers>)
}

function httpError(status: number, message: string) {
	return Object.assign(new Error(message), { status })
}

beforeEach(() => {
	navigate.mockReset()
})

describe('PortalSpacesFeature', () => {
	it('shows the empty state when there is no space', async () => {
		const api = installFakeTanstackApi()
		api.mockGet(SPACES_PATH, () => ({ data: [] }))
		renderWithProviders(<PortalSpacesFeature />)

		expect(await screen.findByText('Aucun espace Cursus')).toBeDefined()
		expect(screen.getByText(/lien d’invitation/)).toBeDefined()
	})

	it('redirects to the only participant space', async () => {
		const api = installFakeTanstackApi()
		api.mockGet(SPACES_PATH, () => ({ data: [CENTRE] }))
		renderWithProviders(<PortalSpacesFeature />)

		await waitFor(() =>
			expect(navigate).toHaveBeenCalledWith({
				to: '/portail/$organizationSlug',
				params: { organizationSlug: 'ascor' },
				replace: true,
			}),
		)
	})

	it('lists several spaces without redirecting', async () => {
		const api = installFakeTanstackApi()
		api.mockGet(SPACES_PATH, () => ({ data: [CENTRE, OWN] }))
		renderWithProviders(<PortalSpacesFeature />)

		expect(await screen.findByText('Ascor')).toBeDefined()
		expect(screen.getByText('Atelier Bois')).toBeDefined()
		expect(navigate).not.toHaveBeenCalled()
	})

	it('does not redirect a lone member space', async () => {
		const api = installFakeTanstackApi()
		api.mockGet(SPACES_PATH, () => ({ data: [OWN] }))
		renderWithProviders(<PortalSpacesFeature />)

		expect(await screen.findByText('Atelier Bois')).toBeDefined()
		expect(navigate).not.toHaveBeenCalled()
	})
})

describe('PortalSpaceFeature', () => {
	it('redirects a member to Mes trinômes (@spec-cur-14)', async () => {
		const api = installFakeTanstackApi()
		api.mockGet(SPACES_PATH, () => ({ data: [OWN] }))
		api.mockGet(TRIADS_PATH, () => ({ data: [] }))
		renderWithProviders(<PortalSpaceFeature organizationSlug="atelier-bois" />)

		await waitFor(() =>
			expect(navigate).toHaveBeenCalledWith({
				to: '/o/$organizationSlug/cursus/mes-trinomes',
				params: { organizationSlug: 'atelier-bois' },
				replace: true,
			}),
		)
		expect(callsTo(api, 'get', TRIADS_PATH)).toHaveLength(0)
	})

	it('lists the triads of a participant space with the read-only badge', async () => {
		const api = installFakeTanstackApi()
		api.mockGet(SPACES_PATH, () => ({ data: [CENTRE] }))
		api.mockGet(TRIADS_PATH, () => ({
			data: [
				{
					triad_id: 'triad-1',
					apprentice_name: 'Lea Martin',
					promotion_label: 'CAP 2026',
					training_name: 'CAP Menuisier',
					role: 'apprentice',
					access: { kind: 'read_only', since: '2026-07-01T00:00:00Z' },
				},
			],
		}))
		renderWithProviders(<PortalSpaceFeature organizationSlug="ascor" />)

		expect(await screen.findByText('Lea Martin')).toBeDefined()
		expect(screen.getByText('Historique, lecture seule')).toBeDefined()
		expect(navigate).not.toHaveBeenCalled()
		expect(callsTo(api, 'get', TRIADS_PATH)[0]?.params).toEqual({
			path: { organization_id: 'org-9' },
		})
	})

	it('says so when the space is unknown', async () => {
		const api = installFakeTanstackApi()
		api.mockGet(SPACES_PATH, () => ({ data: [CENTRE] }))
		renderWithProviders(<PortalSpaceFeature organizationSlug="autre" />)

		expect(await screen.findByText('Espace introuvable')).toBeDefined()
	})
})

describe('PortalTriadFeature', () => {
	it('shows the not found state on a 404', async () => {
		const api = installFakeTanstackApi()
		api.mockGet(SPACES_PATH, () => ({ data: [CENTRE] }))
		api.mockGet(TRIAD_PATH, () => {
			throw httpError(404, 'not found')
		})
		renderWithProviders(
			<PortalTriadFeature organizationSlug="ascor" triadId="triad-x" />,
		)

		expect(await screen.findByText('Trinôme introuvable')).toBeDefined()
	})
})

describe('PortalInvitationFeature', () => {
	it('accepts the token and opens the space', async () => {
		const api = installFakeTanstackApi()
		api.mockGet(SPACES_PATH, () => ({ data: [CENTRE] }))
		api.mockMutation('post', ACCEPT_PATH, () => ({ data: CENTRE }))
		renderWithProviders(<PortalInvitationFeature token="tok-1" />)

		await waitFor(() =>
			expect(navigate).toHaveBeenCalledWith({
				to: '/portail/$organizationSlug',
				params: { organizationSlug: 'ascor' },
				replace: true,
			}),
		)
		expect(callsTo(api, 'post', ACCEPT_PATH)).toHaveLength(1)
		expect(callsTo(api, 'post', ACCEPT_PATH)[0]?.params).toEqual({
			path: { token: 'tok-1' },
		})
	})

	it('goes to the console when the person is also a member', async () => {
		const api = installFakeTanstackApi()
		api.mockMutation('post', ACCEPT_PATH, () => ({ data: OWN }))
		renderWithProviders(<PortalInvitationFeature token="tok-2" />)

		await waitFor(() =>
			expect(navigate).toHaveBeenCalledWith({
				to: '/o/$organizationSlug/cursus/mes-trinomes',
				params: { organizationSlug: 'atelier-bois' },
				replace: true,
			}),
		)
	})

	it('explains an expired link on a 410', async () => {
		const api = installFakeTanstackApi()
		api.mockMutation('post', ACCEPT_PATH, () => {
			throw httpError(410, 'gone')
		})
		renderWithProviders(<PortalInvitationFeature token="tok-3" />)

		expect(
			await screen.findByText(
				'Ce lien n’est plus valide, demandez un nouveau lien à votre centre de formation.',
			),
		).toBeDefined()
		expect(navigate).not.toHaveBeenCalled()
	})

	it('shows the API message on a 409', async () => {
		const api = installFakeTanstackApi()
		api.mockMutation('post', ACCEPT_PATH, () => {
			throw httpError(409, 'Cette invitation est liée à un autre compte.')
		})
		renderWithProviders(<PortalInvitationFeature token="tok-4" />)

		expect(
			await screen.findByText('Cette invitation est liée à un autre compte.'),
		).toBeDefined()
	})
})
