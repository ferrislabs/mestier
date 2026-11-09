import { describe, expect, it } from 'vitest'
import {
	addTutor,
	removeTutor,
	setMainTutor,
	toTutorsRequest,
	tutorsToEntries,
} from '#/pages/cursus/triads/lib/contract-tutors'

describe('contract tutor list helpers', () => {
	it('makes the first added tutor main and ignores duplicates', () => {
		const one = addTutor([], 't-1')
		expect(one).toEqual([{ companyTutorId: 't-1', isMain: true }])
		const two = addTutor(one, 't-2')
		expect(two.map((t) => t.isMain)).toEqual([true, false])
		expect(addTutor(two, 't-1')).toBe(two)
	})

	it('moves the main flag', () => {
		const list = setMainTutor(addTutor(addTutor([], 't-1'), 't-2'), 't-2')
		expect(list.map((t) => t.isMain)).toEqual([false, true])
	})

	it('promotes the first remaining tutor when the main is removed', () => {
		const list = removeTutor(
			setMainTutor(
				addTutor(addTutor(addTutor([], 't-1'), 't-2'), 't-3'),
				't-2',
			),
			't-2',
		)
		expect(list.map((t) => [t.companyTutorId, t.isMain])).toEqual([
			['t-1', true],
			['t-3', false],
		])
		expect(removeTutor(addTutor([], 't-1'), 't-1')).toEqual([])
	})

	it('drops archived tutors when loading and builds the request', () => {
		const entries = tutorsToEntries([
			{
				company_tutor_id: 't-1',
				participant_id: 'p-1',
				name: 'A',
				is_main: true,
				archived_at: '2026-02-01T00:00:00Z',
			},
			{
				company_tutor_id: 't-2',
				participant_id: 'p-2',
				name: 'B',
				is_main: false,
			},
		])
		expect(toTutorsRequest(entries)).toEqual({
			tutors: [{ company_tutor_id: 't-2', is_main: true }],
		})
	})
})
