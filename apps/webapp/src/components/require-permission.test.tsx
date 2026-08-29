import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen } from '@testing-library/react'
import type { ReactNode } from 'react'
import { describe, expect, it } from 'vitest'
import { RequirePermission } from '#/components/require-permission'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import { renderWithRouter } from '#/test/render-with-router'

const ORGANIZATION = { id: 'org-1', name: 'Dupont', slug: 'dupont' }

const MY_PERMISSIONS_PATH =
	'/api/v1/organizations/{organization_id}/members/me/permissions'

function installFakePermissionsApi(permissions: string[]) {
	const fakeApi = {
		get(path: string) {
			const queryKey = [{ _id: path }]
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						if (path === MY_PERMISSIONS_PATH) {
							return { data: { permissions }, pagination: null }
						}
						throw new Error(`unmocked GET ${path}`)
					},
				},
			}
		},
		mutation() {
			throw new Error('unmocked mutation')
		},
	}

	// biome-ignore lint/suspicious/noExplicitAny: test-only fake, shape matches TanstackQueryApiClient's used surface
	;(window as any).tanstackApi = fakeApi
}

async function renderGuarded(permission: string, permissions: string[]) {
	installFakePermissionsApi(permissions)
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false } },
	})

	function Providers({ children }: { children: ReactNode }) {
		return (
			<QueryClientProvider client={queryClient}>
				<OrganizationListProvider organizations={[ORGANIZATION]}>
					<ActiveOrganizationProvider activeOrganization={ORGANIZATION}>
						{children}
					</ActiveOrganizationProvider>
				</OrganizationListProvider>
			</QueryClientProvider>
		)
	}

	return renderWithRouter(
		<Providers>
			<p>sentinel</p>
			{/** biome-ignore lint/suspicious/noExplicitAny: PermissionName is a closed union; the test passes a plain string on purpose */}
			<RequirePermission permission={permission as any}>
				<button type="button">Nouveau client</button>
			</RequirePermission>
		</Providers>,
	)
}

describe('RequirePermission', () => {
	it('keeps the control hidden when the caller lacks the permission', async () => {
		await renderGuarded('MANAGE_CUSTOMERS', [])

		// Sync point: once the sentinel (unconditionally rendered) is still
		// there and the read has had a tick to settle, the gated button's
		// absence is the real assertion, not a race with the query.
		await screen.findByText('sentinel')
		expect(screen.queryByText('Nouveau client')).toBeNull()
	})

	it('shows the control once the caller holds the permission', async () => {
		await renderGuarded('MANAGE_CUSTOMERS', ['MANAGE_CUSTOMERS'])

		expect(await screen.findByText('Nouveau client')).toBeDefined()
	})
})
