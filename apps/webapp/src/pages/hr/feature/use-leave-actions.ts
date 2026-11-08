import { useMemo } from 'react'
import {
	useArchiveLeaveAccount,
	useRestoreLeaveAccount,
	useUpdateLeaveAccount,
} from '#/hooks/use-leave-accounts'
import {
	useCreateLeaveAdjustment,
	useDeleteLeaveAdjustment,
} from '#/hooks/use-leave-adjustments'
import {
	accountRequestOf,
	adjustmentRequestOf,
} from '#/pages/hr/lib/rh/leave-wire'
import type { RhLeaveActions } from '#/pages/hr/ui/rh/shared/rh-context'

export function useLeaveActions(organizationId: string): RhLeaveActions {
	const update = useUpdateLeaveAccount()
	const archive = useArchiveLeaveAccount()
	const restore = useRestoreLeaveAccount()
	const createAdjustment = useCreateLeaveAdjustment()
	const deleteAdjustment = useDeleteLeaveAdjustment()
	const updateAsync = update.mutateAsync
	const archiveAsync = archive.mutateAsync
	const restoreAsync = restore.mutateAsync
	const createAdjustmentAsync = createAdjustment.mutateAsync
	const deleteAdjustmentAsync = deleteAdjustment.mutateAsync

	return useMemo(() => {
		const accountPath = (accountId: string) => ({
			organization_id: organizationId,
			account_id: accountId,
		})
		return {
			saveAccount: async (account, settings) => {
				await updateAsync({
					path: accountPath(account.id),
					body: accountRequestOf(account, settings),
				})
			},
			archiveAccount: async (accountId) => {
				await archiveAsync({ path: accountPath(accountId) })
			},
			restoreAccount: async (accountId) => {
				await restoreAsync({ path: accountPath(accountId) })
			},
			addAdjustment: async (draft) => {
				await createAdjustmentAsync({
					path: { organization_id: organizationId },
					body: adjustmentRequestOf(draft),
				})
			},
			removeAdjustment: async (adjustmentId) => {
				await deleteAdjustmentAsync({
					path: {
						organization_id: organizationId,
						adjustment_id: adjustmentId,
					},
				})
			},
		}
	}, [
		organizationId,
		updateAsync,
		archiveAsync,
		restoreAsync,
		createAdjustmentAsync,
		deleteAdjustmentAsync,
	])
}
