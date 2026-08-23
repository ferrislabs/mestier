import { describe, expect, it } from 'vitest'
import { buildMonthModel } from '#/pages/planning/lib/build-month-model'
import { computeMonthGridWindow } from '#/pages/planning/lib/window'
import type { PlanningEntry, PlanningResource } from '#/pages/planning/types'

const TIME_ZONE = 'UTC'

const RESOURCES: PlanningResource[] = [
	{
		resource_id: 'r-1',
		member_id: 'e-1',
		employee_id: 'e-1',
		display_name: 'Marie Leroy',
		weekly_contract_minutes: 2100,
	},
]

/** August 2026: the 1st is a Saturday, so the grid starts on Monday 27 July. */
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
		member_ids: ['e-1'],
		labels: [],
		status: 'PLANNED',
		...overrides,
	} as PlanningEntry
}

function leave(overrides: Record<string, unknown> = {}) {
	return {
		kind: 'absence',
		id: 'a-1',
		member_id: 'e-2',
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
		resources: RESOURCES,
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
	it('covers whole weeks around the month', () => {
		expect(GRID).toEqual({ from: '2026-07-27', to: '2026-09-06' })
	})
})

describe('buildMonthModel', () => {
	it('cuts the window into seven-day weeks', () => {
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

	it("marks neighbouring months' days, today and the weekend", () => {
		const model = build([])

		expect(dayOf(model, '2026-07-27')?.isOutsideMonth).toBe(true)
		expect(dayOf(model, '2026-08-03')?.isOutsideMonth).toBe(false)
		expect(dayOf(model, '2026-09-01')?.isOutsideMonth).toBe(true)
		expect(dayOf(model, '2026-08-09')?.isToday).toBe(true)
		expect(dayOf(model, '2026-08-08')?.isWeekend).toBe(true)
	})

	it("names a month's first day with its month, the way Apple does", () => {
		const model = build([])

		expect(dayOf(model, '2026-08-01')?.dayLabel).toMatch(/^1 août$/)
		expect(dayOf(model, '2026-08-02')?.dayLabel).toBe('2')
	})

	it('files a timed entry in its cell, with its start time', () => {
		const model = build([task()])
		const day = dayOf(model, '2026-08-05')

		expect(day?.entries).toHaveLength(1)
		expect(day?.entries[0]?.title).toBe('Taille de haie')
		expect(day?.entries[0]?.timeLabel).toMatch(/^\d{2}:\d{2}$/)
	})

	it("sorts a cell's entries by start time", () => {
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

	it('counts the entries that do not fit in a cell', () => {
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

	it('spreads an all-day entry over the columns it covers', () => {
		const model = build([leave()])
		const week = model.weeks.find((week) =>
			week.days.some((day) => day.date === '2026-08-05'),
		)
		const span = week?.spans[0]

		// Wednesday 5 through Friday 7 inclusive: `ends_at` is the exclusive bound.
		expect(span?.startIndex).toBe(2)
		expect(span?.length).toBe(3)
		expect(span?.continuesBefore).toBe(false)
		expect(span?.continuesAfter).toBe(false)
	})

	it('cuts a leave straddling two weeks into one segment per week', () => {
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

	it('stacks two overlapping banners on distinct ranks', () => {
		const model = build([
			leave(),
			leave({ id: 'a-2', absence_kind: 'SICK', member_id: 'e-3' }),
		])
		const week = model.weeks.find((week) => week.spans.length > 0)

		expect(week?.spans.map((span) => span.lane)).toEqual([0, 1])
		expect(week?.laneCount).toBe(2)
	})

	it('does not show an all-day entry as a cell row', () => {
		const model = build([leave()])

		expect(dayOf(model, '2026-08-05')?.entries).toHaveLength(0)
	})

	it('describes each row for the detail panel', () => {
		const model = build([task()])
		const detail = dayOf(model, '2026-08-05')?.entries[0]?.detail

		expect(detail?.title).toBe('Taille de haie')
		expect(detail?.dateLabel).toMatch(/^mercredi 5 août$/)
		expect(detail?.timeLabel).toMatch(/^\d{2}:\d{2} – \d{2}:\d{2}$/)
		expect(detail?.attendees.map((a) => a.name)).toEqual(['Marie Leroy'])
	})

	it('describes a banner as a full day, dated from its first day', () => {
		const model = build([leave()])
		const detail = model.weeks.find((week) => week.spans.length > 0)?.spans[0]
			?.detail

		expect(detail?.timeLabel).toBe('Journée entière')
		expect(detail?.dateLabel).toMatch(/^mercredi 5 août$/)
	})

	it('filters by kind and counts what is hidden', () => {
		const model = build([task(), leave()], { filter: 'task' })

		expect(model.hiddenByFilter).toBe(1)
		expect(model.weeks.every((week) => week.spans.length === 0)).toBe(true)
	})

	it('filters by member', () => {
		const model = build([task(), leave()], { memberIds: ['e-1'] })

		expect(model.hiddenByFilter).toBe(1)
		expect(dayOf(model, '2026-08-05')?.entries).toHaveLength(1)
	})

	it('marks a timed entry as recurring when its task carries a recurrence_id', () => {
		const model = build([task({ recurrence_id: 'recurrence-1' })])

		expect(dayOf(model, '2026-08-05')?.entries[0]?.isRecurring).toBe(true)
	})

	it('marks a banner as recurring when its all-day task carries a recurrence_id', () => {
		const model = build([
			task({
				all_day: true,
				starts_at: '2026-08-05T00:00:00Z',
				ends_at: '2026-08-08T00:00:00Z',
				recurrence_id: 'recurrence-1',
			}),
		])
		const span = model.weeks.find((week) => week.spans.length > 0)?.spans[0]

		expect(span?.isRecurring).toBe(true)
	})

	it('does not mark a one-off task as recurring', () => {
		const model = build([task()])

		expect(dayOf(model, '2026-08-05')?.entries[0]?.isRecurring).toBe(false)
	})
})
