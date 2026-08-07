import { describe, expect, it } from 'vitest'
import { FALLBACK_AMPLITUDE } from '#/pages/planning/lib/amplitude'
import { buildGridModel } from '#/pages/planning/lib/build-grid-model'
import type {
	PlanningEntry,
	PlanningResource,
	PlanningWorkTime,
} from '#/pages/planning/types'

const TZ = 'Europe/Paris'

function employeeResource(
	overrides: Partial<PlanningResource> = {},
): PlanningResource {
	return {
		resource_id: 'employee:employee-1',
		kind: 'employee',
		employee_id: 'employee-1',
		user_id: null,
		display_name: 'Alix Martin',
		hourly_rate_cents: 1500,
		weekly_contract_minutes: 2100,
		...overrides,
	}
}

function memberResource(
	overrides: Partial<PlanningResource> = {},
): PlanningResource {
	return {
		resource_id: 'member:user-9',
		kind: 'member',
		employee_id: null,
		user_id: 'user-9',
		display_name: 'Sans fiche',
		hourly_rate_cents: null,
		weekly_contract_minutes: 0,
		...overrides,
	}
}

function task(overrides: Partial<PlanningEntry> = {}): PlanningEntry {
	return {
		kind: 'task',
		title: 'Tâche',
		blocks_availability: true,
		child_count: 0,
		id: 'wo-1',
		starts_at: '2026-08-10T08:00:00Z', // 10:00 Paris
		ends_at: '2026-08-10T10:00:00Z', // 12:00 Paris
		all_day: false,
		status: 'PLANNED',
		employee_ids: ['employee-1'],
		customer_name: 'Client Dupont',
		context_label: 'Chantier toiture',
		...overrides,
	} as PlanningEntry
}

function absence(overrides: Partial<PlanningEntry> = {}): PlanningEntry {
	return {
		kind: 'absence',
		id: 'ab-1',
		starts_at: '2026-08-10T00:00:00+02:00',
		ends_at: '2026-08-11T00:00:00+02:00',
		all_day: true,
		absence_kind: 'LEAVE',
		employee_id: 'employee-1',
		...overrides,
	} as PlanningEntry
}

function unknownKindEntry(): PlanningEntry {
	return {
		kind: 'external_source',
		id: 'ext-1',
		starts_at: '2026-08-10T08:00:00Z',
		ends_at: '2026-08-10T10:00:00Z',
		all_day: false,
	} as unknown as PlanningEntry
}

function workTime(overrides: Partial<PlanningWorkTime> = {}): PlanningWorkTime {
	return {
		employee_id: 'employee-1',
		days: [
			{
				date: '2026-08-10',
				intervals: [{ starts_minute: 9 * 60, ends_minute: 17 * 60 }],
			},
		],
		...overrides,
	}
}

