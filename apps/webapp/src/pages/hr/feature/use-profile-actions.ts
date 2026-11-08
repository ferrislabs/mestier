import { useMemo } from 'react'
import {
	useArchiveMember,
	usePutMemberProfile,
	useRestoreMember,
} from '#/hooks/use-member-profiles'
import { getRhState, updateRh } from '#/pages/hr/feature/mocks/rh-store'
import {
	type ProfileFields,
	profileRequestOf,
} from '#/pages/hr/lib/rh/member-profile'
import {
	cancelUpcomingAbsences,
	findMemberById,
} from '#/pages/hr/lib/rh/members'
import type { RhProfileActions } from '#/pages/hr/ui/rh/shared/rh-context'

export function useProfileActions(organizationId: string): RhProfileActions {
	const put = usePutMemberProfile()
	const archive = useArchiveMember()
	const restore = useRestoreMember()
	const putAsync = put.mutateAsync
	const archiveAsync = archive.mutateAsync
	const restoreAsync = restore.mutateAsync

	return useMemo(() => {
		const path = (memberId: string) => ({
			organization_id: organizationId,
			member_id: memberId,
		})
		const save = async (memberId: string, fields: ProfileFields) => {
			await putAsync({
				path: path(memberId),
				body: profileRequestOf(fields),
			})
		}
		return {
			save,
			archive: async (memberId: string) => {
				const archived = findMemberById(getRhState(), memberId)
				await archiveAsync({
					path: path(memberId),
					body: { reassign_reports_to: archived?.managerId ?? null },
				})
				await updateRh((current, context) =>
					cancelUpcomingAbsences(current, context, memberId),
				)
			},
			restore: async (memberId: string) => {
				await restoreAsync({ path: path(memberId) })
			},
		}
	}, [organizationId, putAsync, archiveAsync, restoreAsync])
}
