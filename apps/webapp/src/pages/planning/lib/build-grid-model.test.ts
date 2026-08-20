import { describe, expect, it } from 'vitest'
import { FALLBACK_AMPLITUDE } from '#/pages/planning/lib/amplitude'
import { buildGridModel } from '#/pages/planning/lib/build-grid-model'
import type {
	PlanningEntry,
	PlanningResource,
	PlanningWorkTime,
} from '#/pages/planning/types'

const TZ = 'Europe/Paris'

function memberWithProfile(
	overrides: Partial<PlanningResource> = {},
): PlanningResource {
	return {
		resource_id: 'member:member-1',
		member_id: 'member-1',
		employee_id: 'employee-1',
		display_name: 'Alix Martin',
		hourly_rate_cents: 1500,
		weekly_contract_minutes: 2100,
		...overrides,
	}
}

function memberWithoutProfile(
	overrides: Partial<PlanningResource> = {},
): PlanningResource {
	return {
		resource_id: 'member:member-9',
		member_id: 'member-9',
		employee_id: null,
		display_name: 'Sans fiche',
		hourly_rate_cents: null,
		weekly_contract_minutes: 0,
		...overrides,
	}
}

function task(overrides: Partial<PlanningEntry> = {}): PlanningEntry {
	return {
		kind: 'task',
		labels: [],
		title: 'Tâche',
		blocks_availability: true,
		child_count: 0,
		id: 'wo-1',
		starts_at: '2026-08-10T08:00:00Z', // 10:00 Paris
		ends_at: '2026-08-10T10:00:00Z', // 12:00 Paris
		all_day: false,
		status: 'PLANNED',
		member_ids: ['member-1'],
		customer_name: 'Client Dupont',
		context_label: 'Projet toiture',
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
		member_id: 'member-1',
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
		member_id: 'member-1',
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
	it('one column per day of the window, one row per resource', () => {
		const model = buildGridModel({
			windowFrom: '2026-08-10',
			windowTo: '2026-08-12',
			timeZone: TZ,
			resources: [memberWithProfile()],
			entries: [],
			workTime: [],
		})

		expect(model.columns).toEqual(['2026-08-10', '2026-08-11', '2026-08-12'])
		expect(model.rows).toHaveLength(1)
		expect(model.rows[0]?.cells).toHaveLength(3)
	})

	it('positions a job segment in the right cell, at the right place', () => {
		const model = buildGridModel({
			windowFrom: '2026-08-10',
			windowTo: '2026-08-10',
			timeZone: TZ,
			resources: [memberWithProfile()],
			entries: [task()],
			workTime: [],
		})

		// Amplitude derived from the job alone: 10:00–12:00 Paris.
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

	it('draws the work ranges in the background, positioned on the amplitude', () => {
		const model = buildGridModel({
			windowFrom: '2026-08-10',
			windowTo: '2026-08-10',
			timeZone: TZ,
			resources: [memberWithProfile()],
			entries: [task()],
			workTime: [workTime()],
		})

		// Amplitude derived from the job (10-12h) and the work range (9-17h): 9h-17h.
		expect(model.amplitude).toEqual({ startMinute: 9 * 60, endMinute: 17 * 60 })
		const cell = model.rows[0]?.cells[0]
		expect(cell?.backgroundBars).toEqual([{ left: 0, width: 100 }])
	})

	it('stacks two jobs overlapping on the same day', () => {
		const model = buildGridModel({
			windowFrom: '2026-08-10',
			windowTo: '2026-08-10',
			timeZone: TZ,
			resources: [memberWithProfile()],
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

	it('rowCount is 1 when the cell is empty', () => {
		const model = buildGridModel({
			windowFrom: '2026-08-10',
			windowTo: '2026-08-10',
			timeZone: TZ,
			resources: [memberWithProfile()],
			entries: [],
			workTime: [],
		})

		expect(model.rows[0]?.cells[0]?.rowCount).toBe(1)
	})

	it('a multi-day absence lays a segment in each of its cells', () => {
		const model = buildGridModel({
			windowFrom: '2026-08-10',
			windowTo: '2026-08-12',
			timeZone: TZ,
			resources: [memberWithProfile()],
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

	it('a member with no employee profile still carries its assigned entries', () => {
		const model = buildGridModel({
			windowFrom: '2026-08-10',
			windowTo: '2026-08-10',
			timeZone: TZ,
			resources: [memberWithoutProfile()],
			entries: [task({ member_ids: ['member-9'] })],
			workTime: [],
		})

		expect(model.rows[0]?.cells[0]?.segments).toHaveLength(1)
	})

	it('falls back to the default amplitude when there is no data', () => {
		const model = buildGridModel({
			windowFrom: '2026-08-10',
			windowTo: '2026-08-10',
			timeZone: TZ,
			resources: [memberWithProfile()],
			entries: [],
			workTime: [],
		})

		expect(model.amplitude).toEqual(FALLBACK_AMPLITUDE)
	})

	it('an unknown entry kind does not break model building', () => {
		expect(() =>
			buildGridModel({
				windowFrom: '2026-08-10',
				windowTo: '2026-08-10',
				timeZone: TZ,
				resources: [memberWithProfile()],
				entries: [task(), unknownKindEntry()],
				workTime: [],
			}),
		).not.toThrow()

		const model = buildGridModel({
			windowFrom: '2026-08-10',
			windowTo: '2026-08-10',
			timeZone: TZ,
			resources: [memberWithProfile()],
			entries: [task(), unknownKindEntry()],
			workTime: [],
		})
		// The unknown-kind entry attaches to no known resource: it drops out
		// silently rather than breaking the render.
		expect(model.rows[0]?.cells[0]?.segments).toHaveLength(1)
		expect(model.rows[0]?.cells[0]?.segments[0]?.entryId).toBe('wo-1')
	})

	it("carries a task's labels through to the segment", () => {
		const model = buildGridModel({
			windowFrom: '2026-08-10',
			windowTo: '2026-08-10',
			timeZone: TZ,
			resources: [memberWithProfile()],
			entries: [
				task({
					labels: [
						{
							id: 'label-1',
							organization_id: 'org-1',
							name: 'Réunion',
							color: '#2563EB',
							created_at: '2026-01-01T00:00:00Z',
							updated_at: '2026-01-01T00:00:00Z',
						},
					],
				}),
			],
			workTime: [],
		})

		const segment = model.rows[0]?.cells[0]?.segments[0]
		expect(segment?.labels).toEqual([
			{ id: 'label-1', name: 'Réunion', color: '#2563EB' },
		])
	})

	it('an absence segment never carries labels', () => {
		const model = buildGridModel({
			windowFrom: '2026-08-10',
			windowTo: '2026-08-10',
			timeZone: TZ,
			resources: [memberWithProfile()],
			entries: [absence()],
			workTime: [],
		})

		const segment = model.rows[0]?.cells[0]?.segments[0]
		expect(segment?.labels).toEqual([])
	})
})
