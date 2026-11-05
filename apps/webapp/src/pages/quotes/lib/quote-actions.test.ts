import { describe, expect, it } from 'vitest'
import { availableQuoteActions } from './quote-actions'

describe('availableQuoteActions', () => {
	it('lets a draft be sent first', () => {
		expect(availableQuoteActions('DRAFT')[0]).toBe('send')
	})

	it('lets a sent quote be answered', () => {
		const actions = availableQuoteActions('SENT')

		expect(actions).toContain('accept')
		expect(actions).toContain('decline')
		expect(actions).not.toContain('send')
	})

	it('turns an accepted quote into a project', () => {
		expect(availableQuoteActions('ACCEPTED')[0]).toBe('project')
	})

	it('can always be duplicated and deleted', () => {
		for (const status of [
			'DRAFT',
			'SENT',
			'ACCEPTED',
			'DECLINED',
			'CANCELLED',
		] as const) {
			expect(availableQuoteActions(status)).toContain('duplicate')
			expect(availableQuoteActions(status)).toContain('delete')
		}
	})
})
