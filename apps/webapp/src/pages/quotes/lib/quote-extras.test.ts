import { describe, expect, it } from 'vitest'
import {
	acceptedPatch,
	declinedPatch,
	EMPTY_QUOTE_EXTRAS,
	lineExtrasAt,
	mergeQuoteExtras,
	parseQuoteExtras,
	revisionPatches,
} from './quote-extras'

describe('mergeQuoteExtras', () => {
	it('starts from the empty extras and applies the patch', () => {
		expect(mergeQuoteExtras(undefined, { notes: 'Accès par la cour' })).toEqual(
			{ ...EMPTY_QUOTE_EXTRAS, notes: 'Accès par la cour' },
		)
	})

	it('keeps what the patch does not mention', () => {
		const base = { ...EMPTY_QUOTE_EXTRAS, ownerId: 'm1', depositPercent: 30 }

		expect(mergeQuoteExtras(base, { notes: 'x' })).toMatchObject({
			ownerId: 'm1',
			depositPercent: 30,
			notes: 'x',
		})
	})
})

describe('answers', () => {
	it('records who signed and when', () => {
		expect(acceptedPatch('  Mme Durand ', '2026-11-07')).toEqual({
			signer: 'Mme Durand',
			respondedAt: '2026-11-07',
			declineReason: '',
		})
	})

	it('records the reason of a refusal', () => {
		expect(declinedPatch(' trop cher ', '2026-11-07')).toEqual({
			declineReason: 'trop cher',
			respondedAt: '2026-11-07',
			signer: '',
		})
	})
})

describe('revisionPatches', () => {
	const input = {
		previous: { id: 'q1', label: 'DEV-001' },
		next: { id: 'q2', label: 'Brouillon' },
		previousExtras: EMPTY_QUOTE_EXTRAS,
		at: '2026-11-07T10:00:00Z',
	}

	it('links the new draft to its predecessor', () => {
		expect(revisionPatches(input).next).toEqual({
			revisionOf: { id: 'q1', label: 'DEV-001' },
		})
	})

	it('supersedes the old quote and notes it in its history', () => {
		const { previous } = revisionPatches(input)

		expect(previous.supersededBy).toEqual({ id: 'q2', label: 'Brouillon' })
		expect(previous.events).toEqual([
			{
				id: 'revised-q2',
				kind: 'revised',
				detail: 'Remplacé par Brouillon',
				at: '2026-11-07T10:00:00Z',
			},
		])
	})

	it('keeps the events already recorded', () => {
		const earlier = {
			id: 'e1',
			kind: 'reminder' as const,
			detail: 'Relancé',
			at: '2026-11-01T10:00:00Z',
		}
		const { previous } = revisionPatches({
			...input,
			previousExtras: { ...EMPTY_QUOTE_EXTRAS, events: [earlier] },
		})

		expect(previous.events?.[0]).toEqual(earlier)
		expect(previous.events).toHaveLength(2)
	})
})

describe('lineExtrasAt', () => {
	it('defaults to no discount and one-shot billing', () => {
		expect(lineExtrasAt(EMPTY_QUOTE_EXTRAS, 4)).toEqual({
			discountPercent: 0,
			frequency: 'one_shot',
		})
	})

	it('reads the extras of the line at that position', () => {
		const extras = {
			...EMPTY_QUOTE_EXTRAS,
			lines: [{ discountPercent: 10, frequency: 'monthly' as const }],
		}

		expect(lineExtrasAt(extras, 0)).toEqual({
			discountPercent: 10,
			frequency: 'monthly',
		})
	})
})

describe('parseQuoteExtras', () => {
	it('falls back to the empty extras on anything unreadable', () => {
		expect(parseQuoteExtras(null)).toEqual(EMPTY_QUOTE_EXTRAS)
		expect(parseQuoteExtras('{oops')).toEqual(EMPTY_QUOTE_EXTRAS)
		expect(parseQuoteExtras('42')).toEqual(EMPTY_QUOTE_EXTRAS)
	})

	it('keeps the valid fields and drops the others', () => {
		const parsed = parseQuoteExtras(
			JSON.stringify({
				ownerId: 'm1',
				depositPercent: 'abc',
				validUntil: '2026-12-01',
				lines: [{ discountPercent: 5, frequency: 'weekly' }],
			}),
		)

		expect(parsed.ownerId).toBe('m1')
		expect(parsed.depositPercent).toBe(0)
		expect(parsed.validUntil).toBe('2026-12-01')
		expect(parsed.lines).toEqual([
			{ discountPercent: 5, frequency: 'one_shot' },
		])
	})
})
