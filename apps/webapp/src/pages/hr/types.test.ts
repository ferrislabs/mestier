import { describe, expect, it } from 'vitest'
import type { Rhythm, WorkSlot } from '#/hooks/use-work-time'
import {
	accessState,
	addDaysIso,
	computeWeeklyGap,
	draftToRhythmSlots,
	draftToWorkSlots,
	emptyRhythmSlotDraft,
	emptyWorkSlotDraft,
	findOpenRhythm,
	formatDateFr,
	formatDurationMinutes,
	minutesToTime,
	parseDurationLabel,
	rhythmToDraft,
	timeToMinutes,
	validateRhythmDraft,
	validateWorkSlotsDraft,
	workSlotsToDraft,
} from '#/pages/hr/types'

describe('accessState', () => {
	it('is linkedAccount when the seat has an account', () => {
		expect(
			accessState({ account: { email: 'a@example.com', name: 'A' } }),
		).toBe('linkedAccount')
	})

	it('is none for a free or unlinked seat', () => {
		expect(accessState({ account: null })).toBe('none')
	})
})

describe('minutesToTime / timeToMinutes', () => {
	it('converts 0 minutes to 00:00', () => {
		expect(minutesToTime(0)).toBe('00:00')
	})

	it('converts 1440 minutes (end of day) to 24:00', () => {
		expect(minutesToTime(1440)).toBe('24:00')
	})

	it('converts an arbitrary minute', () => {
		expect(minutesToTime(480)).toBe('08:00')
		expect(minutesToTime(570)).toBe('09:30')
	})

	it('parses 00:00 as 0', () => {
		expect(timeToMinutes('00:00')).toBe(0)
	})

	it('parses 24:00 as 1440', () => {
		expect(timeToMinutes('24:00')).toBe(1440)
	})

	it('parses an arbitrary time', () => {
		expect(timeToMinutes('09:30')).toBe(570)
	})

	it('round-trips minutes -> time -> minutes without loss', () => {
		for (const minutes of [0, 1, 59, 60, 480, 719, 720, 1439, 1440]) {
			expect(timeToMinutes(minutesToTime(minutes))).toBe(minutes)
		}
	})

	it('rejects a malformed time', () => {
		expect(timeToMinutes('not-a-time')).toBeNull()
		expect(timeToMinutes('25:00')).toBeNull()
		expect(timeToMinutes('24:01')).toBeNull()
		expect(timeToMinutes('12:60')).toBeNull()
	})
})

describe('formatDurationMinutes / parseDurationLabel', () => {
	it('formate 0 minute', () => {
		expect(formatDurationMinutes(0)).toBe('0h00')
	})

	it('formats a duration carrying minutes', () => {
		expect(formatDurationMinutes(90)).toBe('1h30')
	})

	it('formats a negative duration with a sign', () => {
		expect(formatDurationMinutes(-30)).toBe('-0h30')
	})

	it('formats 1440 minutes (a full day)', () => {
		expect(formatDurationMinutes(1440)).toBe('24h00')
	})

	it('parses an "Xh" label', () => {
		expect(parseDurationLabel('35h')).toBe(2100)
	})

	it('parses an "XhYY" label', () => {
		expect(parseDurationLabel('35h30')).toBe(2130)
	})

	it('parses 0h00 as 0', () => {
		expect(parseDurationLabel('0h00')).toBe(0)
	})

	it('rejects an invalid label', () => {
		expect(parseDurationLabel('bogus')).toBeNull()
		expect(parseDurationLabel('12h60')).toBeNull()
		expect(parseDurationLabel('')).toBeNull()
	})

	it('round-trips label -> minutes -> label', () => {
		for (const label of ['0h00', '7h30', '35h00', '24h00']) {
			const minutes = parseDurationLabel(label)
			expect(minutes).not.toBeNull()
			expect(formatDurationMinutes(minutes as number)).toBe(label)
		}
	})
})

