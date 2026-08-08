import { describe, expect, it } from 'vitest'
import { buildMonthModel } from '#/pages/planning/lib/build-month-model'
import { computeMonthGridWindow } from '#/pages/planning/lib/window'
import type { PlanningEntry } from '#/pages/planning/types'

const TIME_ZONE = 'UTC'

/** Août 2026 : le 1er tombe un samedi, la grille démarre donc le lundi 27 juillet. */
const MONTH = '2026-08'
const GRID = computeMonthGridWindow('2026-08-15')

function task(overrides: Record<string, unknown> = {}) {
	return {
		kind: 'task',
		id: 't-1',
		title: 'Taille de haie',
		starts_at: '2026-08-05T09:30:00Z',
		ends_at: '2026-08-05T11:00:00Z',
		all_day: false,
		blocks_availability: true,
		child_count: 0,
		employee_ids: ['e-1'],
		labels: [],
		status: 'PLANNED',
		...overrides,
	} as PlanningEntry
}

function leave(overrides: Record<string, unknown> = {}) {
	return {
		kind: 'absence',
		id: 'a-1',
		employee_id: 'e-2',
		absence_kind: 'LEAVE',
		starts_at: '2026-08-05T00:00:00Z',
		ends_at: '2026-08-08T00:00:00Z',
		all_day: true,
		...overrides,
	} as PlanningEntry
}

function build(entries: PlanningEntry[], overrides = {}) {
	return buildMonthModel({
		from: GRID.from,
		to: GRID.to,
		month: MONTH,
		entries,
		timeZone: TIME_ZONE,
		today: '2026-08-09',
		filter: 'all',
		...overrides,
	})
}

function dayOf(model: ReturnType<typeof build>, date: string) {
	return model.weeks
		.flatMap((week) => week.days)
		.find((day) => day.date === date)
}

describe('computeMonthGridWindow', () => {
	it('couvre des semaines entières autour du mois', () => {
		expect(GRID).toEqual({ from: '2026-07-27', to: '2026-09-06' })
	})
})

describe('buildMonthModel', () => {
	it('découpe la fenêtre en semaines de sept jours', () => {
		const model = build([])

		expect(model.weeks).toHaveLength(6)
		expect(model.weeks.every((week) => week.days.length === 7)).toBe(true)
		expect(model.weekdayLabels).toEqual([
			'lun',
			'mar',
			'mer',
			'jeu',
			'ven',
			'sam',
			'dim',
		])
	})

	it('marque les jours des mois voisins, aujourd’hui et le week-end', () => {
		const model = build([])

		expect(dayOf(model, '2026-07-27')?.isOutsideMonth).toBe(true)
		expect(dayOf(model, '2026-08-03')?.isOutsideMonth).toBe(false)
		expect(dayOf(model, '2026-09-01')?.isOutsideMonth).toBe(true)
		expect(dayOf(model, '2026-08-09')?.isToday).toBe(true)
		expect(dayOf(model, '2026-08-08')?.isWeekend).toBe(true)
	})

	it('nomme le premier jour d’un mois avec son mois, comme Apple', () => {
		const model = build([])

		expect(dayOf(model, '2026-08-01')?.dayLabel).toMatch(/^1 août$/)
		expect(dayOf(model, '2026-08-02')?.dayLabel).toBe('2')
	})

	it('range une entrée horaire dans sa case, avec son heure de début', () => {
		const model = build([task()])
		const day = dayOf(model, '2026-08-05')

		expect(day?.entries).toHaveLength(1)
		expect(day?.entries[0]?.title).toBe('Taille de haie')
		expect(day?.entries[0]?.timeLabel).toMatch(/^\d{2}:\d{2}$/)
	})

	it('trie les entrées d’une case par heure de début', () => {
		const model = build([
			task({ id: 't-late', title: 'Tonte', starts_at: '2026-08-05T14:00:00Z' }),
			task({
				id: 't-early',
				title: 'Taille',
				starts_at: '2026-08-05T08:00:00Z',
			}),
		])

		expect(dayOf(model, '2026-08-05')?.entries.map((e) => e.title)).toEqual([
			'Taille',
			'Tonte',
		])
	})

	it('compte les entrées qui ne tiennent pas dans une case', () => {
		const model = build(
			Array.from({ length: 6 }, (_, index) =>
				task({
					id: `t-${index}`,
					starts_at: `2026-08-05T0${index + 1}:00:00Z`,
					ends_at: `2026-08-05T0${index + 2}:00:00Z`,
				}),
			),
		)
		const day = dayOf(model, '2026-08-05')

		expect(day?.entries).toHaveLength(4)
		expect(day?.hiddenCount).toBe(2)
	})

	it('étale une entrée à la journée sur les colonnes qu’elle couvre', () => {
		const model = build([leave()])
		const week = model.weeks.find((week) =>
			week.days.some((day) => day.date === '2026-08-05'),
		)
		const span = week?.spans[0]

		// Du mercredi 5 au vendredi 7 inclus : `ends_at` est la borne exclusive.
		expect(span?.startIndex).toBe(2)
		expect(span?.length).toBe(3)
		expect(span?.continuesBefore).toBe(false)
		expect(span?.continuesAfter).toBe(false)
	})

	it('coupe un congé à cheval sur deux semaines en un segment par semaine', () => {
		const model = build([
			leave({
				starts_at: '2026-08-07T00:00:00Z',
				ends_at: '2026-08-11T00:00:00Z',
			}),
		])
		const withSpans = model.weeks.filter((week) => week.spans.length > 0)

		expect(withSpans).toHaveLength(2)
		expect(withSpans[0]?.spans[0]?.continuesAfter).toBe(true)
		expect(withSpans[1]?.spans[0]?.continuesBefore).toBe(true)
	})

	it('empile deux bandeaux qui se chevauchent sur des rangs distincts', () => {
		const model = build([
			leave(),
			leave({ id: 'a-2', absence_kind: 'SICK', employee_id: 'e-3' }),
		])
		const week = model.weeks.find((week) => week.spans.length > 0)

		expect(week?.spans.map((span) => span.lane)).toEqual([0, 1])
		expect(week?.laneCount).toBe(2)
	})

	it('n’affiche pas une entrée à la journée comme ligne de case', () => {
		const model = build([leave()])

		expect(dayOf(model, '2026-08-05')?.entries).toHaveLength(0)
	})

	it('filtre par nature et compte ce qui est masqué', () => {
		const model = build([task(), leave()], { filter: 'task' })

		expect(model.hiddenByFilter).toBe(1)
		expect(model.weeks.every((week) => week.spans.length === 0)).toBe(true)
	})

	it('filtre par employé', () => {
		const model = build([task(), leave()], { employeeIds: ['e-1'] })

		expect(model.hiddenByFilter).toBe(1)
		expect(dayOf(model, '2026-08-05')?.entries).toHaveLength(1)
	})
})
