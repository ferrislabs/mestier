import { useQuery } from '@tanstack/react-query'
import { useActiveOrganization } from '#/hooks/use-active-organization'

const MY_PERMISSIONS_PATH =
	'/api/v1/organizations/{organization_id}/members/me/permissions'

/**
 * Every named bit the backend currently knows (`mestier_core::domain::role::
 * Permissions::NAMED`) — kept as a literal union rather than a bare
 * `string` so a call site asking for a bit that does not exist is a
 * type error, not a screen that silently never unlocks.
 */
export type PermissionName =
	| 'MANAGE_ORG'
	| 'MANAGE_MEMBERS'
	| 'MANAGE_ROLES'
	| 'MANAGE_CHANNELS'
	| 'MANAGE_WEBHOOKS'
	| 'VIEW_CHANNEL'
	| 'SEND_MESSAGES'
	| 'VIEW_PLANNING'
	| 'MANAGE_PLANNING'
	| 'VIEW_COST'
	| 'MANAGE_COST'
	| 'VIEW_REPORTS'
	| 'MANAGE_CUSTOMERS'
	| 'MANAGE_QUOTES'
	| 'MANAGE_REFERENCE'
	| 'VIEW_CUSTOMERS'
	| 'VIEW_INVOICES'
	| 'MANAGE_INVOICES'

/**
 * The caller's own granted bits in the active organization (#307) —
 * loaded once per organization and cached, the same way
 * `useActiveOrganization` itself is: no organization id to thread
 * through, so this is callable from anywhere already inside
 * `ActiveOrganizationProvider`.
 *
 * Presentational only. Hiding a control here is not the security
 * boundary — the API refuses and redacts on its own account regardless
 * of what this hook returns; a caller whose bit is revoked between this
 * query's last fetch and their click is still refused server-side, this
 * only makes that refusal rare.
 */
export function usePermissions() {
	const { activeOrganizationId } = useActiveOrganization()

	return useQuery({
		...window.tanstackApi.get(MY_PERMISSIONS_PATH, {
			path: { organization_id: activeOrganizationId },
		}).queryOptions,
		// Permissions change on the rare occasion somebody edits a role, not
		// on every navigation — a longer stale time avoids a refetch (and the
		// flash of hidden-then-shown controls it would cause) on every route
		// change within the same organization.
		staleTime: 5 * 60 * 1000,
	})
}

/**
 * Whether the caller holds one bit, right now, in the active organization.
 * `false` while the permissions read is still loading or failed — a
 * control that has not yet confirmed a grant stays hidden rather than
 * flashing into view and back out, since hiding is presentation, not a
 * promise of what the API will accept.
 */
export function useHasPermission(permission: PermissionName): boolean {
	const { data, isSuccess } = usePermissions()
	if (!isSuccess) return false
	return data.data.permissions.includes(permission)
}
