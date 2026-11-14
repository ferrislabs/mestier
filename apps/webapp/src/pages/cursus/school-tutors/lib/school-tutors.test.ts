import { describe, expect, it } from 'vitest'
import {
	apiErrorCode,
	availableMembers,
	schoolTutorOptions,
	toCreateSchoolTutorRequest,
} from '#/pages/cursus/school-tutors/lib/school-tutors'

describe('school tutors lib', () => {
	it('offers only members not yet declared', () => {
		expect(
			availableMembers(
				[
					{ id: 'm-1', display_name: 'Alix' },
					{ id: 'm-2', display_name: 'Jo' },
				],
				[{ member_id: 'm-1' }],
			),
		).toEqual([{ value: 'm-2', label: 'Jo' }])
	})

	it('keeps the current tutor among the active ones', () => {
		const active = [{ member_id: 'm-1', member_name: 'Alix' }]
		expect(schoolTutorOptions(active, { id: 'm-9', name: 'Old' })).toEqual([
			{ value: 'm-1', label: 'Alix' },
			{ value: 'm-9', label: 'Old' },
		])
		expect(schoolTutorOptions(active, { id: 'm-1', name: 'Alix' })).toEqual([
			{ value: 'm-1', label: 'Alix' },
		])
		expect(schoolTutorOptions(active, null)).toEqual([
			{ value: 'm-1', label: 'Alix' },
		])
	})

	it('builds the creation body', () => {
		expect(
			toCreateSchoolTutorRequest({ memberId: 'm-1', jobTitle: '  ' }),
		).toEqual({ member_id: 'm-1', job_title: null })
		expect(
			toCreateSchoolTutorRequest({ memberId: 'm-1', jobTitle: ' Prof ' }),
		).toEqual({ member_id: 'm-1', job_title: 'Prof' })
	})

	it('reads the api error code', () => {
		expect(apiErrorCode({ status: 422, data: { code: 'E_X' } })).toBe('E_X')
		expect(apiErrorCode(new Error('x'))).toBeNull()
		expect(apiErrorCode(null)).toBeNull()
	})
})