describe('buildGridModel', () => {
	it('une colonne par jour de la fenêtre, une ligne par ressource', () => {
		const model = buildGridModel({
			windowFrom: '2026-08-10',
			windowTo: '2026-08-12',
			timeZone: TZ,
			resources: [employeeResource()],
			entries: [],
			workTime: [],
		})

		expect(model.columns).toEqual(['2026-08-10', '2026-08-11', '2026-08-12'])
		expect(model.rows).toHaveLength(1)
		expect(model.rows[0]?.cells).toHaveLength(3)
	})

	it('positionne un segment de chantier dans la bonne cellule, à la bonne place', () => {
		const model = buildGridModel({
			windowFrom: '2026-08-10',
			windowTo: '2026-08-10',
			timeZone: TZ,
			resources: [employeeResource()],
			entries: [task()],
			workTime: [],
		})

		// Amplitude dérivée du seul chantier : 10:00–12:00 Paris.
		expect(model.amplitude).toEqual({
			startMinute: 10 * 60,
			endMinute: 12 * 60,
		})
		const cell = model.rows[0]?.cells[0]
		expect(cell?.segments).toHaveLength(1)
		expect(cell?.segments[0]).toMatchObject({
			entryId: 'wo-1',
			tone: 'task',
			row: 0,
			left: 0,
			width: 100,
		})
	})

	it('affiche les plages de travail en fond, positionnées sur l’amplitude', () => {
		const model = buildGridModel({
			windowFrom: '2026-08-10',
			windowTo: '2026-08-10',
			timeZone: TZ,
			resources: [employeeResource()],
			entries: [task()],
			workTime: [workTime()],
		})

		// Amplitude dérivée du chantier (10-12h) et de la plage de travail (9-17h) : 9h-17h.
		expect(model.amplitude).toEqual({ startMinute: 9 * 60, endMinute: 17 * 60 })
		const cell = model.rows[0]?.cells[0]
		expect(cell?.backgroundBars).toEqual([{ left: 0, width: 100 }])
	})

	it('empile deux chantiers qui se chevauchent le même jour', () => {
		const model = buildGridModel({
			windowFrom: '2026-08-10',
			windowTo: '2026-08-10',
			timeZone: TZ,
			resources: [employeeResource()],
			entries: [
				task({
					id: 'wo-1',
					starts_at: '2026-08-10T08:00:00Z',
					ends_at: '2026-08-10T10:00:00Z',
				}),
				task({
					id: 'wo-2',
					starts_at: '2026-08-10T09:00:00Z',
					ends_at: '2026-08-10T11:00:00Z',
				}),
			],
			workTime: [],
		})

		const cell = model.rows[0]?.cells[0]
		expect(cell?.rowCount).toBe(2)
		const rows = cell?.segments.map((s) => s.row).sort()
		expect(rows).toEqual([0, 1])
	})

	it('rowCount vaut 1 quand la cellule est vide', () => {
		const model = buildGridModel({
			windowFrom: '2026-08-10',
			windowTo: '2026-08-10',
			timeZone: TZ,
			resources: [employeeResource()],
			entries: [],
			workTime: [],
		})

		expect(model.rows[0]?.cells[0]?.rowCount).toBe(1)
	})

	it('une absence de plusieurs jours pose un segment dans chacune de ses cellules', () => {
		const model = buildGridModel({
			windowFrom: '2026-08-10',
			windowTo: '2026-08-12',
			timeZone: TZ,
			resources: [employeeResource()],
			entries: [
				absence({
					starts_at: '2026-08-10T00:00:00+02:00',
					ends_at: '2026-08-13T00:00:00+02:00',
				}),
			],
			workTime: [],
		})

		for (const cell of model.rows[0]?.cells ?? []) {
			expect(cell.segments).toHaveLength(1)
			expect(cell.segments[0]).toMatchObject({
				tone: 'absence',
				left: 0,
				width: 100,
			})
			expect(cell.hasAbsence).toBe(true)
		}
	})

	it('une ressource member sans fiche employé ne porte jamais d’entrées', () => {
		const model = buildGridModel({
			windowFrom: '2026-08-10',
			windowTo: '2026-08-10',
			timeZone: TZ,
			resources: [memberResource()],
			entries: [task({ employee_ids: ['employee-1'] })],
			workTime: [],
		})

		expect(model.rows[0]?.cells[0]?.segments).toEqual([])
	})

	it('replie sur l’amplitude par défaut quand il n’y a aucune donnée', () => {
		const model = buildGridModel({
			windowFrom: '2026-08-10',
			windowTo: '2026-08-10',
			timeZone: TZ,
			resources: [employeeResource()],
			entries: [],
			workTime: [],
		})

		expect(model.amplitude).toEqual(FALLBACK_AMPLITUDE)
	})

	it('un kind d’entrée inconnu ne casse pas la construction du modèle', () => {
		expect(() =>
			buildGridModel({
				windowFrom: '2026-08-10',
				windowTo: '2026-08-10',
				timeZone: TZ,
				resources: [employeeResource()],
				entries: [task(), unknownKindEntry()],
				workTime: [],
			}),
		).not.toThrow()

		const model = buildGridModel({
			windowFrom: '2026-08-10',
			windowTo: '2026-08-10',
			timeZone: TZ,
			resources: [employeeResource()],
			entries: [task(), unknownKindEntry()],
			workTime: [],
		})
		// L'entrée de kind inconnu n'est rattachable à aucune ressource connue :
		// elle disparaît silencieusement plutôt que de casser le rendu.
		expect(model.rows[0]?.cells[0]?.segments).toHaveLength(1)
		expect(model.rows[0]?.cells[0]?.segments[0]?.entryId).toBe('wo-1')
	})
})
