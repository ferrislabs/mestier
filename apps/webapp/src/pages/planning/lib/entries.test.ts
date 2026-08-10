import { describe, expect, it } from 'vitest'
import {
	entryEmployeeIds,
	entryLabel,
	entryTone,
} from '#/pages/planning/lib/entries'
import type { PlanningEntry } from '#/pages/planning/types'

function task(overrides: Partial<PlanningEntry> = {}): PlanningEntry {
	return {
		kind: 'task',
		labels: [],
		title: 'Tâche',
		blocks_availability: true,
		child_count: 0,
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
	it('returns every employee_id for a job', () => {
		expect(entryEmployeeIds(task())).toEqual(['employee-1', 'employee-2'])
	})

	it('returns the single employee_id for an absence', () => {
		expect(entryEmployeeIds(absence())).toEqual(['employee-1'])
	})

	it('returns an empty array for an unknown kind, without throwing', () => {
		expect(() => entryEmployeeIds(unknownKindEntry())).not.toThrow()
		expect(entryEmployeeIds(unknownKindEntry())).toEqual([])
	})
})

describe('entryLabel', () => {
	it('uses the task title — always filled in, no more falling back to the customer', () => {
		expect(entryLabel(task({ title: 'Réfection toiture' }))).toBe(
			'Réfection toiture',
		)
	})

	it('translates the absence reason into French', () => {
		expect(entryLabel(absence({ absence_kind: 'LEAVE' }))).toBe('Congé')
		expect(entryLabel(absence({ absence_kind: 'SICK' }))).toBe('Arrêt maladie')
		expect(entryLabel(absence({ absence_kind: 'UNAVAILABLE' }))).toBe(
			'Indisponible',
		)
	})

	it('returns a generic label for an unknown kind, without throwing', () => {
		expect(() => entryLabel(unknownKindEntry())).not.toThrow()
		expect(entryLabel(unknownKindEntry())).toBe('Entrée')
	})
})

describe('entryTone', () => {
	it('distingue chantier, absence et inconnu', () => {
		expect(entryTone(task())).toBe('task')
		expect(entryTone(absence())).toBe('absence')
		expect(entryTone(unknownKindEntry())).toBe('unknown')
	})
})