describe('computeWeeklyGap', () => {
	it('computes a zero gap when planned == contractual', () => {
		const gap = computeWeeklyGap(
			[{ startTime: '08:00', endTime: '12:00' }],
			240,
		)
		expect(gap).toEqual({
			plannedMinutes: 240,
			contractMinutes: 240,
			deltaMinutes: 0,
		})
	})

	it('computes a shortfall when the rhythm plans less than the contractual baseline', () => {
		// 4 days of 8h = 1920 minutes planned, baseline at 2100 (35h)
		const slots = Array.from({ length: 4 }, () => ({
			startTime: '08:00',
			endTime: '16:00',
		}))
		const gap = computeWeeklyGap(slots, 2100)
		expect(gap.plannedMinutes).toBe(1920)
		expect(gap.deltaMinutes).toBe(-180)
	})

	it('computes a surplus when the rhythm exceeds the contractual baseline', () => {
		const gap = computeWeeklyGap(
			[{ startTime: '08:00', endTime: '20:00' }],
			240,
		)
		expect(gap.deltaMinutes).toBe(480)
	})

	it('ignores invalid slots in the computation', () => {
		const gap = computeWeeklyGap(
			[
				{ startTime: '08:00', endTime: '12:00' },
				{ startTime: 'nope', endTime: '18:00' },
				{ startTime: '14:00', endTime: '10:00' },
			],
			0,
		)
		expect(gap.plannedMinutes).toBe(240)
	})

	it('handles a null contractual baseline (not filled in yet)', () => {
		const gap = computeWeeklyGap([], 0)
		expect(gap).toEqual({
			plannedMinutes: 0,
			contractMinutes: 0,
			deltaMinutes: 0,
		})
	})
})

describe('findOpenRhythm', () => {
	function rhythm(overrides: Partial<Rhythm>): Rhythm {
		return {
			id: 'rhythm-1',
			organization_id: 'org-1',
			employee_id: 'emp-1',
			effective_from: '2026-01-01',
			effective_to: null,
			slots: [],
			created_at: '2026-01-01T00:00:00Z',
			updated_at: '2026-01-01T00:00:00Z',
			...overrides,
		}
	}

	it('returns the version without an effective_to', () => {
		const open = rhythm({ id: 'open', effective_to: null })
		const closed = rhythm({ id: 'closed', effective_to: '2026-06-01' })
		expect(findOpenRhythm([closed, open])?.id).toBe('open')
	})

	it('returns null when no version is open', () => {
		const closed = rhythm({ effective_to: '2026-06-01' })
		expect(findOpenRhythm([closed])).toBeNull()
	})

	it('returns null for an empty list', () => {
		expect(findOpenRhythm([])).toBeNull()
	})
})

describe('rhythmToDraft / draftToRhythmSlots', () => {
	it('falls back to an empty form when there is no rhythm', () => {
		const draft = rhythmToDraft(null, '2026-08-07')
		expect(draft.effectiveFrom).toBe('2026-08-07')
		expect(draft.effectiveTo).toBe('')
		expect(draft.slots).toEqual([])
	})

	it("picks up the rhythm's existing slots", () => {
		const rhythm: Rhythm = {
			id: 'rhythm-1',
			organization_id: 'org-1',
			employee_id: 'emp-1',
			effective_from: '2026-01-01',
			effective_to: null,
			slots: [{ weekday: 1, starts_minute: 480, ends_minute: 720 }],
			created_at: '2026-01-01T00:00:00Z',
			updated_at: '2026-01-01T00:00:00Z',
		}
		const draft = rhythmToDraft(rhythm, '2026-08-07')
		expect(draft.effectiveFrom).toBe('2026-01-01')
		expect(draft.slots).toHaveLength(1)
		expect(draft.slots[0]).toMatchObject({
			weekday: 1,
			startTime: '08:00',
			endTime: '12:00',
		})
	})

	it('converts slot drafts into an API payload', () => {
		const slots = draftToRhythmSlots([
			{ key: 'a', weekday: 1, startTime: '08:00', endTime: '12:00' },
			{ key: 'b', weekday: 3, startTime: '14:00', endTime: '18:00' },
		])
		expect(slots).toEqual([
			{ weekday: 1, starts_minute: 480, ends_minute: 720 },
			{ weekday: 3, starts_minute: 840, ends_minute: 1080 },
		])
	})

	it('excludes slots whose time is invalid', () => {
		const slots = draftToRhythmSlots([
			{ key: 'a', weekday: 1, startTime: 'bogus', endTime: '12:00' },
		])
		expect(slots).toEqual([])
	})

	it('creates an empty slot prefilled for a given weekday', () => {
		const draft = emptyRhythmSlotDraft(2)
		expect(draft.weekday).toBe(2)
		expect(typeof draft.key).toBe('string')
		expect(draft.key.length).toBeGreaterThan(0)
	})
})

