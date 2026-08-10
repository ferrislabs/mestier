import { describe, expect, it } from 'vitest'
import {
	assigneeRefFromResourceId,
	buildAssigneesForMove,
	buildAssigneesForRemoval,
	computeRemoveAssigneePatch,
	computeTaskDropPatch,
	resourceIdFromAssigneeRef,
	shiftInstant,
	toggleAssignee,
} from '#/pages/planning/lib/task-drop'

const TZ = 'Europe/Paris'

describe('assigneeRefFromResourceId', () => {
	it('parses an employee resource_id', () => {
		expect(assigneeRefFromResourceId('employee:emp-1')).toEqual({
			kind: 'employee',
			employee_id: 'emp-1',
		})
	})

	it('parses a member resource_id', () => {
		expect(assigneeRefFromResourceId('member:user-9')).toEqual({
			kind: 'member',
			user_id: 'user-9',
		})
	})

	it('throws on an unknown kind — an upstream bug, not data to swallow', () => {
		expect(() => assigneeRefFromResourceId('unknown:x')).toThrow()
	})
})

describe('shiftInstant', () => {
	it('returns the same value for a zero offset', () => {
		expect(shiftInstant('2026-08-10T08:00:00Z', 0, TZ)).toBe(
			'2026-08-10T08:00:00Z',
		)
	})

	it('shifts by N days while preserving the local time', () => {
		const shifted = shiftInstant('2026-08-10T08:00:00+02:00', 2, TZ)
		expect(shifted).toBe(new Date('2026-08-12T08:00:00+02:00').toISOString())
	})

	it('shifts correctly across a DST change (late October in Europe/Paris)', () => {
		// 2026-10-24 is in CEST (+02:00), 2026-10-27 is in CET (+01:00) —
		// crossing the clock change must keep 08:00 local time.
		const shifted = shiftInstant('2026-10-24T08:00:00+02:00', 3, TZ)
		expect(shifted).toBe(new Date('2026-10-27T08:00:00+01:00').toISOString())
	})
})

describe('buildAssigneesForMove', () => {
	it('replaces the source resource with the target, preserving the others', () => {
		const result = buildAssigneesForMove(
			['emp-1', 'emp-2'],
			'employee:emp-1',
			'employee:emp-3',
		)
		expect(result).toEqual([
			{ kind: 'employee', employee_id: 'emp-3' },
			{ kind: 'employee', employee_id: 'emp-2' },
		])
	})

	it('converts to a member when the target is a member-only resource', () => {
		const result = buildAssigneesForMove(
			['emp-1'],
			'employee:emp-1',
			'member:user-9',
		)
		expect(result).toEqual([{ kind: 'member', user_id: 'user-9' }])
	})

	it('does not duplicate when the target is already assigned elsewhere on the same job', () => {
		const result = buildAssigneesForMove(
			['emp-1', 'emp-2'],
			'employee:emp-1',
			'employee:emp-2',
		)
		expect(result).toEqual([{ kind: 'employee', employee_id: 'emp-2' }])
	})

	it('touches nothing when source and target are the same resource', () => {
		const result = buildAssigneesForMove(
			['emp-1', 'emp-2'],
			'employee:emp-1',
			'employee:emp-1',
		)
		expect(result).toEqual([
			{ kind: 'employee', employee_id: 'emp-1' },
			{ kind: 'employee', employee_id: 'emp-2' },
		])
	})
})

describe('buildAssigneesForRemoval', () => {
	it('removes the targeted resource and keeps the others', () => {
		const result = buildAssigneesForRemoval(
			['emp-1', 'emp-2'],
			'employee:emp-1',
		)
		expect(result).toEqual([{ kind: 'employee', employee_id: 'emp-2' }])
	})

	it('changes nothing if the resource is not in the list', () => {
		const result = buildAssigneesForRemoval(['emp-2'], 'employee:emp-1')
		expect(result).toEqual([{ kind: 'employee', employee_id: 'emp-2' }])
	})
})

describe('computeTaskDropPatch — jour seul', () => {
	it('carries only the shifted starts_at/ends_at, with the full assignee list unchanged', () => {
		const result = computeTaskDropPatch({
			source: {
				entryId: 'wo-1',
				resourceId: 'employee:emp-1',
				date: '2026-08-10',
			},
			target: { resourceId: 'employee:emp-1', date: '2026-08-12' },
			entry: {
				startsAt: '2026-08-10T08:00:00+02:00',
				endsAt: '2026-08-10T10:00:00+02:00',
				employeeIds: ['emp-1'],
			},
			timeZone: TZ,
		})

		expect(result.changed).toBe(true)
		expect(result.body).toEqual({
			starts_at: new Date('2026-08-12T08:00:00+02:00').toISOString(),
			ends_at: new Date('2026-08-12T10:00:00+02:00').toISOString(),
			assignees: [{ kind: 'employee', employee_id: 'emp-1' }],
		})
	})
})

