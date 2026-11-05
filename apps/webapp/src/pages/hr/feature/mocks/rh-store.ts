// MOCK: rh-store — no backend endpoint yet. Replace with TanStack Query hooks over the HR domain API.
import { useSyncExternalStore } from 'react'
import { todayIso } from '#/pages/hr/lib/rh/dates'
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
}

let base: RhState = INITIAL_RH_STATE
let realAbsences: readonly Absence[] = []
let boundKey: string | null = null
let actorId = ''
let snapshot: Snapshot = compose()
const listeners = new Set<() => void>()

function compose(): Snapshot {
	return {
		state: { ...base, absences: [...base.absences, ...realAbsences] },
		boundKey,
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

export function memberKey(members: readonly RealMember[]): string {
	return keyOf(members)
}

export function useRhSnapshot(): Snapshot {
	return useSyncExternalStore(subscribe, () => snapshot)
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
	boundKey = null
	actorId = ''
	publish()
}
