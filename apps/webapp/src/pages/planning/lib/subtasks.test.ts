import { describe, expect, it } from 'vitest'
import {
	canAddSubtask,
	formatWindowPlaceholder,
	formatWindowRange,
	resolveDisplayWindow,
} from '#/pages/planning/lib/subtasks'

describe('canAddSubtask', () => {
	it('allows adding a subtask under a root task', () => {
		expect(canAddSubtask({ parentTaskId: null })).toBe(true)
	})

	it('refuses adding a subtask under a task that is itself a subtask — the domain caps nesting at two levels', () => {
		expect(canAddSubtask({ parentTaskId: 'root-1' })).toBe(false)
	})
})

describe('resolveDisplayWindow', () => {
	const parent = {
		startsAt: '2026-08-10T07:00:00.000Z',
		endsAt: '2026-08-10T09:00:00.000Z',
	}

	it("returns the subtask's own window when it has one", () => {
		const resolved = resolveDisplayWindow(
			{
				startsAt: '2026-08-11T07:00:00.000Z',
				endsAt: '2026-08-11T09:00:00.000Z',
			},
			parent,
		)

		expect(resolved).toEqual({
			startsAt: '2026-08-11T07:00:00.000Z',
			endsAt: '2026-08-11T09:00:00.000Z',
			inherited: false,
		})
	})

	it("falls back to the parent's window when the subtask has none, flagged as inherited", () => {
		const resolved = resolveDisplayWindow(
			{ startsAt: null, endsAt: null },
			parent,
		)

		expect(resolved).toEqual({
			startsAt: parent.startsAt,
			endsAt: parent.endsAt,
			inherited: true,
		})
	})

	it('returns null when neither the task nor a parent carries a window', () => {
		const resolved = resolveDisplayWindow(
			{ startsAt: null, endsAt: null },
			null,
		)

		expect(resolved).toBeNull()
	})
})

describe('formatWindowPlaceholder', () => {
	it('formats a window as a French date-time range placeholder', () => {
		const placeholder = formatWindowPlaceholder(
			{
				startsAt: '2026-08-10T07:00:00.000Z',
				endsAt: '2026-08-10T09:00:00.000Z',
			},
			'Europe/Paris',
		)

		expect(placeholder).toBe('Hérite du parent : 10/08/2026 09:00 – 11:00')
	})

	it('returns null when there is no window to describe', () => {
		expect(formatWindowPlaceholder(null, 'Europe/Paris')).toBeNull()
	})
})

describe('formatWindowRange', () => {
	it('formats a same-day window with a bare end time', () => {
		expect(
			formatWindowRange(
				{
					startsAt: '2026-08-10T07:00:00.000Z',
					endsAt: '2026-08-10T09:00:00.000Z',
				},
				'Europe/Paris',
			),
		).toBe('10/08/2026 09:00 – 11:00')
	})

	it('formats a multi-day window with a full end date-time', () => {
		expect(
			formatWindowRange(
				{
					startsAt: '2026-08-10T07:00:00.000Z',
					endsAt: '2026-08-12T09:00:00.000Z',
				},
				'Europe/Paris',
			),
		).toBe('10/08/2026 09:00 – 12/08/2026 11:00')
	})

	it('an explicit allDay: false formats identically to the default — no regression for existing callers', () => {
		const window = {
			startsAt: '2026-08-10T07:00:00.000Z',
			endsAt: '2026-08-10T09:00:00.000Z',
		}
		expect(formatWindowRange(window, 'Europe/Paris', { allDay: false })).toBe(
			formatWindowRange(window, 'Europe/Paris'),
		)
	})

	it('formats a one-day all-day window as its single date, not a midnight-to-midnight range', () => {
		// A local Europe/Paris all-day task on 2026-08-10: starts_at is local
		// midnight that day, ends_at is *exclusive* local midnight the day
		// after (see `formatAllDayWindow`'s own doc) — 2026-08-10T00:00+02:00
		// and 2026-08-11T00:00+02:00.
		expect(
			formatWindowRange(
				{
					startsAt: '2026-08-09T22:00:00.000Z',
					endsAt: '2026-08-10T22:00:00.000Z',
				},
				'Europe/Paris',
				{ allDay: true },
			),
		).toBe('10/08/2026')
	})

	it('formats a multi-day all-day window with the real last day, one day before the exclusive end bound', () => {
		// 2026-08-10 through 2026-08-12 inclusive: ends_at is exclusive
		// midnight of 2026-08-13.
		expect(
			formatWindowRange(
				{
					startsAt: '2026-08-09T22:00:00.000Z',
					endsAt: '2026-08-12T22:00:00.000Z',
				},
				'Europe/Paris',
				{ allDay: true },
			),
		).toBe('10/08/2026 – 12/08/2026')
	})
})
