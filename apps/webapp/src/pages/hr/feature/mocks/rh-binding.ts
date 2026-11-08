// MOCK: rh-binding — no backend endpoint yet. Replace with the HR domain API once members carry contracts, schedules and balances.
import type { Member, RhState } from '#/pages/hr/lib/rh/types'
import { defaultMember } from './rh-seed'

export interface RealMember {
	id: string
	firstName: string
	lastName: string
	email: string
}

function bindMember(seed: Member, real: RealMember): Member {
	return {
		...seed,
		id: real.id,
		firstName: real.firstName,
		lastName: real.lastName,
		workEmail: real.email || seed.workEmail,
	}
}

function fromReal(real: RealMember): Member {
	return defaultMember({
		id: real.id,
		firstName: real.firstName,
		lastName: real.lastName,
		workEmail: real.email,
	})
}

export function bindSeed(seed: RhState, real: readonly RealMember[]): RhState {
	const ids = new Map<string, string>()
	seed.members.forEach((member, index) => {
		const target = real[index]
		if (target) ids.set(member.id, target.id)
	})
	const remap = (id: string | null): string | null =>
		id === null ? null : (ids.get(id) ?? null)
	const actor = real[0]?.id ?? ''
	const members = real.map((entry, index) => {
		const base = seed.members[index]
		if (!base) return fromReal(entry)
		return {
			...bindMember(base, entry),
			managerId: remap(base.managerId),
		}
	})
	return {
		...seed,
		members,
		absences: seed.absences.flatMap((absence) => {
			const memberId = remap(absence.memberId)
			if (memberId === null) return []
			return [
				{
					...absence,
					memberId,
					createdById: remap(absence.createdById) ?? memberId,
					decidedById: remap(absence.decidedById),
				},
			]
		}),
		adjustments: seed.adjustments.flatMap((adjustment) => {
			const memberId = remap(adjustment.memberId)
			if (memberId === null) return []
			return [
				{
					...adjustment,
					memberId,
					authorId: remap(adjustment.authorId) ?? actor,
				},
			]
		}),
		overrides: seed.overrides.flatMap((override) => {
			const memberId = remap(override.memberId)
			return memberId === null ? [] : [{ ...override, memberId }]
		}),
		extras: seed.extras.flatMap((extra) => {
			const memberId = remap(extra.memberId)
			if (memberId === null) return []
			return [
				{
					...extra,
					memberId,
					createdById: remap(extra.createdById) ?? memberId,
					decidedById: remap(extra.decidedById),
				},
			]
		}),
	}
}

export function reconcileMembers(
	state: RhState,
	real: readonly RealMember[],
): RhState {
	const known = new Set(state.members.map((member) => member.id))
	const wanted = new Set(real.map((entry) => entry.id))
	const kept = state.members.filter((member) => wanted.has(member.id))
	const added = real.filter((entry) => !known.has(entry.id)).map(fromReal)
	const members = [...kept, ...added].map((member) =>
		member.managerId !== null && !wanted.has(member.managerId)
			? { ...member, managerId: null }
			: member,
	)
	return {
		...state,
		members,
		absences: state.absences.filter((absence) => wanted.has(absence.memberId)),
		adjustments: state.adjustments.filter((adjustment) =>
			wanted.has(adjustment.memberId),
		),
		overrides: state.overrides.filter((override) =>
			wanted.has(override.memberId),
		),
		extras: state.extras.filter((extra) => wanted.has(extra.memberId)),
	}
}

export function renameMembers(
	state: RhState,
	real: readonly RealMember[],
): RhState {
	const byId = new Map(real.map((entry) => [entry.id, entry]))
	let changed = false
	const members = state.members.map((member) => {
		const entry = byId.get(member.id)
		if (
			!entry ||
			(entry.firstName === member.firstName &&
				entry.lastName === member.lastName)
		)
			return member
		changed = true
		return { ...member, firstName: entry.firstName, lastName: entry.lastName }
	})
	return changed ? { ...state, members } : state
}
