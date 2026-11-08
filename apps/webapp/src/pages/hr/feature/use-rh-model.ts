import { useEffect, useMemo } from 'react'
import { useAbsenceTypes } from '#/hooks/use-absence-types'
import { useAbsences } from '#/hooks/use-absences'
import { useLeaveAccounts } from '#/hooks/use-leave-accounts'
import { useLeaveAdjustments } from '#/hooks/use-leave-adjustments'
import { useMemberContracts } from '#/hooks/use-member-contracts'
import { useMemberDocuments } from '#/hooks/use-member-documents'
import { useMemberProfiles } from '#/hooks/use-member-profiles'
import { useReferenceCatalog } from '#/hooks/use-reference-catalog'
import { isForbiddenError } from '#/lib/api-error'
import type { RealMember } from '#/pages/hr/feature/mocks/rh-binding'
import {
	bindRhMembers,
	getRhActorId,
	memberKey,
	setAbsences,
	setAbsenceTypes,
	setLeaveAccounts,
	setLeaveAdjustments,
	setMemberContracts,
	setMemberDocuments,
	setMemberProfiles,
	updateRh,
	useRhSnapshot,
} from '#/pages/hr/feature/mocks/rh-store'
import {
	useAbsenceActions,
	useAbsenceTypeActions,
} from '#/pages/hr/feature/use-absence-actions'
import {
	useContractActions,
	useDocumentActions,
} from '#/pages/hr/feature/use-contract-actions'
import { useLeaveActions } from '#/pages/hr/feature/use-leave-actions'
import { useProfileActions } from '#/pages/hr/feature/use-profile-actions'
import type { RhState } from '#/pages/hr/lib/rh/types'
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
	const typesQuery = useAbsenceTypes(organizationId)
	const accountsQuery = useLeaveAccounts(organizationId)
	const adjustmentsQuery = useLeaveAdjustments(organizationId)
	const profilesQuery = useMemberProfiles(organizationId)
	const contractsQuery = useMemberContracts(organizationId)
	const documentsQuery = useMemberDocuments(organizationId)
	const profiles = useProfileActions(organizationId)
	const contracts = useContractActions(organizationId)
	const documents = useDocumentActions(organizationId)
	const absenceActions = useAbsenceActions(organizationId)
	const absenceTypeActions = useAbsenceTypeActions(organizationId)
	const leaveActions = useLeaveActions(organizationId)
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
	const absencesRestricted = isForbiddenError(absencesQuery.error)
	const absencesLoaded = absencesQuery.isSuccess || absencesRestricted
	const typeRows = typesQuery.data?.data
	const typesRestricted = isForbiddenError(typesQuery.error)
	const typesLoaded = typesQuery.isSuccess || typesRestricted
	const accountRows = accountsQuery.data?.data
	const accountsRestricted = isForbiddenError(accountsQuery.error)
	const accountsLoaded = accountsQuery.isSuccess || accountsRestricted
	const adjustmentRows = adjustmentsQuery.data?.data
	const adjustmentsRestricted = isForbiddenError(adjustmentsQuery.error)
	const adjustmentsLoaded = adjustmentsQuery.isSuccess || adjustmentsRestricted

	const membersLoaded = catalog.members.isSuccess
	const profileRows = profilesQuery.data?.data
	const profilesRestricted = isForbiddenError(profilesQuery.error)
	const profilesLoaded = profilesQuery.isSuccess || profilesRestricted

	const contractRows = contractsQuery.data?.data
	const contractsRestricted = isForbiddenError(contractsQuery.error)
	const contractsLoaded = contractsQuery.isSuccess || contractsRestricted
	const documentRows = documentsQuery.data?.data
	const documentsRestricted = isForbiddenError(documentsQuery.error)
	const documentsLoaded = documentsQuery.isSuccess || documentsRestricted

	useEffect(() => {
		if (membersLoaded) bindRhMembers(members)
	}, [membersLoaded, members])

	useEffect(() => {
		if (absencesLoaded) setAbsences(absenceRows ?? [])
	}, [absencesLoaded, absenceRows])

	useEffect(() => {
		if (typesLoaded) setAbsenceTypes(typeRows ?? [])
	}, [typesLoaded, typeRows])

	useEffect(() => {
		if (accountsLoaded) setLeaveAccounts(accountRows ?? [])
	}, [accountsLoaded, accountRows])

	useEffect(() => {
		if (adjustmentsLoaded) setLeaveAdjustments(adjustmentRows ?? [])
	}, [adjustmentsLoaded, adjustmentRows])

	useEffect(() => {
		if (profilesLoaded) setMemberProfiles(profileRows ?? [])
	}, [profilesLoaded, profileRows])

	useEffect(() => {
		if (contractsLoaded) setMemberContracts(contractRows ?? [])
	}, [contractsLoaded, contractRows])

	useEffect(() => {
		if (documentsLoaded) setMemberDocuments(documentRows ?? [])
	}, [documentsLoaded, documentRows])

	const isBound = membersLoaded && snapshot.boundKey === memberKey(members)
	return {
		isLoading:
			catalog.members.isLoading ||
			profilesQuery.isLoading ||
			contractsQuery.isLoading ||
			documentsQuery.isLoading ||
			absencesQuery.isLoading ||
			typesQuery.isLoading ||
			accountsQuery.isLoading ||
			adjustmentsQuery.isLoading ||
			!isBound ||
			!snapshot.profilesLoaded ||
			!snapshot.contractsLoaded ||
			!snapshot.documentsLoaded ||
			!snapshot.absencesLoaded ||
			!snapshot.typesLoaded ||
			!snapshot.accountsLoaded ||
			!snapshot.adjustmentsLoaded,
		error:
			catalog.members.error?.message ??
			(profilesRestricted ? null : profilesQuery.error?.message) ??
			(contractsRestricted ? null : contractsQuery.error?.message) ??
			(documentsRestricted ? null : documentsQuery.error?.message) ??
			(absencesRestricted ? null : absencesQuery.error?.message) ??
			(typesRestricted ? null : typesQuery.error?.message) ??
			(accountsRestricted ? null : accountsQuery.error?.message) ??
			(adjustmentsRestricted ? null : adjustmentsQuery.error?.message) ??
			null,
		state: snapshot.state,
		context: {
			state: snapshot.state,
			update: updateRh,
			profiles,
			contracts,
			documents,
			absences: absenceActions,
			absenceTypes: absenceTypeActions,
			leave: leaveActions,
			actorId: getRhActorId(),
			organizationSlug,
		},
	}
}
