import { useCallback, useMemo, useRef, useState } from 'react'
import { useAbsenceTypes } from '#/hooks/use-absence-types'
import { useAbsences } from '#/hooks/use-absences'
import { useCurrentMember } from '#/hooks/use-current-member'
import { useExtraPeriods } from '#/hooks/use-extra-periods'
import { useLeaveAccounts } from '#/hooks/use-leave-accounts'
import { useLeaveAdjustments } from '#/hooks/use-leave-adjustments'
import { useMemberContracts } from '#/hooks/use-member-contracts'
import { useMemberDocuments } from '#/hooks/use-member-documents'
import { useMemberProfiles } from '#/hooks/use-member-profiles'
import { useReferenceCatalog } from '#/hooks/use-reference-catalog'
import { useSurcharges } from '#/hooks/use-surcharges'
import { isForbiddenError, isNotFoundError } from '#/lib/api-error'
import {
	useAbsenceActions,
	useAbsenceTypeActions,
} from '#/pages/hr/feature/use-absence-actions'
import {
	useContractActions,
	useDocumentActions,
} from '#/pages/hr/feature/use-contract-actions'
import {
	useExtraActions,
	useSurchargeActions,
} from '#/pages/hr/feature/use-extra-actions'
import { useLeaveActions } from '#/pages/hr/feature/use-leave-actions'
import { useOverrideActions } from '#/pages/hr/feature/use-override-actions'
import { useProfileActions } from '#/pages/hr/feature/use-profile-actions'
import {
	actorIdOf,
	composeRhState,
	type RealMember,
} from '#/pages/hr/lib/rh/compose'
import { type OverridePeriod, periodKey } from '#/pages/hr/lib/rh/override-wire'
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
	const surchargesQuery = useSurcharges(organizationId)
	const extrasQuery = useExtraPeriods(organizationId)
	const currentMemberQuery = useCurrentMember(organizationId)
	const profilesQuery = useMemberProfiles(organizationId)
	const contractsQuery = useMemberContracts(organizationId)
	const documentsQuery = useMemberDocuments(organizationId)
	const stateRef = useRef<RhState | null>(null)
	const getState = useCallback((): RhState => {
		if (!stateRef.current) throw new Error('RhState indisponible')
		return stateRef.current
	}, [])
	const [periods, setPeriods] = useState<ReadonlyMap<string, OverridePeriod>>(
		new Map(),
	)
	const setOverridePeriod = useCallback((period: OverridePeriod) => {
		setPeriods((current) => {
			const next = new Map(current)
			const key = periodKey(period.memberId, period.from, period.to)
			next.delete(key)
			next.set(key, period)
			return next
		})
	}, [])
	const profiles = useProfileActions(organizationId, getState)
	const contracts = useContractActions(organizationId)
	const documents = useDocumentActions(organizationId)
	const absenceActions = useAbsenceActions(organizationId)
	const absenceTypeActions = useAbsenceTypeActions(organizationId)
	const leaveActions = useLeaveActions(organizationId)
	const extraActions = useExtraActions(organizationId)
	const surchargeActions = useSurchargeActions(organizationId)
	const overrideActions = useOverrideActions(organizationId, getState)
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

	const surchargeRows = surchargesQuery.data?.data
	const surchargesRestricted = isForbiddenError(surchargesQuery.error)
	const surchargesLoaded = surchargesQuery.isSuccess || surchargesRestricted
	const extraRows = extrasQuery.data?.data
	const extrasRestricted = isForbiddenError(extrasQuery.error)
	const extrasLoaded = extrasQuery.isSuccess || extrasRestricted

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

	const state = useMemo(
		() =>
			composeRhState({
				members,
				profiles: profileRows ?? [],
				contracts: contractRows ?? [],
				documents: documentRows ?? [],
				absenceTypes: typeRows ?? [],
				absences: absenceRows ?? [],
				accounts: accountRows ?? [],
				adjustments: adjustmentRows ?? [],
				surcharges: surchargeRows ?? [],
				extras: extraRows ?? [],
				overridePeriods: [...periods.values()],
			}),
		[
			members,
			profileRows,
			contractRows,
			documentRows,
			typeRows,
			absenceRows,
			accountRows,
			adjustmentRows,
			surchargeRows,
			extraRows,
			periods,
		],
	)
	stateRef.current = state
	const seatless = isNotFoundError(currentMemberQuery.error)
	const currentMemberLoaded = currentMemberQuery.isSuccess || seatless
	const actorId = actorIdOf(currentMemberQuery.data?.data.member_id)

	return {
		isLoading:
			!membersLoaded ||
			!profilesLoaded ||
			!contractsLoaded ||
			!documentsLoaded ||
			!absencesLoaded ||
			!typesLoaded ||
			!accountsLoaded ||
			!adjustmentsLoaded ||
			!surchargesLoaded ||
			!extrasLoaded ||
			!currentMemberLoaded,
		error:
			catalog.members.error?.message ??
			(profilesRestricted ? null : profilesQuery.error?.message) ??
			(contractsRestricted ? null : contractsQuery.error?.message) ??
			(documentsRestricted ? null : documentsQuery.error?.message) ??
			(absencesRestricted ? null : absencesQuery.error?.message) ??
			(typesRestricted ? null : typesQuery.error?.message) ??
			(accountsRestricted ? null : accountsQuery.error?.message) ??
			(adjustmentsRestricted ? null : adjustmentsQuery.error?.message) ??
			(surchargesRestricted ? null : surchargesQuery.error?.message) ??
			(extrasRestricted ? null : extrasQuery.error?.message) ??
			(!seatless && currentMemberQuery.error instanceof Error
				? currentMemberQuery.error.message
				: null) ??
			null,
		state,
		context: {
			state,
			profiles,
			contracts,
			documents,
			absences: absenceActions,
			absenceTypes: absenceTypeActions,
			leave: leaveActions,
			extras: extraActions,
			surcharges: surchargeActions,
			overrides: overrideActions,
			setOverridePeriod,
			actorId,
			organizationId,
			organizationSlug,
		},
	}
}
