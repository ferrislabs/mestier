import { useMemo } from 'react'
import { useCancelAbsence } from '#/hooks/use-absences'
import {
	useArchiveMember,
	usePutMemberProfile,
	useRestoreMember,
} from '#/hooks/use-member-profiles'
import { getRhState } from '#/pages/hr/feature/mocks/rh-store'
import { todayIso } from '#/pages/hr/lib/rh/dates'
import {
	type ProfileFields,
	profileRequestOf,
} from '#/pages/hr/lib/rh/member-profile'
import {
	findMemberById,
	upcomingPendingAbsenceIds,
} from '#/pages/hr/lib/rh/members'
import type { RhProfileActions } from '#/pages/hr/ui/rh/shared/rh-context'

export function useProfileActions(organizationId: string): RhProfileActions {
	const put = usePutMemberProfile()
	const archive = useArchiveMember()
	const restore = useRestoreMember()
	const cancel = useCancelAbsence()
	const putAsync = put.mutateAsync
	const archiveAsync = archive.mutateAsync
	const restoreAsync = restore.mutateAsync
	const cancelAsync = cancel.mutateAsync

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
				const pending = upcomingPendingAbsenceIds(
					getRhState(),
					memberId,
					todayIso(),
				)
				await Promise.allSettled(
					pending.map((absenceId) =>
						cancelAsync({
							path: {
								organization_id: organizationId,
								absence_id: absenceId,
							},
						}),
					),
				)
			},
			restore: async (memberId: string) => {
				await restoreAsync({ path: path(memberId) })
			},
		}
	}, [organizationId, putAsync, archiveAsync, restoreAsync, cancelAsync])
}
