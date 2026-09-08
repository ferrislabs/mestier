import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { type RenderResult, render } from '@testing-library/react'
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

/**
 * Wraps `ui` with the query/organization context `RequirePermission` needs,
 * composable with `renderWithRouter` for pages that also need routing.
 *
 * Seeds the permissions query up front rather than relying on the fake
 * fetch resolving in time: a gated control mounted behind a closed Radix
 * dropdown/dialog may never trigger that fetch during a given test at all,
 * which would otherwise leave the caller's permission check permanently
 * `false` instead of merely delayed.
 */
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
	seedPermissionsCache(queryClient, { permissions })

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
 * `RequirePermission` hides its children until the permissions read
 * resolves — but that read may never even start in this render (e.g. a
 * gated control inside a closed Radix dropdown, never opened by a given
 * test), so waiting reactively for the query to appear can wait forever.
 * `seedPermissionsCache` sidesteps that entirely: the answer is already in
 * the cache before the first render, so there is nothing to wait for.
 */
export function renderWithPermissions(
	ui: ReactElement,
	options?: WithPermissionsOptions,
): RenderResult {
	return render(wrapWithPermissions(ui, options))
}

/**
 * Pre-populates the permissions query this fake's own key shape (`[{ _id:
 * path }]`, see `installFakePermissionsApi`) resolves to, so `usePermissions`
 * reads a cache hit — `isSuccess: true` — on its very first render instead
 * of racing a fetch that a given test's render tree might not even trigger.
 */
export function seedPermissionsCache(
	queryClient: QueryClient,
	{ permissions = ALL_PERMISSIONS }: { permissions?: string[] } = {},
) {
	queryClient.setQueryData([{ _id: MY_PERMISSIONS_PATH }], {
		data: { permissions },
		pagination: null,
	})
}

/**
 * Same idea as {@link seedPermissionsCache}, for test files with their own
 * richer fake `window.tanstackApi` (query key `[{ _id, path, query }]`,
 * matching real params) rather than this module's own simplified one —
 * `task-sheet-feature.test.tsx` and friends. `organizationId` must match
 * the `Organization` those tests render with, since it is part of the key
 * `usePermissions` actually requests.
 */
export function seedPermissionsCacheForOrganization(
	queryClient: QueryClient,
	organizationId: string,
	permissions: string[] = ALL_PERMISSIONS,
) {
	queryClient.setQueryData(
		[{ _id: MY_PERMISSIONS_PATH, path: { organization_id: organizationId } }],
		{ data: { permissions }, pagination: null },
	)
}
