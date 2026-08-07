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
		kind: 'overlapping_work_order',
		work_order_id: 'wo-42',
		starts_at: '2026-08-10T08:00:00Z',
		ends_at: '2026-08-10T10:00:00Z',
	}
}

describe('conflictsForResource', () => {
	it('renvoie les conflits du résident visé', () => {
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

	it("renvoie une liste vide quand la ressource n'apparaît pas dans la réponse", () => {
		const availability: AvailabilityResponse = { resources: [] }
		expect(conflictsForResource(availability, 'employee:employee-1')).toEqual(
			[],
		)
	})
})

describe('buildWarnings — les trois natures venant de availability', () => {
	it('mappe un conflit absence avec sa raison et sa note', () => {
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

	it('mappe un conflit outside_work_hours', () => {
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

	it('mappe un conflit overlapping_work_order avec son work_order_id', () => {
		const warnings = buildWarnings({
			conflicts: [overlappingConflict()],
			resourceKind: 'employee',
		})

		expect(warnings).toEqual([
			{
				kind: 'overlapping_work_order',
				workOrderId: 'wo-42',
				startsAt: '2026-08-10T08:00:00Z',
				endsAt: '2026-08-10T10:00:00Z',
			},
		])
	})

	it('une absence sans note produit note: null plutôt que undefined', () => {
		const warnings = buildWarnings({
			conflicts: [absenceConflict({ note: undefined })],
			resourceKind: 'employee',
		})

		expect(warnings[0]).toMatchObject({ kind: 'absence', note: null })
	})
})

describe('buildWarnings — missing_employee_record déduit du kind', () => {
	it("ajoute l'avertissement quand la ressource est un member sans fiche employé", () => {
		const warnings = buildWarnings({ conflicts: [], resourceKind: 'member' })
		expect(warnings).toEqual([{ kind: 'missing_employee_record' }])
	})

	it("ne l'ajoute pas pour une ressource employee", () => {
		const warnings = buildWarnings({ conflicts: [], resourceKind: 'employee' })
		expect(warnings).toEqual([])
	})
})

describe('buildWarnings — plusieurs natures dans une seule liste', () => {
	it('combine une absence et missing_employee_record dans le même dialogue', () => {
		const warnings = buildWarnings({
			conflicts: [absenceConflict()],
			resourceKind: 'member',
		})

		expect(warnings.map((warning) => warning.kind)).toEqual([
			'absence',
			'missing_employee_record',
		])
	})

	it('combine les trois natures de conflit à la fois', () => {
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
			'overlapping_work_order',
		])
	})
})

describe('warningTitle / warningDetail', () => {
	it('donne un titre par nature de conflit', () => {
		expect(warningTitle({ kind: 'missing_employee_record' })).toMatch(/fiche/i)
		expect(
			warningTitle({
				kind: 'overlapping_work_order',
				workOrderId: 'wo-1',
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

	it("expose la note d'une absence comme détail, et rien quand elle est absente", () => {
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

	it('signale que le taux horaire sera à compléter pour une fiche manquante', () => {
		expect(warningDetail({ kind: 'missing_employee_record' })).toMatch(
			/taux horaire/i,
		)
	})
})
