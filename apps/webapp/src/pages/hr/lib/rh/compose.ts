import {
	type AbsenceRecord,
	type AbsenceTypeRecord,
	overlayAbsences,
	overlayAbsenceTypes,
} from '#/pages/hr/lib/rh/absence-wire'
import {
	type ExtraPeriodRecord,
	overlayExtras,
	overlaySurcharges,
	type SurchargeRecord,
} from '#/pages/hr/lib/rh/extra-wire'
import {
	type LeaveAccountRecord,
	type LeaveAdjustmentRecord,
	overlayAccounts,
	overlayAdjustments,
} from '#/pages/hr/lib/rh/leave-wire'
import { memberHue } from '#/pages/hr/lib/rh/member-hue'
import {
	emptyProfileFields,
	overlayProfiles,
	type ProfileRecord,
} from '#/pages/hr/lib/rh/member-profile'
import {
	type ContractRecord,
	type DocumentRecord,
	overlayContracts,
	overlayDocuments,
} from '#/pages/hr/lib/rh/member-wire'
import {
	type OverridePeriod,
	overlayOverrides,
} from '#/pages/hr/lib/rh/override-wire'
import type { Member, RhState } from '#/pages/hr/lib/rh/types'

export interface RealMember {
	id: string
	firstName: string
	lastName: string
	email: string
}

export interface RhSources {
	members: readonly RealMember[]
	profiles: readonly ProfileRecord[]
	contracts: readonly ContractRecord[]
	documents: readonly DocumentRecord[]
	absenceTypes: readonly AbsenceTypeRecord[]
	absences: readonly AbsenceRecord[]
	accounts: readonly LeaveAccountRecord[]
	adjustments: readonly LeaveAdjustmentRecord[]
	surcharges: readonly SurchargeRecord[]
	extras: readonly ExtraPeriodRecord[]
	overridePeriods: readonly OverridePeriod[]
}

export function baseMemberOf(real: RealMember): Member {
	return {
		id: real.id,
		firstName: real.firstName,
		lastName: real.lastName,
		hue: memberHue(real.id),
		...emptyProfileFields(real.email),
		status: 'active',
		archivedAt: null,
		contracts: [],
		documents: [],
	}
}

export function composeRhState(sources: RhSources): RhState {
	const base: RhState = {
		members: sources.members.map(baseMemberOf),
		absenceTypes: [],
		accounts: [],
		absences: [],
		adjustments: [],
		overrides: [],
		surcharges: [],
		extras: [],
	}
	const accountEmails = new Map(
		sources.members.map((member) => [member.id, member.email]),
	)
	let state = overlayProfiles(base, sources.profiles, accountEmails)
	state = overlayContracts(state, sources.contracts)
	state = overlayDocuments(state, sources.documents)
	state = overlayAbsenceTypes(state, sources.absenceTypes)
	state = overlayAbsences(state, sources.absences)
	state = overlayAccounts(state, sources.accounts)
	state = overlayAdjustments(state, sources.adjustments)
	state = overlaySurcharges(state, sources.surcharges)
	state = overlayExtras(state, sources.extras)
	return overlayOverrides(state, sources.overridePeriods)
}

export function actorIdOf(
	members: readonly RealMember[],
	email: string | null | undefined,
): string {
	const wanted = email?.trim().toLowerCase()
	const own = wanted
		? members.find((member) => member.email.trim().toLowerCase() === wanted)
		: undefined
	return own?.id ?? members[0]?.id ?? ''
}
