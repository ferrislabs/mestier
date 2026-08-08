import { describe, expect, it } from 'vitest'
import {
	buildCalendarModel,
	hourMarks,
	initialsOf,
} from '#/pages/planning/lib/build-calendar-model'
import type { PlanningEntry, PlanningResource } from '#/pages/planning/types'

const TIME_ZONE = 'UTC'

const RESOURCES: PlanningResource[] = [
	{
		resource_id: 'r-1',
		employee_id: 'e-1',
		display_name: 'Marie Leroy',
		kind: 'employee',
		weekly_contract_minutes: 2100,
	},
	{
		resource_id: 'r-2',
		employee_id: 'e-2',
		display_name: 'Paul Bernard',
		kind: 'employee',
		weekly_contract_minutes: 2100,
	},
]

function task(overrides: Partial<Extract<PlanningEntry, { kind: 'task' }>>) {
	return {
		kind: 'task',
		id: 't-1',
		title: 'Taille de haie',
		starts_at: '2026-03-02T09:00:00Z',
		ends_at: '2026-03-02T11:00:00Z',
		all_day: false,
		blocks_availability: true,
		child_count: 0,
		employee_ids: ['e-1'],
		labels: [],
		status: 'PLANNED',
		...overrides,
	} as PlanningEntry
}

function absence(
	overrides: Partial<Extract<PlanningEntry, { kind: 'absence' }>>,
) {
	return {
		kind: 'absence',
		id: 'a-1',
		employee_id: 'e-2',
		absence_kind: 'LEAVE',
		starts_at: '2026-03-03T00:00:00Z',
		ends_at: '2026-03-04T00:00:00Z',
		all_day: true,
		...overrides,
	} as PlanningEntry
}

function build(entries: PlanningEntry[], overrides = {}) {
	return buildCalendarModel({
		from: '2026-03-02',
		to: '2026-03-04',
		entries,
		resources: RESOURCES,
		workTime: [],
		timeZone: TIME_ZONE,
		today: '2026-03-03',
		filter: 'all',
		...overrides,
	})
}

function minutesBetween(label: string): number {
	const [start, end] = label.split(' – ')
	const toMinutes = (value: string) => {
		const [hours, minutes] = value.split(':').map(Number)
		return (hours ?? 0) * 60 + (minutes ?? 0)
	}
	return toMinutes(end ?? '') - toMinutes(start ?? '')
}

describe('buildCalendarModel', () => {
	it('produit une colonne par jour de la fenêtre', () => {
		const model = build([])

		expect(model.days.map((day) => day.date)).toEqual([
			'2026-03-02',
			'2026-03-03',
			'2026-03-04',
		])
	})

	it('marque le jour courant et les fins de semaine', () => {
		const model = buildCalendarModel({
			from: '2026-03-06',
			to: '2026-03-08',
			entries: [],
			resources: RESOURCES,
			workTime: [],
			timeZone: TIME_ZONE,
			today: '2026-03-06',
			filter: 'all',
		})

		expect(model.days.map((day) => day.isToday)).toEqual([true, false, false])
		expect(model.days.map((day) => day.isWeekend)).toEqual([false, true, true])
	})

	it('sépare les entrées à la journée des entrées horaires', () => {
		const model = build([task({}), absence({})])

		const lundi = model.days[0]
		const mardi = model.days[1]
		expect(lundi?.timedEvents).toHaveLength(1)
		expect(lundi?.allDayEvents).toHaveLength(0)
		expect(mardi?.allDayEvents).toHaveLength(1)
		expect(mardi?.timedEvents).toHaveLength(0)
	})

	it("positionne un segment selon l'amplitude visible", () => {
		const model = build([task({})])
		const event = model.days[0]?.timedEvents[0]

		expect(event?.top).toBeGreaterThanOrEqual(0)
		expect(event?.height).toBeGreaterThan(0)
		expect(event?.top ?? 0).toBeLessThan(100)
	})

	// Le libellé est exprimé en heure locale du fuseau : on vérifie sa forme et
	// la durée qu'il couvre, pas un horodatage absolu, pour que le test ne
	// dépende pas du fuseau de la machine qui l'exécute.
	it('libelle la plage horaire du segment', () => {
		const model = build([task({})])
		const label = model.days[0]?.timedEvents[0]?.timeLabel ?? ''

		expect(label).toMatch(/^\d{2}:\d{2} – \d{2}:\d{2}$/)
		expect(minutesBetween(label)).toBe(120)
	})

	it('expose la durée du segment, qui pilote la densité de la carte', () => {
		const model = build([
			task({
				starts_at: '2026-03-02T10:00:00Z',
				ends_at: '2026-03-02T10:30:00Z',
			}),
		])

		expect(model.days[0]?.timedEvents[0]?.durationMinutes).toBe(30)
	})

	it('borne la durée à la portion du segment qui tombe sur le jour', () => {
		const model = build([
			task({
				starts_at: '2026-03-02T22:00:00Z',
				ends_at: '2026-03-03T02:00:00Z',
			}),
		])

		const total =
			(model.days[0]?.timedEvents[0]?.durationMinutes ?? 0) +
			(model.days[1]?.timedEvents[0]?.durationMinutes ?? 0)
		expect(total).toBe(240)
	})

	it('libelle une entrée à la journée sans plage horaire', () => {
		const model = build([absence({})])

		expect(model.days[1]?.allDayEvents[0]?.timeLabel).toBe('Journée entière')
	})

	it('répartit deux entrées qui se chevauchent en colonnes distinctes', () => {
		const model = build([
			task({}),
			task({
				id: 't-2',
				title: 'Tonte',
				starts_at: '2026-03-02T10:00:00Z',
				ends_at: '2026-03-02T12:00:00Z',
				employee_ids: ['e-2'],
			}),
		])

		const columns = model.days[0]?.timedEvents.map((event) => event.column)
		expect(columns).toEqual([0, 1])
		expect(model.days[0]?.timedEvents.every((e) => e.columnCount === 2)).toBe(
			true,
		)
	})

	it('découpe une entrée sur plusieurs jours en un segment par jour', () => {
		const model = build([
			task({
				starts_at: '2026-03-02T16:00:00Z',
				ends_at: '2026-03-03T10:00:00Z',
			}),
		])

		expect(model.days[0]?.timedEvents).toHaveLength(1)
		expect(model.days[1]?.timedEvents).toHaveLength(1)
		expect(model.days[0]?.timedEvents[0]?.key).not.toBe(
			model.days[1]?.timedEvents[0]?.key,
		)
	})

	it('résout les participants depuis les ressources', () => {
		const model = build([task({ employee_ids: ['e-1', 'e-2'] })])
		const event = model.days[0]?.timedEvents[0]

		expect(event?.attendees.map((attendee) => attendee.name)).toEqual([
			'Marie Leroy',
			'Paul Bernard',
		])
		expect(event?.attendees[0]?.initials).toBe('ML')
	})

	it('filtre par nature et compte ce qui est masqué', () => {
		const model = build([task({}), absence({})], { filter: 'leave' })

		expect(model.days[0]?.timedEvents).toHaveLength(0)
		expect(model.days[1]?.allDayEvents).toHaveLength(1)
		expect(model.hiddenCount).toBe(1)
	})

	it('filtre par employé, toute équipe quand la sélection est vide', () => {
		const entries = [task({}), absence({})]

		expect(build(entries, { employeeIds: ['e-1'] }).hiddenCount).toBe(1)
		expect(build(entries, { employeeIds: [] }).hiddenCount).toBe(0)
	})
})

