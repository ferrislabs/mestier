import { describe, expect, it } from 'vitest'
import {
	entryEmployeeIds,
	entryLabel,
	entryTone,
} from '#/pages/planning/lib/entries'
import type { PlanningEntry } from '#/pages/planning/types'

function workOrder(overrides: Partial<PlanningEntry> = {}): PlanningEntry {
	return {
		kind: 'work_order',
		id: 'wo-1',
		starts_at: '2026-08-10T08:00:00Z',
		ends_at: '2026-08-10T10:00:00Z',
		all_day: false,
		status: 'PLANNED',
		employee_ids: ['employee-1', 'employee-2'],
		customer_name: 'Client Dupont',
		context_label: 'Chantier toiture',
		...overrides,
	} as PlanningEntry
}

function absence(overrides: Partial<PlanningEntry> = {}): PlanningEntry {
	return {
		kind: 'absence',
		id: 'ab-1',
		starts_at: '2026-08-10T00:00:00Z',
		ends_at: '2026-08-11T00:00:00Z',
		all_day: true,
		absence_kind: 'LEAVE',
		employee_id: 'employee-1',
		...overrides,
	} as PlanningEntry
}

/** Simulates a future external-source `kind` the current union doesn't know about. */
function unknownKindEntry(): PlanningEntry {
	return {
		kind: 'external_source',
		id: 'ext-1',
		starts_at: '2026-08-10T08:00:00Z',
		ends_at: '2026-08-10T10:00:00Z',
		all_day: false,
	} as unknown as PlanningEntry
}

describe('entryEmployeeIds', () => {
	it('renvoie tous les employee_ids pour un chantier', () => {
		expect(entryEmployeeIds(workOrder())).toEqual(['employee-1', 'employee-2'])
	})

	it("renvoie l'unique employee_id pour une absence", () => {
		expect(entryEmployeeIds(absence())).toEqual(['employee-1'])
	})

	it('renvoie un tableau vide pour un kind inconnu, sans lever', () => {
		expect(() => entryEmployeeIds(unknownKindEntry())).not.toThrow()
		expect(entryEmployeeIds(unknownKindEntry())).toEqual([])
	})
})

describe('entryLabel', () => {
	it('utilise le titre du chantier quand il est renseigné', () => {
		expect(entryLabel(workOrder({ title: 'Réfection toiture' }))).toBe(
			'Réfection toiture',
		)
	})

	it('replie sur le nom du client quand le chantier n’a pas de titre', () => {
		expect(entryLabel(workOrder({ title: null }))).toBe('Client Dupont')
	})

	it("traduit le motif d'absence en français", () => {
		expect(entryLabel(absence({ absence_kind: 'LEAVE' }))).toBe('Congé')
		expect(entryLabel(absence({ absence_kind: 'SICK' }))).toBe('Arrêt maladie')
		expect(entryLabel(absence({ absence_kind: 'UNAVAILABLE' }))).toBe(
			'Indisponible',
		)
	})

	it('renvoie un libellé générique pour un kind inconnu, sans lever', () => {
		expect(() => entryLabel(unknownKindEntry())).not.toThrow()
		expect(entryLabel(unknownKindEntry())).toBe('Entrée')
	})
})

describe('entryTone', () => {
	it('distingue chantier, absence et inconnu', () => {
		expect(entryTone(workOrder())).toBe('work_order')
		expect(entryTone(absence())).toBe('absence')
		expect(entryTone(unknownKindEntry())).toBe('unknown')
	})
})
