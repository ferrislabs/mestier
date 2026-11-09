import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeAll, describe, expect, it, vi } from 'vitest'
import { UserMenu } from '#/components/user-menu'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import {
	installFakeTanstackApi,
	installRadixPolyfills,
	ORGANIZATION,
} from '#/pages/support/feature/support-test-kit'
import { renderWithRouter } from '#/test/render-with-router'

vi.mock('react-oidc-context', () => ({
	useAuth: () => ({
		user: { profile: { name: 'Lea Martin', email: 'lea@example.fr' } },
		signoutRedirect: vi.fn(),
	}),
}))

const SPACES_PATH = '/api/v1/cursus/me/spaces'

beforeAll(installRadixPolyfills)

async function openMenu(spaces: unknown[] | Error) {
	const api = installFakeTanstackApi()
	api.mockGet(SPACES_PATH, () => {
		if (spaces instanceof Error) throw spaces
		return { data: spaces }
	})
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false } },
	})
	await renderWithRouter(
		<QueryClientProvider client={queryClient}>
			<OrganizationListProvider organizations={[ORGANIZATION]}>
				<ActiveOrganizationProvider activeOrganization={ORGANIZATION}>
					<UserMenu />
				</ActiveOrganizationProvider>
			</OrganizationListProvider>
		</QueryClientProvider>,
	)
	await waitFor(() => expect(queryClient.isFetching()).toBe(0))
	await userEvent
		.setup()
		.click(screen.getByRole('button', { name: 'Menu du compte' }))
}

const PARTICIPANT_SPACE = {
	organization_id: 'org-9',
	slug: 'ascor',
	name: 'Ascor',
	is_member: false,
}

describe('UserMenu portal entry', { timeout: 15_000 }, () => {
	it('links to the portal when a participant space exists', async () => {
		await openMenu([PARTICIPANT_SPACE])

		const link = await screen.findByRole('menuitem', {
			name: /Mon portail Cursus/,
		})
		expect(link.getAttribute('href')).toBe('/portail')
	})

	it('hides the entry when every space is one of a member', async () => {
		await openMenu([{ ...PARTICIPANT_SPACE, is_member: true }])

		await screen.findByRole('menuitem', { name: /Se déconnecter/ })
		expect(
			screen.queryByRole('menuitem', { name: /Mon portail Cursus/ }),
		).toBeNull()
	})

	it('hides the entry when the read fails', async () => {
		await openMenu(new Error('boom'))

		await screen.findByRole('menuitem', { name: /Se déconnecter/ })
		expect(
			screen.queryByRole('menuitem', { name: /Mon portail Cursus/ }),
		).toBeNull()
	})
})
