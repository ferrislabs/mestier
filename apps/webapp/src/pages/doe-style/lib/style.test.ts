import { describe, expect, it } from 'vitest'
import {
	completeStyle,
	DEFAULT_DOE_STYLE,
	isStyleValid,
	sameStyle,
	validateStyle,
} from '#/pages/doe-style/lib/style'

describe('completeStyle', () => {
	it('fills every missing field from the defaults', () => {
		expect(completeStyle({})).toEqual(DEFAULT_DOE_STYLE)
		expect(completeStyle(null)).toEqual(DEFAULT_DOE_STYLE)
	})

	it('keeps the fields that are present, even falsy ones', () => {
		const style = completeStyle({
			primary: '#123456',
			cover_photo: false,
			footer_text: '',
			logo_key: 'logo',
		})
		expect(style.primary).toBe('#123456')
		expect(style.cover_photo).toBe(false)
		expect(style.footer_text).toBe('')
		expect(style.logo_key).toBe('logo')
		expect(style.secondary).toBe(DEFAULT_DOE_STYLE.secondary)
	})
})

describe('validateStyle', () => {
	it('accepts the defaults', () => {
		expect(validateStyle(DEFAULT_DOE_STYLE)).toEqual({})
		expect(isStyleValid(DEFAULT_DOE_STYLE)).toBe(true)
	})

	it('refuses a malformed color', () => {
		const errors = validateStyle({ ...DEFAULT_DOE_STYLE, primary: '#12345' })
		expect(errors.primary).toMatch(/#rrggbb/)
		expect(errors.secondary).toBeUndefined()
	})

	it('refuses a blank or too long label', () => {
		expect(
			validateStyle({ ...DEFAULT_DOE_STYLE, cover_title: '   ' }).cover_title,
		).toBeDefined()
		expect(
			validateStyle({ ...DEFAULT_DOE_STYLE, parties_title: 'a'.repeat(121) })
				.parties_title,
		).toBeDefined()
		expect(
			validateStyle({ ...DEFAULT_DOE_STYLE, parties_title: 'a'.repeat(120) })
				.parties_title,
		).toBeUndefined()
	})

	it('refuses an unknown template variable', () => {
		const errors = validateStyle({
			...DEFAULT_DOE_STYLE,
			header_text: 'DOE {client}',
		})
		expect(errors.header_text).toMatch(/\{client\}/)
	})

	it('accepts the five variables and an empty footer', () => {
		expect(
			isStyleValid({
				...DEFAULT_DOE_STYLE,
				header_text: '{titre} {lieu} {lot} {indice} {date}',
				footer_text: '',
			}),
		).toBe(true)
	})

	it('refuses a template over 200 characters', () => {
		expect(
			validateStyle({ ...DEFAULT_DOE_STYLE, footer_text: 'a'.repeat(201) })
				.footer_text,
		).toBeDefined()
	})
})

describe('sameStyle', () => {
	it('ignores color case and label padding', () => {
		expect(
			sameStyle(DEFAULT_DOE_STYLE, {
				...DEFAULT_DOE_STYLE,
				primary: '#0F3D36',
				cover_title: `${DEFAULT_DOE_STYLE.cover_title} `,
			}),
		).toBe(true)
		expect(
			sameStyle(DEFAULT_DOE_STYLE, {
				...DEFAULT_DOE_STYLE,
				body_font: 'barlow',
			}),
		).toBe(false)
	})
})
