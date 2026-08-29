import { describe, expect, it } from 'vitest'
import { isForbiddenError, mutationErrorMessage } from './api-error'

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

describe('mutationErrorMessage', () => {
	it('replaces a 403 with a permission-specific message rather than the raw "Forbidden"', () => {
		const error = Object.assign(new Error('Forbidden'), { status: 403 })
		expect(mutationErrorMessage(error)).toBe(
			"Vous n'avez plus la permission nécessaire pour cette action.",
		)
	})

	it('keeps the original message for any other error', () => {
		expect(mutationErrorMessage(new Error('network down'))).toBe('network down')
	})

	it('returns null when there is no error', () => {
		expect(mutationErrorMessage(null)).toBeNull()
		expect(mutationErrorMessage(undefined)).toBeNull()
	})

	it('falls back to a generic message for a non-Error throw', () => {
		expect(mutationErrorMessage('boom')).toBe('Une erreur est survenue.')
	})
})
