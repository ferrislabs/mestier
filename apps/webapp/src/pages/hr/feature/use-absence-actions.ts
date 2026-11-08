import { useMemo } from 'react'
import {
	useArchiveAbsenceType,
	useCreateAbsenceType,
	useRestoreAbsenceType,
	useUpdateAbsenceType,
} from '#/hooks/use-absence-types'
import {
	useApproveAbsence,
	useCancelAbsence,
	useCreateAbsence,
	useRejectAbsence,
} from '#/hooks/use-absences'
import { useUploadFile } from '#/hooks/use-customers'
import { setTypeAccount } from '#/pages/hr/feature/mocks/rh-store'
import {
	absenceRequestOf,
	absenceTypeRequestOf,
} from '#/pages/hr/lib/rh/absence-wire'
import type {
	RhAbsenceActions,
	RhAbsenceTypeActions,
} from '#/pages/hr/ui/rh/shared/rh-context'

export function useAbsenceActions(organizationId: string): RhAbsenceActions {
	const upload = useUploadFile()
	const create = useCreateAbsence()
	const approve = useApproveAbsence()
	const reject = useRejectAbsence()
	const cancel = useCancelAbsence()
	const uploadAsync = upload.mutateAsync
	const createAsync = create.mutateAsync
	const approveAsync = approve.mutateAsync
	const rejectAsync = reject.mutateAsync
	const cancelAsync = cancel.mutateAsync

	return useMemo(() => {
		const path = (absenceId: string) => ({
			organization_id: organizationId,
			absence_id: absenceId,
		})
		return {
			create: async (draft, proof) => {
				const uploaded = proof ? await uploadAsync(proof) : null
				await createAsync({
					path: { organization_id: organizationId },
					body: absenceRequestOf(draft, uploaded?.data.key ?? null),
				})
			},
			approve: async (absenceId) => {
				await approveAsync({ path: path(absenceId) })
			},
			reject: async (absenceId, comment) => {
				await rejectAsync({
					path: path(absenceId),
					body: { comment: comment.trim() },
				})
			},
			cancel: async (absenceId) => {
				await cancelAsync({ path: path(absenceId) })
			},
		}
	}, [
		organizationId,
		uploadAsync,
		createAsync,
		approveAsync,
		rejectAsync,
		cancelAsync,
	])
}

export function useAbsenceTypeActions(
	organizationId: string,
): RhAbsenceTypeActions {
	const create = useCreateAbsenceType()
	const update = useUpdateAbsenceType()
	const archive = useArchiveAbsenceType()
	const restore = useRestoreAbsenceType()
	const createAsync = create.mutateAsync
	const updateAsync = update.mutateAsync
	const archiveAsync = archive.mutateAsync
	const restoreAsync = restore.mutateAsync

	return useMemo(() => {
		const path = (typeId: string) => ({
			organization_id: organizationId,
			absence_type_id: typeId,
		})
		return {
			save: async (typeId, draft) => {
				const body = absenceTypeRequestOf(draft)
				if (typeId === null) {
					const created = await createAsync({
						path: { organization_id: organizationId },
						body,
					})
					setTypeAccount(created.data.id, draft.accountId)
					return
				}
				await updateAsync({ path: path(typeId), body })
				setTypeAccount(typeId, draft.accountId)
			},
			archive: async (typeId) => {
				await archiveAsync({ path: path(typeId) })
			},
			restore: async (typeId) => {
				await restoreAsync({ path: path(typeId) })
			},
		}
	}, [organizationId, createAsync, updateAsync, archiveAsync, restoreAsync])
}