describe('amplitude du calendrier', () => {
	it('couvre les 24 h quelles que soient les entrées', () => {
		const court = build([
			task({
				starts_at: '2026-03-02T10:00:00Z',
				ends_at: '2026-03-02T12:00:00Z',
			}),
		])
		const vide = build([])

		expect(court.amplitude).toEqual({ startMinute: 0, endMinute: 1440 })
		expect(vide.amplitude).toEqual({ startMinute: 0, endMinute: 1440 })
	})

	it('produit une marque par heure, minuit à minuit', () => {
		const model = build([])

		expect(model.hourMarks).toHaveLength(25)
		expect(model.hourMarks[0]).toBe(0)
		expect(model.hourMarks.at(-1)).toBe(1440)
	})

	it('ouvre la vue une demi-heure avant la première entrée horaire', () => {
		const model = build([
			task({
				starts_at: '2026-03-02T10:00:00Z',
				ends_at: '2026-03-02T12:00:00Z',
			}),
		])
		const premier = model.days[0]?.timedEvents[0]?.startMinute ?? 0

		expect(model.scrollToMinute).toBe(premier - 30)
	})

	it("ouvre sur la journée de travail quand la période n'a aucune entrée horaire", () => {
		expect(build([]).scrollToMinute).toBe(8 * 60)
		expect(build([absence({})]).scrollToMinute).toBe(8 * 60)
	})

	it('ne descend pas sous minuit pour une entrée très matinale', () => {
		const model = build([
			task({
				starts_at: '2026-03-02T00:10:00Z',
				ends_at: '2026-03-02T01:00:00Z',
			}),
		])

		expect(model.scrollToMinute).toBeGreaterThanOrEqual(0)
	})

	it('expose la plage travaillée, journée type à défaut de pointage', () => {
		expect(build([]).workingRange).toEqual({
			startMinute: 8 * 60,
			endMinute: 18 * 60,
		})
	})
})

describe('hourMarks', () => {
	it('produit une marque par heure pleine, bornes comprises', () => {
		expect(hourMarks({ startMinute: 480, endMinute: 660 })).toEqual([
			480, 540, 600, 660,
		])
	})

	it("démarre à l'heure pleine suivante quand l'amplitude ne commence pas rond", () => {
		expect(hourMarks({ startMinute: 490, endMinute: 620 })).toEqual([540, 600])
	})
})

describe('initialsOf', () => {
	it('prend les deux premières initiales', () => {
		expect(initialsOf('Marie Leroy')).toBe('ML')
		expect(initialsOf('Paul')).toBe('P')
		expect(initialsOf('   ')).toBe('?')
	})
})
