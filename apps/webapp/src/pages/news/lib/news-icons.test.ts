import { describe, expect, it } from 'vitest'
import { NEWS_ICON_OPTIONS, newsIconComponent } from './news-icons'

describe('news icons', () => {
	it('maps every backend icon to a component and a label', () => {
		expect(NEWS_ICON_OPTIONS).toHaveLength(16)
		for (const option of NEWS_ICON_OPTIONS) {
			expect(newsIconComponent(option.value)).toBeTruthy()
			expect(option.label.length).toBeGreaterThan(0)
		}
	})

	it('has no duplicate values', () => {
		const values = NEWS_ICON_OPTIONS.map((option) => option.value)
		expect(new Set(values).size).toBe(values.length)
	})

	it('falls back to a default icon for an unknown value', () => {
		expect(newsIconComponent('nope' as never)).toBe(
			newsIconComponent('sparkles'),
		)
	})
})
