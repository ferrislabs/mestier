import { afterEach, describe, expect, it, vi } from 'vitest'
import {
	forgetResumeToken,
	readResumeToken,
	resumeKey,
	saveResumeToken,
} from '#/pages/survey-respond/lib/resume-store'

afterEach(() => {
	vi.restoreAllMocks()
	window.localStorage.clear()
})

describe('resume store', () => {
	it('keeps one token per survey link', () => {
		saveResumeToken('lien-a', 'token-a')
		saveResumeToken('lien-b', 'token-b')
		expect(readResumeToken('lien-a')).toBe('token-a')
		expect(window.localStorage.getItem(resumeKey('lien-b'))).toBe('token-b')
		forgetResumeToken('lien-a')
		expect(readResumeToken('lien-a')).toBeNull()
		expect(readResumeToken('lien-b')).toBe('token-b')
	})

	it('works without storage', () => {
		vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => {
			throw new Error('denied')
		})
		vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
			throw new Error('denied')
		})
		vi.spyOn(Storage.prototype, 'removeItem').mockImplementation(() => {
			throw new Error('denied')
		})
		expect(() => saveResumeToken('lien', 'token')).not.toThrow()
		expect(readResumeToken('lien')).toBeNull()
		expect(() => forgetResumeToken('lien')).not.toThrow()
	})
})
