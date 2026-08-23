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
		member_id: 'e-1',
		employee_id: 'e-1',
		display_name: 'Marie Leroy',
		weekly_contract_minutes: 2100,
	},
	{
		resource_id: 'r-2',
		member_id: 'e-2',
		employee_id: 'e-2',
		display_name: 'Paul Bernard',
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
		member_ids: ['e-1'],
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
		member_id: 'e-2',
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
	it('produces one column per day of the window', () => {
		const model = build([])

		expect(model.days.map((day) => day.date)).toEqual([
			'2026-03-02',
			'2026-03-03',
			'2026-03-04',
		])
	})

	it('marks the current day and the weekends', () => {
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

	it('separates all-day entries from timed ones', () => {
		const model = build([task({}), absence({})])

		const lundi = model.days[0]
		const mardi = model.days[1]
		expect(lundi?.timedEvents).toHaveLength(1)
		expect(lundi?.allDayEvents).toHaveLength(0)
		expect(mardi?.allDayEvents).toHaveLength(1)
		expect(mardi?.timedEvents).toHaveLength(0)
	})

	it('positions a segment according to the visible amplitude', () => {
		const model = build([task({})])
		const event = model.days[0]?.timedEvents[0]

		expect(event?.top).toBeGreaterThanOrEqual(0)
		expect(event?.height).toBeGreaterThan(0)
		expect(event?.top ?? 0).toBeLessThan(100)
	})

	// The label is expressed in the zone's local time: we check its shape and
	// the span it covers, not an absolute timestamp, so the test does not
	// depend on the zone of the machine running it.
	it("labels the segment's time range", () => {
		const model = build([task({})])
		const label = model.days[0]?.timedEvents[0]?.timeLabel ?? ''

		expect(label).toMatch(/^\d{2}:\d{2} – \d{2}:\d{2}$/)
		expect(minutesBetween(label)).toBe(120)
	})

	it("exposes the segment's duration, which drives the card's density", () => {
		const model = build([
			task({
				starts_at: '2026-03-02T10:00:00Z',
				ends_at: '2026-03-02T10:30:00Z',
			}),
		])

		expect(model.days[0]?.timedEvents[0]?.durationMinutes).toBe(30)
	})

	it('clamps the duration to the part of the segment falling on that day', () => {
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

	it('labels an all-day entry with no time range', () => {
		const model = build([absence({})])

		expect(model.days[1]?.allDayEvents[0]?.timeLabel).toBe('Journée entière')
	})

	it('spreads two overlapping entries into distinct columns', () => {
		const model = build([
			task({}),
			task({
				id: 't-2',
				title: 'Tonte',
				starts_at: '2026-03-02T10:00:00Z',
				ends_at: '2026-03-02T12:00:00Z',
				member_ids: ['e-2'],
			}),
		])

		const columns = model.days[0]?.timedEvents.map((event) => event.column)
		expect(columns).toEqual([0, 1])
		expect(model.days[0]?.timedEvents.every((e) => e.columnCount === 2)).toBe(
			true,
		)
	})

	it('cuts a multi-day entry into one segment per day', () => {
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

	it('resolves participants from the resources', () => {
		const model = build([task({ member_ids: ['e-1', 'e-2'] })])
		const event = model.days[0]?.timedEvents[0]

		expect(event?.attendees.map((attendee) => attendee.name)).toEqual([
			'Marie Leroy',
			'Paul Bernard',
		])
		expect(event?.attendees[0]?.initials).toBe('ML')
	})

	it('filters by kind and counts what is hidden', () => {
		const model = build([task({}), absence({})], { filter: 'leave' })

		expect(model.days[0]?.timedEvents).toHaveLength(0)
		expect(model.days[1]?.allDayEvents).toHaveLength(1)
		expect(model.hiddenCount).toBe(1)
	})

	it('filters by member, whole team when the selection is empty', () => {
		const entries = [task({}), absence({})]

		expect(build(entries, { memberIds: ['e-1'] }).hiddenCount).toBe(1)
		expect(build(entries, { memberIds: [] }).hiddenCount).toBe(0)
	})
})

describe('calendar amplitude', () => {
	it('covers the full 24 h whatever the entries', () => {
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

	it('produces one tick per hour, midnight to midnight', () => {
		const model = build([])

		expect(model.hourMarks).toHaveLength(25)
		expect(model.hourMarks[0]).toBe(0)
		expect(model.hourMarks.at(-1)).toBe(1440)
	})

	it('opens the view half an hour before the first timed entry', () => {
		const model = build([
			task({
				starts_at: '2026-03-02T10:00:00Z',
				ends_at: '2026-03-02T12:00:00Z',
			}),
		])
		const premier = model.days[0]?.timedEvents[0]?.startMinute ?? 0

		expect(model.scrollToMinute).toBe(premier - 30)
	})

	it('opens on the working day when the period has no timed entry', () => {
		expect(build([]).scrollToMinute).toBe(8 * 60)
		expect(build([absence({})]).scrollToMinute).toBe(8 * 60)
	})

	it('does not go below midnight for a very early entry', () => {
		const model = build([
			task({
				starts_at: '2026-03-02T00:10:00Z',
				ends_at: '2026-03-02T01:00:00Z',
			}),
		])

		expect(model.scrollToMinute).toBeGreaterThanOrEqual(0)
	})

	it('exposes the worked range, a typical day when there is no clocking', () => {
		expect(build([]).workingRange).toEqual({
			startMinute: 8 * 60,
			endMinute: 18 * 60,
		})
	})

	it('marks an event as recurring when its task carries a recurrence_id', () => {
		const model = build([task({ recurrence_id: 'recurrence-1' })])

		const event = model.days
			.flatMap((day) => day.timedEvents)
			.find((event) => event.entryId === 't-1')

		expect(event?.isRecurring).toBe(true)
	})

	it('does not mark a one-off task as recurring', () => {
		const model = build([task({ recurrence_id: null })])

		const event = model.days
			.flatMap((day) => day.timedEvents)
			.find((event) => event.entryId === 't-1')

		expect(event?.isRecurring).toBe(false)
	})
})

describe('hourMarks', () => {
	it('produces one tick per whole hour, bounds included', () => {
		expect(hourMarks({ startMinute: 480, endMinute: 660 })).toEqual([
			480, 540, 600, 660,
		])
	})

	it('starts at the next whole hour when the amplitude does not begin round', () => {
		expect(hourMarks({ startMinute: 490, endMinute: 620 })).toEqual([540, 600])
	})
})

describe('initialsOf', () => {
	it('takes the first two initials', () => {
		expect(initialsOf('Marie Leroy')).toBe('ML')
		expect(initialsOf('Paul')).toBe('P')
		expect(initialsOf('   ')).toBe('?')
	})
})
