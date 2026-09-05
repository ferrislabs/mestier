import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { type RenderResult, render, waitFor } from '@testing-library/react'
import type { ReactElement, ReactNode } from 'react'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { PERMISSION_CATALOG } from '#/lib/permission-catalog'

const MY_PERMISSIONS_PATH =
	'/api/v1/organizations/{organization_id}/members/me/permissions'

const ALL_PERMISSIONS = PERMISSION_CATALOG.map((entry) => entry.name)

const DEFAULT_ORGANIZATION: Organization = {
	id: 'org-1',
	name: 'Organisation',
	slug: 'organisation',
	owner_id: 'user-1',
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
	field_clock_enabled: false,
	vat_on_debits: false,
	missing_legal_identity_fields: [],
}

/**
 * Any component gated by `RequirePermission`/`useHasPermission` reads
 * `window.tanstackApi` for the caller's permissions — a render that skips
 * this throws `useActiveOrganization must be used inside
 * ActiveOrganizationProvider` well before the component under test gets a
 * chance to run, taking every assertion in the file down with it.
 */
export function installFakePermissionsApi(permissions: string[]) {
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

export interface WithPermissionsOptions {
	/** Defaults to every named bit — most tests predate permission gating
	 * and assert on a fully-privileged caller; pass a narrower list only to
	 * exercise the gate itself. */
	permissions?: string[]
	organization?: Organization
	/** Supplied by `renderWithPermissions` so it can wait on the same
	 * client afterwards; build your own only when composing with another
	 * render helper (e.g. `renderWithRouter`) that needs to own it. */
	queryClient?: QueryClient
}

/** Wraps `ui` with the query/organization context `RequirePermission` needs,
 * composable with `renderWithRouter` for pages that also need routing. */
export function wrapWithPermissions(
	ui: ReactNode,
	{
		permissions = ALL_PERMISSIONS,
		organization = DEFAULT_ORGANIZATION,
		queryClient = new QueryClient({
			defaultOptions: { queries: { retry: false } },
		}),
	}: WithPermissionsOptions = {},
): ReactElement {
	installFakePermissionsApi(permissions)

	return (
		<QueryClientProvider client={queryClient}>
			<OrganizationListProvider organizations={[organization]}>
				<ActiveOrganizationProvider activeOrganization={organization}>
					{ui}
				</ActiveOrganizationProvider>
			</OrganizationListProvider>
		</QueryClientProvider>
	)
}

/**
 * Drop-in replacement for `@testing-library/react`'s `render`, for
 * components that don't also need router context.
 *
 * Awaits the permissions read the same way `renderWithRouter` awaits the
 * router reaching `idle` — `RequirePermission` hides its children until
 * that fetch resolves, so a caller asserting on a gated button right after
 * a synchronous `render()` would race it and find nothing.
 */
export async function renderWithPermissions(
	ui: ReactElement,
	options?: WithPermissionsOptions,
): Promise<RenderResult> {
	const queryClient =
		options?.queryClient ??
		new QueryClient({ defaultOptions: { queries: { retry: false } } })
	const result = render(wrapWithPermissions(ui, { ...options, queryClient }))
	await waitFor(() => {
		if (queryClient.isFetching() > 0)
			throw new Error('permissions still loading')
	})
	return result
}
