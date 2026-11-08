import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'
import { isForbiddenError } from '#/lib/api-error'

const CONTRACTS_PATH =
	'/api/v1/organizations/{organization_id}/member-contracts'
const MEMBER_CONTRACTS_PATH =
	'/api/v1/organizations/{organization_id}/members/{member_id}/contracts'
const MEMBER_CONTRACT_PATH =
	'/api/v1/organizations/{organization_id}/members/{member_id}/contracts/{contract_id}'

interface QueryKeyMeta {
	_id?: unknown
}

function queryKeyId(queryKey: readonly unknown[]) {
	const meta = queryKey[0]
	return typeof meta === 'object' && meta !== null
		? (meta as QueryKeyMeta)._id
		: undefined
}

function invalidateContractReads(
	queryClient: ReturnType<typeof useQueryClient>,
) {
	return queryClient.invalidateQueries({
		predicate: (query) => {
			const id = queryKeyId(query.queryKey)
			return id === CONTRACTS_PATH || id === MEMBER_CONTRACTS_PATH
		},
	})
}

export function useMemberContracts(organizationId: string, enabled = true) {
	return useQuery({
		...window.tanstackApi.get(CONTRACTS_PATH, {
			path: { organization_id: organizationId },
		}).queryOptions,
		enabled: enabled && Boolean(organizationId),
		retry: (failures: number, error: unknown) =>
			!isForbiddenError(error) && failures < 3,
	})
}

export function useCreateMemberContract() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('post', MEMBER_CONTRACTS_PATH)
			.mutationOptions,
		onSuccess: () => invalidateContractReads(queryClient),
	})
}

export function useUpdateMemberContract() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('put', MEMBER_CONTRACT_PATH).mutationOptions,
		onSuccess: () => invalidateContractReads(queryClient),
	})
}

export function useDeleteMemberContract() {
	const queryClient = useQueryClient()

	return useMutation({
		...window.tanstackApi.mutation('delete', MEMBER_CONTRACT_PATH)
			.mutationOptions,
		onSuccess: () => invalidateContractReads(queryClient),
	})
}

export type MemberContract = Schemas.MemberContractResponse
export type PutMemberContractRequest = Schemas.PutMemberContractRequest
