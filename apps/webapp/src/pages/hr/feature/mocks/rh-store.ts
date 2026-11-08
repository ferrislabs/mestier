// MOCK: rh-store — contracts, schedules, documents, balances, overrides and extras have no backend endpoint yet. Member profile fields (contacts, address, job title, manager, work permit, archiving) come from the API through setMemberProfiles and are never held here.
import { useSyncExternalStore } from 'react'
import { todayIso } from '#/pages/hr/lib/rh/dates'
import {
	overlayProfiles,
	type ProfileRecord,
} from '#/pages/hr/lib/rh/member-profile'
import type { Absence, ActionContext, RhState } from '#/pages/hr/lib/rh/types'
import {
	bindSeed,
	isRealAbsenceId,
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
}

let base: RhState = INITIAL_RH_STATE
let realAbsences: readonly Absence[] = []
let boundKey: string | null = null
let profiles: readonly ProfileRecord[] = []
let profilesLoaded = false
let accountEmails: ReadonlyMap<string, string> = new Map()
let actorId = ''
let snapshot: Snapshot = compose()
const listeners = new Set<() => void>()

function compose(): Snapshot {
	const overlaid = overlayProfiles(base, profiles, accountEmails)
	return {
		state: { ...overlaid, absences: [...base.absences, ...realAbsences] },
		boundKey,
		profilesLoaded,
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
	base =
		boundKey === null || !sameTenant
			? bindSeed(INITIAL_RH_STATE, members)
			: reconcileMembers(base, members)
	actorId = members[0]?.id ?? ''
	boundKey = key
	publish()
}

export function setRealAbsences(absences: readonly Absence[]) {
	realAbsences = absences
	publish()
}

export function setMemberProfiles(records: readonly ProfileRecord[]) {
	profiles = records
	profilesLoaded = true
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
	base = {
		...next,
		absences: next.absences.filter((absence) => !isRealAbsenceId(absence.id)),
	}
	publish()
	return new Promise((resolve) => setTimeout(resolve, WRITE_DELAY_MS))
}

export function resetRhStore() {
	base = INITIAL_RH_STATE
	realAbsences = []
	profiles = []
	profilesLoaded = false
	accountEmails = new Map()
	boundKey = null
	actorId = ''
	publish()
}
