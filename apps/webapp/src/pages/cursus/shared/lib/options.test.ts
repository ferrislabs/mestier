import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	apprenticeOptions,
	companyTutorOptions,
} from '#/pages/cursus/shared/lib/options'

function detail(
	overrides: Partial<Schemas.ParticipantDetailResponse>,
): Schemas.ParticipantDetailResponse {
	return {
		id: 'p-1',
		organization_id: 'org-1',
		email: 'a@b.fr',
		first_name: 'Lea',
		last_name: 'Martin',
		status: 'active',
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
		company_tutors: [],
		...overrides,
	}
}

describe('apprenticeOptions', () => {
	it('keys each option on the apprentice record id, not the participant id', () => {
		const options = apprenticeOptions([
			detail({
				apprentice: {
					id: 'a-1',
					participant_id: 'p-1',
					created_at: '',
					updated_at: '',
				},
			}),
		])
		expect(options).toEqual([{ value: 'a-1', label: 'Lea Martin' }])
	})

	it('skips participants without an apprentice record and inactive ones', () => {
		const options = apprenticeOptions([
			detail({}),
			detail({
				status: 'inactive',
				apprentice: {
					id: 'a-2',
					participant_id: 'p-1',
					created_at: '',
					updated_at: '',
				},
			}),
		])
		expect(options).toEqual([])
	})
})

describe('companyTutorOptions', () => {
	it('keeps only tutor records of the given host company', () => {
		const options = companyTutorOptions(
			[
				detail({
					company_tutors: [
						{
							id: 't-1',
							host_company_id: 'h-1',
							host_company_name: 'A',
							participant_id: 'p-1',
							created_at: '',
							updated_at: '',
						},
						{
							id: 't-2',
							host_company_id: 'h-2',
							host_company_name: 'B',
							participant_id: 'p-1',
							created_at: '',
							updated_at: '',
						},
					],
				}),
			],
			'h-1',
		)
		expect(options).toEqual([{ value: 't-1', label: 'Lea Martin' }])
	})
})
