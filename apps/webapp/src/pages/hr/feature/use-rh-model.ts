import { useEffect, useMemo } from 'react'
import { useAbsences } from '#/hooks/use-absences'
import { useMemberProfiles } from '#/hooks/use-member-profiles'
import { useReferenceCatalog } from '#/hooks/use-reference-catalog'
import {
	type RealMember,
	realAbsenceToAbsence,
} from '#/pages/hr/feature/mocks/rh-binding'
import {
	bindRhMembers,
	getRhActorId,
	memberKey,
	setMemberProfiles,
	setRealAbsences,
	updateRh,
	useRhSnapshot,
} from '#/pages/hr/feature/mocks/rh-store'
import { useProfileActions } from '#/pages/hr/feature/use-profile-actions'
import { absenceToDraft } from '#/pages/hr/lib/absences'
import type { RhState } from '#/pages/hr/lib/rh/types'
import { browserTimeZone } from '#/pages/hr/types'
import type { RhContextValue } from '#/pages/hr/ui/rh/shared/rh-context'

export interface RhModel {
	isLoading: boolean
	error: string | null
	context: RhContextValue
	state: RhState
}

export function useRhModel(
	organizationId: string,
	organizationSlug: string,
): RhModel {
	const catalog = useReferenceCatalog(organizationId, {
		employeeProfiles: false,
		equipment: false,
		serviceRates: false,
		products: false,
	})
	const absencesQuery = useAbsences(organizationId)
	const profilesQuery = useMemberProfiles(organizationId)
	const profiles = useProfileActions(organizationId)
	const snapshot = useRhSnapshot()

	const memberRows = catalog.members.data?.data
	const members = useMemo<RealMember[]>(
		() =>
			(memberRows ?? []).map((member) => ({
				id: member.id,
				firstName: member.first_name ?? '',
				lastName: member.last_name,
				email: member.account?.email ?? '',
			})),
		[memberRows],
	)
	const absenceRows = absencesQuery.data?.data
	const absences = useMemo(() => {
		const timeZone = browserTimeZone()
		return (absenceRows ?? []).map((absence) =>
			realAbsenceToAbsence(
				absence.id,
				absenceToDraft({ ...absence, absence_kind: absence.kind }, timeZone),
			),
		)
	}, [absenceRows])

	const membersLoaded = catalog.members.isSuccess
	const absencesLoaded = absencesQuery.isSuccess
	const profileRows = profilesQuery.data?.data
	const profilesLoaded = profilesQuery.isSuccess

	useEffect(() => {
		if (membersLoaded) bindRhMembers(members)
	}, [membersLoaded, members])

	useEffect(() => {
		if (absencesLoaded) setRealAbsences(absences)
	}, [absencesLoaded, absences])

	useEffect(() => {
		if (profilesLoaded) setMemberProfiles(profileRows ?? [])
	}, [profilesLoaded, profileRows])

	const isBound = membersLoaded && snapshot.boundKey === memberKey(members)
	return {
		isLoading:
			catalog.members.isLoading ||
			profilesQuery.isLoading ||
			!isBound ||
			!snapshot.profilesLoaded,
		error:
			catalog.members.error?.message ?? profilesQuery.error?.message ?? null,
		state: snapshot.state,
		context: {
			state: snapshot.state,
			update: updateRh,
			profiles,
			actorId: getRhActorId(),
			organizationSlug,
		},
	}
}
