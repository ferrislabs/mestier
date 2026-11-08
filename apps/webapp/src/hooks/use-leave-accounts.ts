import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'
import { isForbiddenError } from '#/lib/api-error'

const ACCOUNTS_PATH = '/api/v1/organizations/{organization_id}/leave-accounts'
const ACCOUNT_PATH =
	'/api/v1/organizations/{organization_id}/leave-accounts/{account_id}'
const ARCHIVE_PATH =
	'/api/v1/organizations/{organization_id}/leave-accounts/{account_id}/archive'
const RESTORE_PATH =
	'/api/v1/organizations/{organization_id}/leave-accounts/{account_id}/restore'

interface QueryKeyMeta {
	_id?: unknown
}

function queryKeyId(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as QueryKeyMeta)._id
		: undefined
}

function invalidateAccountReads(
	queryClient: ReturnType<typeof useQueryClient>,
) {
	return queryClient.invalidateQueries({
		predicate: (query) => queryKeyId(query.queryKey) === ACCOUNTS_PATH,
	})
}

export function useLeaveAccounts(organizationId: string, enabled = true) {
	return useQuery({
		...window.tanstackApi.get(ACCOUNTS_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
		enabled: enabled && Boolean(organizationId),
		retry: (failures: number, error: unknown) =>
			!isForbiddenError(error) && failures < 3,
	})
}

export function useCreateLeaveAccount() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', ACCOUNTS_PATH).mutationOptions,
		onSuccess: () => invalidateAccountReads(queryClient),
	})
}

export function useUpdateLeaveAccount() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('put', ACCOUNT_PATH).mutationOptions,
		onSuccess: () => invalidateAccountReads(queryClient),
	})
}

export function useArchiveLeaveAccount() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', ARCHIVE_PATH).mutationOptions,
		onSuccess: () => invalidateAccountReads(queryClient),
	})
}

export function useRestoreLeaveAccount() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', RESTORE_PATH).mutationOptions,
		onSuccess: () => invalidateAccountReads(queryClient),
	})
}

export type LeaveAccountRecord = Schemas.LeaveAccountResponse
export type LeaveAccountRequest = Schemas.LeaveAccountRequest