describe('computeTaskDropPatch — ligne seule', () => {
	it('carries only the assignee list, without starts_at/ends_at', () => {
		const result = computeTaskDropPatch({
			source: {
				entryId: 'wo-1',
				resourceId: 'employee:emp-1',
				date: '2026-08-10',
			},
			target: { resourceId: 'employee:emp-2', date: '2026-08-10' },
			entry: {
				startsAt: '2026-08-10T08:00:00+02:00',
				endsAt: '2026-08-10T10:00:00+02:00',
				employeeIds: ['emp-1'],
			},
			timeZone: TZ,
		})

		expect(result.changed).toBe(true)
		expect(result.body).toEqual({
			assignees: [{ kind: 'employee', employee_id: 'emp-2' }],
		})
		expect(result.body.starts_at).toBeUndefined()
		expect(result.body.ends_at).toBeUndefined()
	})
})

describe('computeTaskDropPatch — day and row at once', () => {
	it('carries the shifted starts_at/ends_at and the new assignee list, in a single object', () => {
		const result = computeTaskDropPatch({
			source: {
				entryId: 'wo-1',
				resourceId: 'employee:emp-1',
				date: '2026-08-10',
			},
			target: { resourceId: 'employee:emp-2', date: '2026-08-11' },
			entry: {
				startsAt: '2026-08-10T08:00:00+02:00',
				endsAt: '2026-08-10T10:00:00+02:00',
				employeeIds: ['emp-1'],
			},
			timeZone: TZ,
		})

		expect(result.changed).toBe(true)
		expect(result.body).toEqual({
			starts_at: new Date('2026-08-11T08:00:00+02:00').toISOString(),
			ends_at: new Date('2026-08-11T10:00:00+02:00').toISOString(),
			assignees: [{ kind: 'employee', employee_id: 'emp-2' }],
		})
	})
})

describe('computeTaskDropPatch — a drop with no effect', () => {
	it('changes nothing when the drop lands on the same row and the same date', () => {
		const result = computeTaskDropPatch({
			source: {
				entryId: 'wo-1',
				resourceId: 'employee:emp-1',
				date: '2026-08-10',
			},
			target: { resourceId: 'employee:emp-1', date: '2026-08-10' },
			entry: {
				startsAt: '2026-08-10T08:00:00+02:00',
				endsAt: '2026-08-10T10:00:00+02:00',
				employeeIds: ['emp-1'],
			},
			timeZone: TZ,
		})

		expect(result.changed).toBe(false)
		expect(result.body).toEqual({})
	})
})

describe('computeRemoveAssigneePatch', () => {
	it('produces the full list of remaining assignees', () => {
		const result = computeRemoveAssigneePatch({
			employeeIds: ['emp-1', 'emp-2'],
			resourceId: 'employee:emp-1',
		})

		expect(result.changed).toBe(true)
		expect(result.body).toEqual({
			assignees: [{ kind: 'employee', employee_id: 'emp-2' }],
		})
	})

	it('changes nothing if the targeted resource is not assigned', () => {
		const result = computeRemoveAssigneePatch({
			employeeIds: ['emp-2'],
			resourceId: 'employee:emp-1',
		})

		expect(result.changed).toBe(false)
		expect(result.body).toEqual({})
	})
})

describe('resourceIdFromAssigneeRef', () => {
	it('formats an employee AssigneeRef into a resource_id', () => {
		expect(
			resourceIdFromAssigneeRef({ kind: 'employee', employee_id: 'emp-1' }),
		).toBe('employee:emp-1')
	})

	it('formats a member AssigneeRef into a resource_id', () => {
		expect(
			resourceIdFromAssigneeRef({ kind: 'member', user_id: 'user-1' }),
		).toBe('member:user-1')
	})

	it('is the exact inverse of assigneeRefFromResourceId', () => {
		const resourceId = 'member:user-42'
		expect(
			resourceIdFromAssigneeRef(assigneeRefFromResourceId(resourceId)),
		).toBe(resourceId)
	})
})

describe('toggleAssignee', () => {
	it('adds a resource missing from the selection', () => {
		const result = toggleAssignee(
			[{ kind: 'employee', employee_id: 'emp-1' }],
			'employee:emp-2',
		)

		expect(result).toEqual([
			{ kind: 'employee', employee_id: 'emp-1' },
			{ kind: 'employee', employee_id: 'emp-2' },
		])
	})

	it('removes an already selected resource', () => {
		const result = toggleAssignee(
			[
				{ kind: 'employee', employee_id: 'emp-1' },
				{ kind: 'member', user_id: 'user-1' },
			],
			'employee:emp-1',
		)

		expect(result).toEqual([{ kind: 'member', user_id: 'user-1' }])
	})

	it('does not mutate the array it is given', () => {
		const assignees = [{ kind: 'employee' as const, employee_id: 'emp-1' }]
		toggleAssignee(assignees, 'employee:emp-2')
		expect(assignees).toEqual([{ kind: 'employee', employee_id: 'emp-1' }])
	})

	it('a subtask starts with no assignee — never inherited from the parent', () => {
		// Nothing to remove: an empty selection stays empty until a toggle is
		// called, see invariant 7 of the design doc.
		expect(toggleAssignee([], 'employee:emp-1')).toEqual([
			{ kind: 'employee', employee_id: 'emp-1' },
		])
	})
})
