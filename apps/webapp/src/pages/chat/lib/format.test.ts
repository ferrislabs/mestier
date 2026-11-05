import { describe, expect, it } from 'vitest'
import { formatFileSize, getInitials, userHue } from '#/pages/chat/lib/format'

describe('formatFileSize', () => {
	it('writes octets, Ko and Mo with a decimal comma', () => {
		expect(formatFileSize(512)).toBe('512 o')
		expect(formatFileSize(1536)).toBe('1,5 Ko')
		expect(formatFileSize(3 * 1024 * 1024)).toBe('3 Mo')
	})
})

describe('getInitials', () => {
	it('keeps two initials', () => {
		expect(getInitials('Karim Benali Junior')).toBe('KB')
	})
})

describe('userHue', () => {
	it('is stable and within the hue circle', () => {
		expect(userHue('user-1')).toBe(userHue('user-1'))
		expect(userHue('user-1')).toBeLessThan(360)
		expect(userHue(null)).toBe(0)
	})
})
