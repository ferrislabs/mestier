import { describe, expect, it } from 'vitest'
import {
	contractFormErrors,
	EMPTY_CONTRACT_FORM,
	toContractRequest,
} from '#/pages/cursus/triads/lib/contract-form'

describe('contract form', () => {
	it('requires company and dates', () => {
		expect(Object.keys(contractFormErrors(EMPTY_CONTRACT_FORM)).sort()).toEqual(
			['endDate', 'hostCompanyId', 'startDate'],
		)
	})

	it('rejects an end date before the start date', () => {
		const errors = contractFormErrors({
			...EMPTY_CONTRACT_FORM,
			hostCompanyId: 'h-1',
			startDate: '2026-09-01',
			endDate: '2026-08-01',
		})
		expect(errors.endDate).toBeDefined()
	})

	it('sends a blank number as null', () => {
		expect(
			toContractRequest({
				hostCompanyId: 'h-1',
				kind: 'PROFESSIONALIZATION',
				number: ' ',
				comment: '  ',
				startDate: '2026-09-01',
				endDate: '2027-08-31',
			}),
		).toEqual({
			host_company_id: 'h-1',
			kind: 'PROFESSIONALIZATION',
			number: null,
			comment: null,
			start_date: '2026-09-01',
			end_date: '2027-08-31',
		})
	})

	it('keeps the markdown comment as typed', () => {
		expect(
			toContractRequest({
				...EMPTY_CONTRACT_FORM,
				hostCompanyId: 'h-1',
				comment: '**Important**',
				startDate: '2026-09-01',
				endDate: '2027-08-31',
			}).comment,
		).toBe('**Important**')
	})
})
