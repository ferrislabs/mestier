import { describe, expect, it } from 'vitest'
import { emptyQuoteLine, type QuoteFormValues } from '#/pages/quotes/types'
import { EMPTY_QUOTE_EXTRAS } from './quote-extras'
import {
	copiedExtras,
	extrasFormFields,
	extrasPatchFromForm,
	lineWithExtras,
} from './quote-form-extras'

function form(overrides: Partial<QuoteFormValues> = {}): QuoteFormValues {
	return {
		title: 'Salle de bain',
		customerId: 'c1',
		customerContextId: 'a1',
		lines: [],
		ownerId: null,
		validUntil: '',
		depositPercent: '',
		notes: '',
		...overrides,
	}
}

describe('extrasPatchFromForm', () => {
	it('turns the typed values into stored ones', () => {
		const patch = extrasPatchFromForm(
			form({
				ownerId: 'm1',
				validUntil: '2026-12-01',
				depositPercent: '30',
				notes: '  Accès par la cour ',
				lines: [
					{
						...emptyQuoteLine(),
						discountPercent: '12,5',
						frequency: 'monthly',
					},
					emptyQuoteLine('l2'),
				],
			}),
		)

		expect(patch).toEqual({
			ownerId: 'm1',
			validUntil: '2026-12-01',
			depositPercent: 30,
			notes: 'Accès par la cour',
			lines: [
				{ discountPercent: 12.5, frequency: 'monthly' },
				{ discountPercent: 0, frequency: 'one_shot' },
			],
		})
	})

	it('stores nothing for an empty date and drops unreadable percents', () => {
		const patch = extrasPatchFromForm(
			form({
				depositPercent: 'abc',
				lines: [{ ...emptyQuoteLine(), discountPercent: '250' }],
			}),
		)

		expect(patch.validUntil).toBeNull()
		expect(patch.depositPercent).toBe(0)
		expect(patch.lines?.[0]?.discountPercent).toBe(0)
	})
})

describe('extrasFormFields', () => {
	it('shows the stored values as typed ones', () => {
		expect(
			extrasFormFields({
				...EMPTY_QUOTE_EXTRAS,
				ownerId: 'm1',
				validUntil: '2026-12-01',
				depositPercent: 30,
				notes: 'n',
			}),
		).toEqual({
			ownerId: 'm1',
			validUntil: '2026-12-01',
			depositPercent: '30',
			notes: 'n',
		})
	})

	it('leaves a zero deposit and a missing date blank', () => {
		expect(extrasFormFields(EMPTY_QUOTE_EXTRAS)).toEqual({
			ownerId: null,
			validUntil: '',
			depositPercent: '',
			notes: '',
		})
	})
})

describe('lineWithExtras', () => {
	it('fills the discount and the frequency of the line at that position', () => {
		const extras = {
			...EMPTY_QUOTE_EXTRAS,
			lines: [{ discountPercent: 10, frequency: 'yearly' as const }],
		}

		expect(lineWithExtras(emptyQuoteLine(), extras, 0)).toMatchObject({
			discountPercent: '10',
			frequency: 'yearly',
		})
	})

	it('keeps a blank discount when there is none', () => {
		expect(
			lineWithExtras(emptyQuoteLine(), EMPTY_QUOTE_EXTRAS, 3),
		).toMatchObject({ discountPercent: '', frequency: 'one_shot' })
	})
})

describe('copiedExtras', () => {
	it('carries the terms over to a copy but neither the answer nor the links', () => {
		const copy = copiedExtras({
			...EMPTY_QUOTE_EXTRAS,
			ownerId: 'm1',
			validUntil: '2026-12-01',
			depositPercent: 30,
			notes: 'n',
			dealId: 'd1',
			signer: 'Mme Durand',
			respondedAt: '2026-11-02',
			supersededBy: { id: 'q9', label: 'x' },
			revisionOf: { id: 'q0', label: 'y' },
			lines: [{ discountPercent: 5, frequency: 'monthly' }],
		})

		expect(copy).toEqual({
			ownerId: 'm1',
			depositPercent: 30,
			notes: 'n',
			dealId: 'd1',
			lines: [{ discountPercent: 5, frequency: 'monthly' }],
		})
	})
})
