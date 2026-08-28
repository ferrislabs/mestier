import { describe, expect, it } from 'vitest'
import { isForbiddenError } from './api-error'

describe('isForbiddenError', () => {
	it('recognizes the shape thrown by the fetcher on a 403', () => {
		const error = Object.assign(new Error('Forbidden'), { status: 403 })
		expect(isForbiddenError(error)).toBe(true)
	})

	it('rejects other status codes', () => {
		const error = Object.assign(new Error('Not found'), { status: 404 })
		expect(isForbiddenError(error)).toBe(false)
	})

	it('rejects a plain error with no status', () => {
		expect(isForbiddenError(new Error('network down'))).toBe(false)
	})

	it('rejects null and non-object values', () => {
		expect(isForbiddenError(null)).toBe(false)
		expect(isForbiddenError(undefined)).toBe(false)
		expect(isForbiddenError('403')).toBe(false)
	})
})
