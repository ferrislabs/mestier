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
})