describe('workSlotsToDraft / draftToWorkSlots', () => {
	it('sorts ranges by date then by start time', () => {
		const slots: WorkSlot[] = [
			{
				id: 'b',
				organization_id: 'org-1',
				member_id: 'member-1',
				work_date: '2026-08-10',
				starts_minute: 780,
				ends_minute: 1020,
			},
			{
				id: 'a',
				organization_id: 'org-1',
				member_id: 'member-1',
				work_date: '2026-08-10',
				starts_minute: 480,
				ends_minute: 720,
			},
		]
		const draft = workSlotsToDraft(slots, {
			from: '2026-08-03',
			to: '2026-08-10',
		})
		expect(draft.from).toBe('2026-08-03')
		expect(draft.to).toBe('2026-08-10')
		expect(draft.slots.map((slot) => slot.key)).toEqual(['a', 'b'])
	})

	it('converts drafts into an API payload', () => {
		const slots = draftToWorkSlots([
			{
				key: 'a',
				workDate: '2026-08-10',
				startTime: '08:00',
				endTime: '12:00',
			},
		])
		expect(slots).toEqual([
			{ work_date: '2026-08-10', starts_minute: 480, ends_minute: 720 },
		])
	})

	it('creates an empty range prefilled for a given date', () => {
		const draft = emptyWorkSlotDraft('2026-08-10')
		expect(draft.workDate).toBe('2026-08-10')
	})
})

describe('validateRhythmDraft', () => {
	it('reports no error for a valid form', () => {
		const errors = validateRhythmDraft(
			{
				effectiveFrom: '2026-08-07',
				effectiveTo: '',
				slots: [{ key: 'a', weekday: 1, startTime: '08:00', endTime: '12:00' }],
			},
			null,
		)
		expect(errors).toEqual([])
	})

	it('reports a slot ending before it starts', () => {
		const errors = validateRhythmDraft(
			{
				effectiveFrom: '2026-08-07',
				effectiveTo: '',
				slots: [{ key: 'a', weekday: 1, startTime: '12:00', endTime: '08:00' }],
			},
			null,
		)
		expect(errors.some((error) => error.key === 'a')).toBe(true)
	})

	it('reports a start date earlier than the current version', () => {
		const errors = validateRhythmDraft(
			{ effectiveFrom: '2026-01-01', effectiveTo: '', slots: [] },
			'2026-06-01',
		)
		expect(errors.some((error) => error.key === 'effectiveFrom')).toBe(true)
	})

	it('accepts a start date equal to the current version (in-place edit)', () => {
		const errors = validateRhythmDraft(
			{ effectiveFrom: '2026-06-01', effectiveTo: '', slots: [] },
			'2026-06-01',
		)
		expect(errors.some((error) => error.key === 'effectiveFrom')).toBe(false)
	})
})

describe('validateWorkSlotsDraft', () => {
	it('reports no error for a valid form', () => {
		const errors = validateWorkSlotsDraft({
			from: '2026-08-03',
			to: '2026-08-10',
			slots: [
				{
					key: 'a',
					workDate: '2026-08-05',
					startTime: '08:00',
					endTime: '12:00',
				},
			],
		})
		expect(errors).toEqual([])
	})

	it('reports a dated range outside the selected period', () => {
		const errors = validateWorkSlotsDraft({
			from: '2026-08-03',
			to: '2026-08-10',
			slots: [
				{
					key: 'a',
					workDate: '2026-09-01',
					startTime: '08:00',
					endTime: '12:00',
				},
			],
		})
		expect(errors.some((error) => error.key === 'a')).toBe(true)
	})

	it('reports an inverted period', () => {
		const errors = validateWorkSlotsDraft({
			from: '2026-08-10',
			to: '2026-08-03',
			slots: [],
		})
		expect(errors.some((error) => error.key === 'range')).toBe(true)
	})
})

describe('addDaysIso', () => {
	it('adds days to an ISO date', () => {
		expect(addDaysIso('2026-08-07', 1)).toBe('2026-08-08')
	})

	it('crosses a month boundary', () => {
		expect(addDaysIso('2026-08-31', 1)).toBe('2026-09-01')
	})

	it('accepts a negative offset', () => {
		expect(addDaysIso('2026-08-07', -7)).toBe('2026-07-31')
	})
})

describe('formatDateFr', () => {
	it('formats an ISO date as dd/mm/yyyy', () => {
		expect(formatDateFr('2026-08-07')).toBe('07/08/2026')
	})
})
