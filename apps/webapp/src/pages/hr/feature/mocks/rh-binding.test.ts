import { describe, expect, it } from 'vitest'
import { bindSeed, reconcileMembers, renameMembers } from './rh-binding'
import { INITIAL_RH_STATE } from './rh-seed'

const REAL = [
	{ id: 'u-1', firstName: 'Alix', lastName: 'Nova', email: 'alix@acme.fr' },
	{ id: 'u-2', firstName: 'Bao', lastName: 'Tran', email: '' },
	{ id: 'u-3', firstName: 'Cleo', lastName: 'Dupont', email: '' },
]

describe('bindSeed', () => {
	it('maps the first seed members onto the real ones by position', () => {
		const state = bindSeed(INITIAL_RH_STATE, REAL)

		expect(state.members.map((member) => member.id)).toEqual([
			'u-1',
			'u-2',
			'u-3',
		])
		expect(state.members[0]?.firstName).toBe('Alix')
		expect(state.members[0]?.workEmail).toBe('alix@acme.fr')
		expect(state.members[1]?.workEmail).not.toBe('')
	})

	it('keeps only the records that point at a bound member', () => {
		const state = bindSeed(INITIAL_RH_STATE, REAL)
		const ids = new Set(REAL.map((member) => member.id))

		expect(state.absences.length).toBeGreaterThan(0)
		for (const absence of state.absences)
			expect(ids.has(absence.memberId)).toBe(true)
		for (const extra of state.extras) expect(ids.has(extra.memberId)).toBe(true)
		for (const override of state.overrides)
			expect(ids.has(override.memberId)).toBe(true)
		for (const adjustment of state.adjustments)
			expect(ids.has(adjustment.memberId)).toBe(true)
	})

	it('drops a manager that is not part of the real members', () => {
		const state = bindSeed(INITIAL_RH_STATE, REAL)

		for (const member of state.members) {
			if (member.managerId !== null)
				expect(REAL.some((entry) => entry.id === member.managerId)).toBe(true)
		}
	})

	it('creates a default member when there are more real members than seeds', () => {
		const many = Array.from({
			length: INITIAL_RH_STATE.members.length + 1,
		}).map((_, index) => ({
			id: `m-${index}`,
			firstName: `P${index}`,
			lastName: 'Test',
			email: '',
		}))
		const state = bindSeed(INITIAL_RH_STATE, many)
		const extra = state.members.at(-1)

		expect(state.members).toHaveLength(many.length)
		expect(extra?.id).toBe(`m-${many.length - 1}`)
		expect(extra?.contracts).toHaveLength(1)
	})

	it('binds nothing when there is no real member', () => {
		const state = bindSeed(INITIAL_RH_STATE, [])

		expect(state.members).toEqual([])
		expect(state.absences).toEqual([])
	})
})

describe('reconcileMembers', () => {
	it('adds new members and removes the ones that disappeared with their records', () => {
		const bound = bindSeed(INITIAL_RH_STATE, REAL)
		const next = reconcileMembers(bound, [
			REAL[0],
			{ id: 'u-9', firstName: 'Zoe', lastName: 'New', email: '' },
		] as typeof REAL)

		expect(next.members.map((member) => member.id)).toEqual(['u-1', 'u-9'])
		for (const absence of next.absences)
			expect(['u-1', 'u-9']).toContain(absence.memberId)
	})
})

describe('renameMembers', () => {
	it('returns the same state when no name changed', () => {
		const bound = bindSeed(INITIAL_RH_STATE, REAL)

		expect(renameMembers(bound, REAL)).toBe(bound)
	})

	it('applies a new name from the real member', () => {
		const bound = bindSeed(INITIAL_RH_STATE, REAL)
		const next = renameMembers(bound, [
			{ ...REAL[0], lastName: 'Renamed' },
			REAL[1],
			REAL[2],
		] as typeof REAL)

		expect(next.members[0]?.lastName).toBe('Renamed')
		expect(next.members[0]?.firstName).toBe('Alix')
	})
})
