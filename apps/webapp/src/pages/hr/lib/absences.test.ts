import { describe, expect, it } from 'vitest'
import {
	absenceToDraft,
	calendarSelectionToRange,
	draftToCreateAbsenceRequest,
	draftToUpdateAbsenceRequest,
	emptyAbsenceDraft,
	rangeToCalendarSelection,
	validateAbsenceDraft,
} from '#/pages/hr/lib/absences'

const TZ = 'Europe/Paris'

describe('emptyAbsenceDraft', () => {
	it('prefills a single-day full-day leave for a given member', () => {
		const draft = emptyAbsenceDraft('member-1', '2026-08-10')
		expect(draft.memberId).toBe('member-1')
		expect(draft.kind).toBe('LEAVE')
		expect(draft.allDay).toBe(true)
		expect(draft.range).toEqual({ from: '2026-08-10', to: '2026-08-10' })
	})
})

describe('validateAbsenceDraft', () => {
	it('reports the missing member on creation', () => {
		const draft = emptyAbsenceDraft('', '2026-08-10')
		const errors = validateAbsenceDraft(draft, { requireMember: true })
		expect(errors).toContain('Personne requise')
	})

	it('does not require the member when editing', () => {
		const draft = emptyAbsenceDraft('member-1', '2026-08-10')
		const errors = validateAbsenceDraft(draft, { requireMember: false })
		expect(errors).not.toContain('Personne requise')
	})

	it('accepts a single-day full-day absence (end == start)', () => {
		const draft = emptyAbsenceDraft('member-1', '2026-08-10')
		expect(validateAbsenceDraft(draft)).toEqual([])
	})

	it('rejects a full-day end before the start', () => {
		const draft = {
			...emptyAbsenceDraft('member-1', '2026-08-10'),
			range: { from: '2026-08-10', to: '2026-08-09' },
		}
		expect(validateAbsenceDraft(draft).length).toBeGreaterThan(0)
	})

	it('rejects an invalid time when it is not a full day', () => {
		const draft = {
			...emptyAbsenceDraft('member-1', '2026-08-10'),
			allDay: false,
			startTime: 'nope',
			endTime: '18:00',
		}
		expect(validateAbsenceDraft(draft).length).toBeGreaterThan(0)
	})

	it('rejects an end before the start on a time slot', () => {
		const draft = {
			...emptyAbsenceDraft('member-1', '2026-08-10'),
			allDay: false,
			startTime: '18:00',
			endTime: '08:00',
		}
		expect(validateAbsenceDraft(draft).length).toBeGreaterThan(0)
	})
})

describe('draftToCreateAbsenceRequest — full day', () => {
	it('builds starts_at/ends_at as the local window [00:00, 24:00) over the day range', () => {
		const draft = {
			...emptyAbsenceDraft('member-1', '2026-08-10'),
			range: { from: '2026-08-10', to: '2026-08-12' },
			note: 'Congés été',
		}
		const request = draftToCreateAbsenceRequest(draft, TZ)

		expect(request).toEqual({
			member_id: 'member-1',
			kind: 'LEAVE',
			all_day: true,
			starts_at: new Date('2026-08-10T00:00:00+02:00').toISOString(),
			ends_at: new Date('2026-08-13T00:00:00+02:00').toISOString(),
			note: 'Congés été',
		})
	})

	it('accepts a range reduced to a single day (from === to)', () => {
		const draft = {
			...emptyAbsenceDraft('member-1', '2026-08-10'),
			range: { from: '2026-08-10', to: '2026-08-10' },
		}
		const request = draftToCreateAbsenceRequest(draft, TZ)

		expect(request).toMatchObject({
			starts_at: new Date('2026-08-10T00:00:00+02:00').toISOString(),
			ends_at: new Date('2026-08-11T00:00:00+02:00').toISOString(),
		})
	})

	it('returns null when validation fails rather than forging a broken payload', () => {
		const draft = { ...emptyAbsenceDraft('', '2026-08-10') }
		expect(draftToCreateAbsenceRequest(draft, TZ)).toBeNull()
	})
})

describe('draftToCreateAbsenceRequest — time slot', () => {
	it('combines local date and time into ISO instants', () => {
		const draft = {
			...emptyAbsenceDraft('member-1', '2026-08-10'),
			allDay: false,
			kind: 'SICK' as const,
			startTime: '09:00',
			endTime: '17:00',
			note: '',
		}
		const request = draftToCreateAbsenceRequest(draft, TZ)

		expect(request).toEqual({
			member_id: 'member-1',
			kind: 'SICK',
			all_day: false,
			starts_at: new Date('2026-08-10T09:00:00+02:00').toISOString(),
			ends_at: new Date('2026-08-10T17:00:00+02:00').toISOString(),
			note: null,
		})
	})
})

