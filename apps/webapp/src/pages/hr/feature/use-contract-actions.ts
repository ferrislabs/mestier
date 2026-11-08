import { useMemo } from 'react'
import { useUploadFile } from '#/hooks/use-customers'
import {
	useCreateMemberContract,
	useDeleteMemberContract,
	useUpdateMemberContract,
} from '#/hooks/use-member-contracts'
import {
	useAddMemberDocument,
	useDeleteMemberDocument,
} from '#/hooks/use-member-documents'
import {
	contractRequestOf,
	documentRequestOf,
} from '#/pages/hr/lib/rh/member-wire'
import type {
	RhContractActions,
	RhDocumentActions,
} from '#/pages/hr/ui/rh/shared/rh-context'

export function useContractActions(organizationId: string): RhContractActions {
	const create = useCreateMemberContract()
	const update = useUpdateMemberContract()
	const remove = useDeleteMemberContract()
	const createAsync = create.mutateAsync
	const updateAsync = update.mutateAsync
	const removeAsync = remove.mutateAsync

	return useMemo(() => {
		const path = (memberId: string) => ({
			organization_id: organizationId,
			member_id: memberId,
		})
		return {
			add: async (memberId, draft) => {
				await createAsync({
					path: path(memberId),
					body: contractRequestOf(draft),
				})
			},
			update: async (memberId, contractId, draft) => {
				await updateAsync({
					path: { ...path(memberId), contract_id: contractId },
					body: contractRequestOf(draft),
				})
			},
			remove: async (memberId, contractId) => {
				await removeAsync({
					path: { ...path(memberId), contract_id: contractId },
				})
			},
		}
	}, [organizationId, createAsync, updateAsync, removeAsync])
}

export function useDocumentActions(organizationId: string): RhDocumentActions {
	const upload = useUploadFile()
	const add = useAddMemberDocument()
	const remove = useDeleteMemberDocument()
	const uploadAsync = upload.mutateAsync
	const addAsync = add.mutateAsync
	const removeAsync = remove.mutateAsync

	return useMemo(() => {
		const path = (memberId: string) => ({
			organization_id: organizationId,
			member_id: memberId,
		})
		return {
			add: async (memberId, file, kind) => {
				const uploaded = await uploadAsync(file)
				await addAsync({
					path: path(memberId),
					body: documentRequestOf(file.name, kind, uploaded.data.key),
				})
			},
			remove: async (memberId, documentId) => {
				await removeAsync({
					path: { ...path(memberId), document_id: documentId },
				})
			},
		}
	}, [organizationId, uploadAsync, addAsync, removeAsync])
}
