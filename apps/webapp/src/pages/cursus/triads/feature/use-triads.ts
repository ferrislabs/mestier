import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from '#/api/api.client'
import { useUploadFile } from '#/hooks/use-customers'
import { invalidateCursus } from '#/pages/cursus/shared/feature/cursus-queries'

const TRIADS_PATH = '/api/v1/organizations/{organization_id}/cursus/triads'
const TRIAD_PATH =
	'/api/v1/organizations/{organization_id}/cursus/triads/{triad_id}'
const TRIAD_CLOSE_PATH =
	'/api/v1/organizations/{organization_id}/cursus/triads/{triad_id}/close'
const TRIAD_CONTRACTS_PATH =
	'/api/v1/organizations/{organization_id}/cursus/triads/{triad_id}/contracts'
const CONTRACT_PATH =
	'/api/v1/organizations/{organization_id}/cursus/contracts/{contract_id}'
const CONTRACT_CLOSE_PATH =
	'/api/v1/organizations/{organization_id}/cursus/contracts/{contract_id}/close'
const CONTRACT_TUTORS_PATH =
	'/api/v1/organizations/{organization_id}/cursus/contracts/{contract_id}/tutors'
const CONTRACT_FILE_PATH =
	'/api/v1/organizations/{organization_id}/cursus/contracts/{contract_id}/files/{kind}'

export type ContractFileKind = 'cerfa' | 'mission_sheet'

export interface TriadFilters {
	promotionId: string | undefined
	closed: boolean | undefined
}

export function useTriads(
	organizationId: string,
	filters: TriadFilters,
	enabled = true,
) {
	return useQuery({
		...window.tanstackApi.get(TRIADS_PATH, {
			path: { organization_id: organizationId },
			query: {
				page: 1,
				per_page: 100,
				...(filters.promotionId ? { promotion_id: filters.promotionId } : {}),
				...(filters.closed === undefined ? {} : { closed: filters.closed }),
			},
		}).queryOptions,
		enabled,
	})
}

export function useTriad(organizationId: string, triadId: string) {
	return useQuery(
		window.tanstackApi.get(TRIAD_PATH, {
			path: { organization_id: organizationId, triad_id: triadId },
		}).queryOptions,
	)
}

export function useCreateTriad() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('post', TRIADS_PATH).mutationOptions,
		onSuccess: () => invalidateCursus(queryClient),
	})
}

export function useUpdateTriad() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('patch', TRIAD_PATH).mutationOptions,
		onSuccess: () => invalidateCursus(queryClient),
	})
}

export function useCloseTriad() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('post', TRIAD_CLOSE_PATH).mutationOptions,
		onSuccess: () => invalidateCursus(queryClient),
	})
}

export function useCreateContract() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('post', TRIAD_CONTRACTS_PATH)
			.mutationOptions,
		onSuccess: () => invalidateCursus(queryClient),
	})
}

export function useUpdateContract() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('patch', CONTRACT_PATH).mutationOptions,
		onSuccess: () => invalidateCursus(queryClient),
	})
}

export function useCloseContract() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('post', CONTRACT_CLOSE_PATH).mutationOptions,
		onSuccess: () => invalidateCursus(queryClient),
	})
}

export function useSetContractTutors() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('put', CONTRACT_TUTORS_PATH).mutationOptions,
		onSuccess: () => invalidateCursus(queryClient),
	})
}

export function useSetContractFile() {
	const queryClient = useQueryClient()
	return useMutation({
		...window.tanstackApi.mutation('put', CONTRACT_FILE_PATH).mutationOptions,
		onSuccess: () => invalidateCursus(queryClient),
	})
}

export function useAttachContractFile(organizationId: string) {
	const upload = useUploadFile()
	const setFile = useSetContractFile()

	return {
		isPending: upload.isPending || setFile.isPending,
		error: upload.error ?? setFile.error,
		attach: async (contractId: string, kind: ContractFileKind, file: File) => {
			const uploaded = await upload.mutateAsync(file)
			await setFile.mutateAsync({
				path: {
					organization_id: organizationId,
					contract_id: contractId,
					kind,
				},
				body: { file_key: uploaded.data.key },
			})
		},
	}
}

export function useOpenContractFile(organizationId: string) {
	const queryClient = useQueryClient()
	return async (contractId: string, kind: ContractFileKind) => {
		const response = await queryClient.fetchQuery({
			...window.tanstackApi.get(CONTRACT_FILE_PATH, {
				path: {
					organization_id: organizationId,
					contract_id: contractId,
					kind,
				},
			}).queryOptions,
			staleTime: 0,
		})
		window.open(response.data.url, '_blank', 'noopener,noreferrer')
	}
}

export type TriadSummary = Schemas.TriadSummaryResponse
export type TriadDetail = Schemas.TriadDetailResponse
export type Contract = Schemas.ContractResponse
