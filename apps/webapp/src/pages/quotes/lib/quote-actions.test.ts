import { describe, expect, it } from 'vitest'
import { availableQuoteActions } from './quote-actions'
import type { QuoteDisplayStatus } from './quote-status'

const ALL: QuoteDisplayStatus[] = [
	'draft',
	'sent',
	'accepted',
	'declined',
	'cancelled',
	'superseded',
	'expired',
]

describe('availableQuoteActions', () => {
	it('lets a draft be sent first', () => {
		expect(availableQuoteActions('draft')[0]).toBe('send')
	})

	it('lets a sent quote be answered or revised', () => {
		const actions = availableQuoteActions('sent')

		expect(actions).toContain('accept')
		expect(actions).toContain('decline')
		expect(actions).toContain('revise')
		expect(actions).not.toContain('send')
	})

	it('refuses an answer once the quote has expired, but allows a revision', () => {
		const actions = availableQuoteActions('expired')

		expect(actions).not.toContain('accept')
		expect(actions).not.toContain('decline')
		expect(actions).toContain('revise')
	})

	it('turns an accepted quote into a project, then invoices it', () => {
		const actions = availableQuoteActions('accepted')

		expect(actions[0]).toBe('project')
		expect(actions).toContain('invoice')
	})

	it('offers a revision of a declined quote', () => {
		expect(availableQuoteActions('declined')).toContain('revise')
	})

	it('does not revise a replaced quote again', () => {
		expect(availableQuoteActions('superseded')).not.toContain('revise')
	})

	it('can always be duplicated', () => {
		for (const status of ALL) {
			expect(availableQuoteActions(status)).toContain('duplicate')
		}
	})

	it('only deletes a draft, a sent quote is locked', () => {
		for (const status of ALL.filter((item) => item !== 'draft')) {
			expect(availableQuoteActions(status)).not.toContain('delete')
		}
		expect(availableQuoteActions('draft')).toContain('delete')
	})

	it('only invoices an accepted quote', () => {
		for (const status of ALL.filter((item) => item !== 'accepted')) {
			expect(availableQuoteActions(status)).not.toContain('invoice')
		}
	})
})
