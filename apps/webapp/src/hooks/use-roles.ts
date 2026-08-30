import type { QueryClient } from '@tanstack/react-query'
import {
	useMutation,
	useQueries,
	useQuery,
	useQueryClient,
} from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'

const ROLES_PATH = '/api/v1/organizations/{organization_id}/roles'
const ROLE_PATH = '/api/v1/roles/{role_id}'
const MEMBER_ROLES_PATH = '/api/v1/members/{member_id}/roles'
const MEMBER_ROLE_PATH = '/api/v1/members/{member_id}/roles/{role_id}'

interface QueryKeyMeta {
	_id?: unknown
	path?: {
		organization_id?: unknown
		member_id?: unknown
	}
}

function queryKeyMeta(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as QueryKeyMeta)
		: null
}

function isRolesListQuery(
	queryKey: readonly unknown[],
	organizationId?: string,
) {
	const meta = queryKeyMeta(queryKey)
	return (
		meta?._id === ROLES_PATH &&
		(!organizationId || meta.path?.organization_id === organizationId)
	)
}

function isMemberRolesQuery(queryKey: readonly unknown[], memberId?: string) {
	const meta = queryKeyMeta(queryKey)
	return (
		meta?._id === MEMBER_ROLES_PATH &&
		(!memberId || meta.path?.member_id === memberId)
	)
}

/** Invalidates every organization's roles list — a role edit doesn't carry
 * its organization id back on every mutation response, so this widens
 * rather than narrows; there is at most one active organization's list
 * mounted at a time in practice. */
function invalidateRolesLists(queryClient: QueryClient) {
	return queryClient.invalidateQueries({
		predicate: (query) => isRolesListQuery(query.queryKey),
	})
}

/**
 * An organization's roles, with their permissions (#308). Gated on
 * `MANAGE_ROLES` server-side — a caller without it gets a 403, not an
 * empty list, so this is only ever mounted behind `RequirePermission`.
 */
export function useRoles(organizationId: string, enabled = true) {
	return useQuery({
		...window.tanstackApi.get(ROLES_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
		enabled: enabled && Boolean(organizationId),
	})
}

export function useCreateRole(organizationId: string) {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', ROLES_PATH).mutationOptions,
		onSuccess: () => invalidateRolesLists(queryClient),
		meta: { organizationId },
	})
}

/**
 * `PATCH /roles/{id}` — full replacement, not a patch: the editor always
 * posts back the complete name and permission set it read. Renaming a
 * seeded role (`owner`/`admin`/`member`) is refused server-side; its
 * permissions stay editable.
 */
export function useUpdateRole() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('patch', ROLE_PATH).mutationOptions,
		onSuccess: () => invalidateRolesLists(queryClient),
	})
}

/**
 * `DELETE /roles/{id}` — refused server-side for a seeded role, or one
 * still assigned to a member (see `RoleService::delete_role`'s own doc):
 * a delete never silently unassigns.
 */
export function useDeleteRole() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('delete', ROLE_PATH).mutationOptions,
		onSuccess: () => invalidateRolesLists(queryClient),
	})
}

/** The role ids a member holds (#308) — ids only; resolve a name against
 * the organization's own role list (`useRoles`). */
export function useMemberRoleIds(memberId: string, enabled = true) {
	return useQuery({
		...window.tanstackApi.get(MEMBER_ROLES_PATH, {
			path: { member_id: memberId },
		}).queryOptions,
		enabled: enabled && Boolean(memberId),
	})
}

/** `POST /members/{id}/roles` — additive, not a replace: a member can hold
 * more than one role, and this never removes an existing one. */
export function useAssignRole() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', MEMBER_ROLES_PATH).mutationOptions,
		onSuccess: (_data, variables) =>
			queryClient.invalidateQueries({
				predicate: (query) =>
					isMemberRolesQuery(
						query.queryKey,
						(variables as { path?: { member_id?: string } }).path?.member_id,
					),
			}),
	})
}

/** `DELETE /members/{id}/roles/{id}` — the counterpart of `useAssignRole`;
 * removes exactly the one role, leaving any other the member holds. */
export function useUnassignRole() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('delete', MEMBER_ROLE_PATH).mutationOptions,
		onSuccess: (_data, variables) =>
			queryClient.invalidateQueries({
				predicate: (query) =>
					isMemberRolesQuery(
						query.queryKey,
						(variables as { path?: { member_id?: string } }).path?.member_id,
					),
			}),
	})
}

/**
 * How many members hold each role, keyed by role id. One `useMemberRoleIds`
 * query per member rather than a dedicated aggregate endpoint — the roles
 * section is a small-team settings screen, not a list that scales past a
 * couple dozen rows, and this reuses the same cached per-member query
 * `MemberRoleCell` already populates on the team page.
 */
export function useRoleMemberCounts(memberIds: string[]): Map<string, number> {
	return useQueries({
		queries: memberIds.map((memberId) => ({
			...window.tanstackApi.get(MEMBER_ROLES_PATH, {
				path: { member_id: memberId },
			}).queryOptions,
		})),
		combine: (results) => {
			const counts = new Map<string, number>()
			for (const result of results) {
				const roleIds = (
					result.data as { data?: { role_ids?: string[] } } | undefined
				)?.data?.role_ids
				if (!roleIds) continue
				for (const roleId of roleIds) {
					counts.set(roleId, (counts.get(roleId) ?? 0) + 1)
				}
			}
			return counts
		},
	})
}

export type Role = Schemas.RoleResponse
