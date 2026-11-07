import { beforeEach, describe, expect, it } from 'vitest'
import { EMPTY_QUOTE_EXTRAS } from '#/pages/quotes/lib/quote-extras'
import {
	readQuoteExtras,
	resetQuoteExtrasCache,
	saveQuoteExtras,
} from './quote-extras'

beforeEach(() => {
	window.localStorage.clear()
	resetQuoteExtrasCache()
})

describe('quote extras store', () => {
	it('reads the empty extras for a quote nobody wrote to', () => {
		expect(readQuoteExtras('q1')).toEqual(EMPTY_QUOTE_EXTRAS)
	})

	it('keeps what was saved, per quote', () => {
		saveQuoteExtras('q1', { notes: 'a', depositPercent: 30 })
		saveQuoteExtras('q2', { notes: 'b' })

		expect(readQuoteExtras('q1')).toMatchObject({
			notes: 'a',
			depositPercent: 30,
		})
		expect(readQuoteExtras('q2').notes).toBe('b')
	})

	it('merges successive saves', () => {
		saveQuoteExtras('q1', { notes: 'a' })
		saveQuoteExtras('q1', { ownerId: 'm1' })

		expect(readQuoteExtras('q1')).toMatchObject({ notes: 'a', ownerId: 'm1' })
	})

	it('survives a reload', () => {
		saveQuoteExtras('q1', { signer: 'Mme Durand' })
		resetQuoteExtrasCache()

		expect(readQuoteExtras('q1').signer).toBe('Mme Durand')
	})

	it('hands back the same object until the next write', () => {
		expect(readQuoteExtras('q1')).toBe(readQuoteExtras('q1'))
	})

	it('survives a corrupted entry', () => {
		window.localStorage.setItem('mestier.mock.quote-extras.q1', '{oops')
		resetQuoteExtrasCache()

		expect(readQuoteExtras('q1')).toEqual(EMPTY_QUOTE_EXTRAS)
	})
})
