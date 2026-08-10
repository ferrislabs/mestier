import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	buildWarnings,
	conflictsForResource,
	warningDetail,
	warningTitle,
} from '#/pages/planning/lib/warnings'

type ConflictResponse = Schemas.ConflictResponse
type AvailabilityResponse = Schemas.AvailabilityResponse

function absenceConflict(
	overrides: Partial<Extract<ConflictResponse, { kind: 'absence' }>> = {},
): ConflictResponse {
	return {
		kind: 'absence',
		reason: 'LEAVE',
		note: 'De retour lundi',
		starts_at: '2026-08-10T00:00:00+02:00',
		ends_at: '2026-08-11T00:00:00+02:00',
		...overrides,
	}
}

function outsideWorkHoursConflict(): ConflictResponse {
	return {
		kind: 'outside_work_hours',
		starts_at: '2026-08-10T06:00:00Z',
		ends_at: '2026-08-10T08:00:00Z',
	}
}

function overlappingConflict(): ConflictResponse {
	return {
		kind: 'overlapping_task',
		task_id: 'wo-42',
		starts_at: '2026-08-10T08:00:00Z',
		ends_at: '2026-08-10T10:00:00Z',
	}
}

describe('conflictsForResource', () => {
	it("returns the targeted resource's conflicts", () => {
		const availability: AvailabilityResponse = {
			resources: [
				{
					resource_id: 'employee:employee-1',
					available: false,
					conflicts: [absenceConflict()],
				},
				{ resource_id: 'employee:employee-2', available: true, conflicts: [] },
			],
		}

		expect(conflictsForResource(availability, 'employee:employee-1')).toEqual([
			absenceConflict(),
		])
	})

	it('returns an empty list when the resource is absent from the response', () => {
		const availability: AvailabilityResponse = { resources: [] }
		expect(conflictsForResource(availability, 'employee:employee-1')).toEqual(
			[],
		)
	})
})

describe('buildWarnings — the three kinds coming from availability', () => {
	it('maps an absence conflict with its reason and its note', () => {
		const warnings = buildWarnings({
			conflicts: [absenceConflict()],
			resourceKind: 'employee',
		})

		expect(warnings).toEqual([
			{
				kind: 'absence',
				reason: 'LEAVE',
				note: 'De retour lundi',
				startsAt: '2026-08-10T00:00:00+02:00',
				endsAt: '2026-08-11T00:00:00+02:00',
			},
		])
	})

	it('maps an outside_work_hours conflict', () => {
		const warnings = buildWarnings({
			conflicts: [outsideWorkHoursConflict()],
			resourceKind: 'employee',
		})

		expect(warnings).toEqual([
			{
				kind: 'outside_work_hours',
				startsAt: '2026-08-10T06:00:00Z',
				endsAt: '2026-08-10T08:00:00Z',
			},
		])
	})

	it('maps an overlapping_task conflict with its task_id', () => {
		const warnings = buildWarnings({
			conflicts: [overlappingConflict()],
			resourceKind: 'employee',
		})

		expect(warnings).toEqual([
			{
				kind: 'overlapping_task',
				taskId: 'wo-42',
				startsAt: '2026-08-10T08:00:00Z',
				endsAt: '2026-08-10T10:00:00Z',
			},
		])
	})

	it('an absence with no note yields note: null rather than undefined', () => {
		const warnings = buildWarnings({
			conflicts: [absenceConflict({ note: undefined })],
			resourceKind: 'employee',
		})

		expect(warnings[0]).toMatchObject({ kind: 'absence', note: null })
	})
})

describe('buildWarnings — missing_employee_record inferred from the kind', () => {
	it('adds the warning when the resource is a member with no employee record', () => {
		const warnings = buildWarnings({ conflicts: [], resourceKind: 'member' })
		expect(warnings).toEqual([{ kind: 'missing_employee_record' }])
	})

	it('does not add it for an employee resource', () => {
		const warnings = buildWarnings({ conflicts: [], resourceKind: 'employee' })
		expect(warnings).toEqual([])
	})
})

describe('buildWarnings — several kinds in a single list', () => {
	it('combines an absence and missing_employee_record in the same dialog', () => {
		const warnings = buildWarnings({
			conflicts: [absenceConflict()],
			resourceKind: 'member',
		})

		expect(warnings.map((warning) => warning.kind)).toEqual([
			'absence',
			'missing_employee_record',
		])
	})

	it('combines all three conflict kinds at once', () => {
		const warnings = buildWarnings({
			conflicts: [
				absenceConflict(),
				outsideWorkHoursConflict(),
				overlappingConflict(),
			],
			resourceKind: 'employee',
		})

		expect(warnings.map((warning) => warning.kind)).toEqual([
			'absence',
			'outside_work_hours',
			'overlapping_task',
		])
	})
})

describe('warningTitle / warningDetail', () => {
	it('gives one title per conflict kind', () => {
		expect(warningTitle({ kind: 'missing_employee_record' })).toMatch(/fiche/i)
		expect(
			warningTitle({
				kind: 'overlapping_task',
				taskId: 'wo-1',
				startsAt: 'a',
				endsAt: 'b',
			}),
		).toMatch(/chantier/i)
		expect(
			warningTitle({ kind: 'outside_work_hours', startsAt: 'a', endsAt: 'b' }),
		).toMatch(/plages/i)
		expect(
			warningTitle({
				kind: 'absence',
				reason: 'SICK',
				note: null,
				startsAt: 'a',
				endsAt: 'b',
			}),
		).toMatch(/arrêt maladie/i)
	})

	it("exposes an absence's note as detail, and nothing when it is missing", () => {
		expect(
			warningDetail({
				kind: 'absence',
				reason: 'LEAVE',
				note: 'De retour lundi',
				startsAt: 'a',
				endsAt: 'b',
			}),
		).toBe('De retour lundi')
		expect(
			warningDetail({
				kind: 'absence',
				reason: 'LEAVE',
				note: null,
				startsAt: 'a',
				endsAt: 'b',
			}),
		).toBeNull()
	})

	it('flags that the hourly rate will need filling in for a missing record', () => {
		expect(warningDetail({ kind: 'missing_employee_record' })).toMatch(
			/taux horaire/i,
		)
	})
})
