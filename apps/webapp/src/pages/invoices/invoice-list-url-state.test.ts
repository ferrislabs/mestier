import { afterEach, describe, expect, it } from 'vitest'
import {
	clearInvoiceClientParam,
	readInvoiceClientParam,
} from './invoice-list-url-state'

afterEach(() => {
	window.history.replaceState(null, '', '/')
})

describe('invoice client filter', () => {
	it('reads the customer the list is narrowed to', () => {
		window.history.replaceState(null, '', '/crm/factures?client=c1&q=abc')

		expect(readInvoiceClientParam()).toBe('c1')
	})

	it('is null without a client param', () => {
		window.history.replaceState(null, '', '/crm/factures?q=abc')

		expect(readInvoiceClientParam()).toBeNull()
	})

	it('removes only the client param when cleared', () => {
		window.history.replaceState(null, '', '/crm/factures?client=c1&q=abc')

		clearInvoiceClientParam()

		expect(window.location.search).toBe('?q=abc')
	})
})
