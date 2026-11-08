import { useMemo } from 'react'
import {
	useApproveExtraPeriod,
	useCancelExtraPeriod,
	useCreateExtraPeriod,
	useRejectExtraPeriod,
} from '#/hooks/use-extra-periods'
import {
	useArchiveSurcharge,
	useCreateSurcharge,
	useRestoreSurcharge,
	useUpdateSurcharge,
} from '#/hooks/use-surcharges'
import {
	extraRequestOf,
	surchargeRequestOf,
} from '#/pages/hr/lib/rh/extra-wire'
import type {
	RhExtraActions,
	RhSurchargeActions,
} from '#/pages/hr/ui/rh/shared/rh-context'

export function useExtraActions(organizationId: string): RhExtraActions {
	const create = useCreateExtraPeriod()
	const approve = useApproveExtraPeriod()
	const reject = useRejectExtraPeriod()
	const cancel = useCancelExtraPeriod()
	const createAsync = create.mutateAsync
	const approveAsync = approve.mutateAsync
	const rejectAsync = reject.mutateAsync
	const cancelAsync = cancel.mutateAsync

	return useMemo(() => {
		const path = (extraId: string) => ({
			organization_id: organizationId,
			period_id: extraId,
		})
		return {
			create: async (draft, approveNow) => {
				await createAsync({
					path: { organization_id: organizationId },
					body: extraRequestOf(draft, approveNow),
				})
			},
			approve: async (extraId) => {
				await approveAsync({ path: path(extraId) })
			},
			reject: async (extraId, comment) => {
				await rejectAsync({
					path: path(extraId),
					body: { comment: comment.trim() },
				})
			},
			cancel: async (extraId) => {
				await cancelAsync({ path: path(extraId) })
			},
		}
	}, [organizationId, createAsync, approveAsync, rejectAsync, cancelAsync])
}

export function useSurchargeActions(
	organizationId: string,
): RhSurchargeActions {
	const create = useCreateSurcharge()
	const update = useUpdateSurcharge()
	const archive = useArchiveSurcharge()
	const restore = useRestoreSurcharge()
	const createAsync = create.mutateAsync
	const updateAsync = update.mutateAsync
	const archiveAsync = archive.mutateAsync
	const restoreAsync = restore.mutateAsync

	return useMemo(() => {
		const path = (surchargeId: string) => ({
			organization_id: organizationId,
			surcharge_id: surchargeId,
		})
		return {
			save: async (surchargeId, draft) => {
				const body = surchargeRequestOf(draft)
				if (surchargeId === null) {
					await createAsync({
						path: { organization_id: organizationId },
						body,
					})
					return
				}
				await updateAsync({ path: path(surchargeId), body })
			},
			archive: async (surchargeId) => {
				await archiveAsync({ path: path(surchargeId) })
			},
			restore: async (surchargeId) => {
				await restoreAsync({ path: path(surchargeId) })
			},
		}
	}, [organizationId, createAsync, updateAsync, archiveAsync, restoreAsync])
}