describe('draftToUpdateAbsenceRequest', () => {
	it('carries no member_id — the API rejects it when editing', () => {
		const draft = emptyAbsenceDraft('member-1', '2026-08-10')
		const request = draftToUpdateAbsenceRequest(draft, TZ)
		expect(request).not.toHaveProperty('member_id')
		expect(request).toMatchObject({ kind: 'LEAVE', all_day: true })
	})

	it('returns null when validation fails', () => {
		const draft = {
			...emptyAbsenceDraft('member-1', '2026-08-10'),
			range: { from: '2026-08-10', to: '2026-08-01' },
		}
		expect(draftToUpdateAbsenceRequest(draft, TZ)).toBeNull()
	})
})

describe('absenceToDraft — aller-retour', () => {
	it('rebuilds a full-day draft consistent with the payload that produced it', () => {
		const created = draftToCreateAbsenceRequest(
			{
				...emptyAbsenceDraft('member-1', '2026-08-10'),
				range: { from: '2026-08-10', to: '2026-08-12' },
				kind: 'UNAVAILABLE',
				note: 'Formation',
			},
			TZ,
		)
		if (!created) throw new Error('expected a request')

		const draft = absenceToDraft(
			{
				member_id: 'member-1',
				absence_kind: created.kind,
				all_day: created.all_day ?? true,
				starts_at: created.starts_at,
				ends_at: created.ends_at,
				note: created.note ?? null,
			},
			TZ,
		)

		expect(draft).toMatchObject({
			memberId: 'member-1',
			kind: 'UNAVAILABLE',
			allDay: true,
			range: { from: '2026-08-10', to: '2026-08-12' },
			note: 'Formation',
		})
	})

	it('rebuilds a single-day full-day draft (from === to)', () => {
		const created = draftToCreateAbsenceRequest(
			{
				...emptyAbsenceDraft('member-1', '2026-08-10'),
				range: { from: '2026-08-10', to: '2026-08-10' },
			},
			TZ,
		)
		if (!created) throw new Error('expected a request')

		const draft = absenceToDraft(
			{
				member_id: 'member-1',
				absence_kind: created.kind,
				all_day: created.all_day ?? true,
				starts_at: created.starts_at,
				ends_at: created.ends_at,
				note: created.note ?? null,
			},
			TZ,
		)

		expect(draft.range).toEqual({ from: '2026-08-10', to: '2026-08-10' })
	})

	it('rebuilds a time-slot draft consistent with the payload that produced it', () => {
		const created = draftToCreateAbsenceRequest(
			{
				...emptyAbsenceDraft('member-1', '2026-08-10'),
				allDay: false,
				startTime: '09:00',
				endTime: '17:00',
			},
			TZ,
		)
		if (!created) throw new Error('expected a request')

		const draft = absenceToDraft(
			{
				member_id: 'member-1',
				absence_kind: created.kind,
				all_day: created.all_day ?? false,
				starts_at: created.starts_at,
				ends_at: created.ends_at,
				note: created.note ?? null,
			},
			TZ,
		)

		expect(draft).toMatchObject({
			allDay: false,
			range: { from: '2026-08-10', to: '2026-08-10' },
			startTime: '09:00',
			endTime: '17:00',
		})
	})
})

describe('calendarSelectionToRange', () => {
	it('returns null when nothing is selected', () => {
		expect(calendarSelectionToRange(undefined)).toBeNull()
	})

	it('folds to onto from after the first click (selection in progress)', () => {
		const from = new Date('2026-08-10T00:00:00Z')
		expect(calendarSelectionToRange({ from })).toEqual({
			from: '2026-08-10',
			to: '2026-08-10',
		})
	})

	it('carries both bounds once the range is complete', () => {
		const from = new Date('2026-08-10T00:00:00Z')
		const to = new Date('2026-08-12T00:00:00Z')
		expect(calendarSelectionToRange({ from, to })).toEqual({
			from: '2026-08-10',
			to: '2026-08-12',
		})
	})
})

describe('rangeToCalendarSelection', () => {
	it('is the inverse of calendarSelectionToRange on a complete range', () => {
		const range = { from: '2026-08-10', to: '2026-08-12' }
		const selection = rangeToCalendarSelection(range)
		expect(calendarSelectionToRange(selection)).toEqual(range)
	})
})
