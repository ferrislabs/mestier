// MOCK: rh-store — nothing here is read from the backend except through the setters. Day overrides come from the API by period through setDayOverrides and the seed overrides are never read. Member profile fields (contacts, address, job title, manager, work permit, archiving), contracts with their schedules, documents, absence types, absences, leave accounts and balance adjustments come from the API through setMemberProfiles, setMemberContracts, setMemberDocuments, setAbsenceTypes, setAbsences, setLeaveAccounts and setLeaveAdjustments, and surcharges and extra periods through setSurcharges and setExtraPeriods; none of them is held here.
import { useSyncExternalStore } from 'react'
import {
	type AbsenceRecord,
	type AbsenceTypeRecord,
	overlayAbsences,
	overlayAbsenceTypes,
} from '#/pages/hr/lib/rh/absence-wire'
import { todayIso } from '#/pages/hr/lib/rh/dates'
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
import {
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
	periodKey,
} from '#/pages/hr/lib/rh/override-wire'
import type { ActionContext, RhState } from '#/pages/hr/lib/rh/types'
import {
	bindSeed,
	type RealMember,
	reconcileMembers,
	renameMembers,
} from './rh-binding'
import { INITIAL_RH_STATE } from './rh-seed'

const WRITE_DELAY_MS = 120

interface Snapshot {
	state: RhState
	boundKey: string | null
	profilesLoaded: boolean
	contractsLoaded: boolean
	documentsLoaded: boolean
	typesLoaded: boolean
	absencesLoaded: boolean
	accountsLoaded: boolean
	adjustmentsLoaded: boolean
	surchargesLoaded: boolean
	extrasLoaded: boolean
}

let base: RhState = INITIAL_RH_STATE
let absenceTypes: readonly AbsenceTypeRecord[] = []
let typesLoaded = false
let absences: readonly AbsenceRecord[] = []
let absencesLoaded = false
let accounts: readonly LeaveAccountRecord[] = []
let accountsLoaded = false
let adjustments: readonly LeaveAdjustmentRecord[] = []
let adjustmentsLoaded = false
let surcharges: readonly SurchargeRecord[] = []
let surchargesLoaded = false
let extras: readonly ExtraPeriodRecord[] = []
let extrasLoaded = false
let overridePeriods = new Map<string, OverridePeriod>()
let boundKey: string | null = null
let profiles: readonly ProfileRecord[] = []
let profilesLoaded = false
let contracts: readonly ContractRecord[] = []
let contractsLoaded = false
let documents: readonly DocumentRecord[] = []
let documentsLoaded = false
let accountEmails: ReadonlyMap<string, string> = new Map()
let actorId = ''
let snapshot: Snapshot = compose()
const listeners = new Set<() => void>()

function compose(): Snapshot {
	let overlaid = overlayProfiles(base, profiles, accountEmails)
	overlaid = overlayContracts(overlaid, contracts)
	overlaid = overlayDocuments(overlaid, documents)
	overlaid = overlayAbsenceTypes(overlaid, absenceTypes)
	overlaid = overlayAbsences(overlaid, absences)
	overlaid = overlayAccounts(overlaid, accounts)
	overlaid = overlayAdjustments(overlaid, adjustments)
	overlaid = overlaySurcharges(overlaid, surcharges)
	overlaid = overlayExtras(overlaid, extras)
	overlaid = overlayOverrides(overlaid, [...overridePeriods.values()])
	return {
		state: overlaid,
		boundKey,
		profilesLoaded,
		contractsLoaded,
		documentsLoaded,
		typesLoaded,
		absencesLoaded,
		accountsLoaded,
		adjustmentsLoaded,
		surchargesLoaded,
		extrasLoaded,
	}
}

function publish() {
	snapshot = compose()
	for (const listener of listeners) listener()
}

function subscribe(listener: () => void) {
	listeners.add(listener)
	return () => {
		listeners.delete(listener)
	}
}

function keyOf(members: readonly RealMember[]): string {
	return members.map((member) => member.id).join('|')
}

export function bindRhMembers(members: readonly RealMember[]) {
	const key = keyOf(members)
	accountEmails = new Map(members.map((member) => [member.id, member.email]))
	if (key === boundKey) {
		const renamed = renameMembers(base, members)
		if (renamed !== base) {
			base = renamed
			publish()
		}
		return
	}
	const ids = new Set(members.map((member) => member.id))
	const sameTenant = base.members.some((member) => ids.has(member.id))
	if (boundKey === null || !sameTenant) overridePeriods = new Map()
	base =
		boundKey === null || !sameTenant
			? bindSeed(INITIAL_RH_STATE, members)
			: reconcileMembers(base, members)
	actorId = members[0]?.id ?? ''
	boundKey = key
	publish()
}

export function setAbsenceTypes(records: readonly AbsenceTypeRecord[]) {
	absenceTypes = records
	typesLoaded = true
	publish()
}

export function setLeaveAccounts(records: readonly LeaveAccountRecord[]) {
	accounts = records
	accountsLoaded = true
	publish()
}

export function setLeaveAdjustments(records: readonly LeaveAdjustmentRecord[]) {
	adjustments = records
	adjustmentsLoaded = true
	publish()
}

export function setSurcharges(records: readonly SurchargeRecord[]) {
	surcharges = records
	surchargesLoaded = true
	publish()
}

export function setExtraPeriods(records: readonly ExtraPeriodRecord[]) {
	extras = records
	extrasLoaded = true
	publish()
}

export function setDayOverrides(period: OverridePeriod) {
	const key = periodKey(period.memberId, period.from, period.to)
	const next = new Map(overridePeriods)
	next.delete(key)
	next.set(key, period)
	overridePeriods = next
	publish()
}

export function setAbsences(records: readonly AbsenceRecord[]) {
	absences = records
	absencesLoaded = true
	publish()
}

export function setMemberProfiles(records: readonly ProfileRecord[]) {
	profiles = records
	profilesLoaded = true
	publish()
}

export function setMemberContracts(records: readonly ContractRecord[]) {
	contracts = records
	contractsLoaded = true
	publish()
}

export function setMemberDocuments(records: readonly DocumentRecord[]) {
	documents = records
	documentsLoaded = true
	publish()
}

export function memberKey(members: readonly RealMember[]): string {
	return keyOf(members)
}

export function useRhSnapshot(): Snapshot {
	return useSyncExternalStore(subscribe, () => snapshot)
}

export function getRhState(): RhState {
	return snapshot.state
}

export function getRhActorId(): string {
	return actorId
}

function actionContext(): ActionContext {
	return {
		now: new Date().toISOString(),
		today: todayIso(),
		actorId,
		newId: () => crypto.randomUUID(),
	}
}

export function updateRh(
	update: (current: RhState, context: ActionContext) => RhState,
): Promise<void> {
	const next = update(snapshot.state, actionContext())
	base = next
	publish()
	return new Promise((resolve) => setTimeout(resolve, WRITE_DELAY_MS))
}

export function resetRhStore() {
	base = INITIAL_RH_STATE
	absenceTypes = []
	typesLoaded = false
	absences = []
	absencesLoaded = false
	accounts = []
	accountsLoaded = false
	adjustments = []
	adjustmentsLoaded = false
	surcharges = []
	surchargesLoaded = false
	extras = []
	extrasLoaded = false
	overridePeriods = new Map()
	profiles = []
	profilesLoaded = false
	contracts = []
	contractsLoaded = false
	documents = []
	documentsLoaded = false
	accountEmails = new Map()
	boundKey = null
	actorId = ''
	publish()
}
