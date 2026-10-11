import { describe, expect, it } from 'vitest'
import { DEFAULT_DOE_STYLE } from '#/pages/doe-style/lib/style'
import {
	activeThemeId,
	applyTheme,
	DOE_THEMES,
} from '#/pages/doe-style/lib/themes'

const theme = (id: string) => {
	const found = DOE_THEMES.find((entry) => entry.id === id)
	if (!found) throw new Error(id)
	return found
}

describe('themes', () => {
	it('offers five presets', () => {
		expect(DOE_THEMES.map((entry) => entry.name)).toEqual([
			'Sobre',
			'Nature',
			'Terre',
			'Corporate',
			'Élégant',
		])
	})

	it('applies the look and keeps logo, templates and labels', () => {
		const current = {
			...DEFAULT_DOE_STYLE,
			logo_key: 'logo-1',
			header_logo: true,
			header_text: '{titre} – {lot}',
			footer_text: '',
			cover_title: 'MON DOE',
		}
		const next = applyTheme(current, theme('elegant'))
		expect(next.primary).toBe('#2b2b2b')
		expect(next.on_primary).toBe('#f5efe1')
		expect(next.heading_font).toBe('source_serif')
		expect(next.logo_key).toBe('logo-1')
		expect(next.header_logo).toBe(true)
		expect(next.header_text).toBe('{titre} – {lot}')
		expect(next.footer_text).toBe('')
		expect(next.cover_title).toBe('MON DOE')
	})

	it('detects the active theme, and none once a value is tweaked', () => {
		expect(activeThemeId(DEFAULT_DOE_STYLE)).toBe('sobre')
		const nature = applyTheme(DEFAULT_DOE_STYLE, theme('nature'))
		expect(activeThemeId(nature)).toBe('nature')
		expect(activeThemeId({ ...nature, primary: '#2F5D3A' })).toBe('nature')
		expect(activeThemeId({ ...nature, primary: '#2f5d3b' })).toBeNull()
		expect(activeThemeId({ ...nature, cover_photo: false })).toBeNull()
	})

	it('keeps the default style on the first theme', () => {
		expect(applyTheme(DEFAULT_DOE_STYLE, theme('sobre'))).toEqual(
			DEFAULT_DOE_STYLE,
		)
	})
})
