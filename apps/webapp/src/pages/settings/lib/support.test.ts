import { describe, expect, it } from 'vitest'
import {
	MAX_SUPPORT_DISPLAY_FIELD_CHARS,
	MAX_SUPPORT_DISPLAY_FIELDS,
	parseSupportDisplayFields,
	supportDisplayFieldsToText,
} from '#/pages/settings/lib/support'

function keys(count: number) {
	return Array.from({ length: count }, (_, index) => `cle_${index}`)
}

describe('limits', () => {
	it('mirror the API bounds', () => {
		expect(MAX_SUPPORT_DISPLAY_FIELDS).toBe(8)
		expect(MAX_SUPPORT_DISPLAY_FIELD_CHARS).toBe(64)
	})
})

describe('parseSupportDisplayFields', () => {
	it('trims every line and keeps the order', () => {
		expect(
			parseSupportDisplayFields('  numero_contrat \nchantier\t\n marque'),
		).toEqual({ ok: true, fields: ['numero_contrat', 'chantier', 'marque'] })
	})

	it('drops empty and blank lines', () => {
		expect(parseSupportDisplayFields('\n  \nchantier\n\t\n\nmarque\n')).toEqual(
			{
				ok: true,
				fields: ['chantier', 'marque'],
			},
		)
	})

	it('splits on CRLF as well as LF', () => {
		expect(
			parseSupportDisplayFields('chantier\r\nmarque\r\n\r\nmodele'),
		).toEqual({ ok: true, fields: ['chantier', 'marque', 'modele'] })
	})

	it('accepts an empty input as an empty list', () => {
		expect(parseSupportDisplayFields('')).toEqual({ ok: true, fields: [] })
		expect(parseSupportDisplayFields(' \n \r\n')).toEqual({
			ok: true,
			fields: [],
		})
	})

	it('refuses a duplicate and names the key', () => {
		expect(parseSupportDisplayFields('chantier\nmarque\nchantier')).toEqual({
			ok: false,
			error: 'La clé « chantier » est listée deux fois.',
		})
	})

	it('refuses a duplicate that only differs by surrounding spaces', () => {
		expect(parseSupportDisplayFields('chantier\n  chantier  ')).toEqual({
			ok: false,
			error: 'La clé « chantier » est listée deux fois.',
		})
	})

	it('treats keys that differ by case as distinct', () => {
		expect(parseSupportDisplayFields('Chantier\nchantier')).toEqual({
			ok: true,
			fields: ['Chantier', 'chantier'],
		})
	})

	it('accepts eight keys', () => {
		const result = parseSupportDisplayFields(keys(8).join('\n'))

		expect(result).toEqual({ ok: true, fields: keys(8) })
	})

	it('refuses nine keys', () => {
		expect(parseSupportDisplayFields(keys(9).join('\n'))).toEqual({
			ok: false,
			error: 'Huit clés au plus.',
		})
	})

	it('counts nine keys after blank lines are dropped, not before', () => {
		const lines = keys(8).flatMap((key) => [key, ''])

		expect(parseSupportDisplayFields(lines.join('\n'))).toEqual({
			ok: true,
			fields: keys(8),
		})
	})

	it('accepts a 64-character key', () => {
		const key = 'a'.repeat(64)

		expect(parseSupportDisplayFields(key)).toEqual({ ok: true, fields: [key] })
	})

	it('refuses a 65-character key and names it', () => {
		const key = 'a'.repeat(65)

		expect(parseSupportDisplayFields(key)).toEqual({
			ok: false,
			error: `La clé « ${key} » dépasse 64 caractères.`,
		})
	})

	it('counts characters, not bytes, for accented keys', () => {
		const accepted = 'é'.repeat(64)
		const refused = 'é'.repeat(65)

		expect(parseSupportDisplayFields(accepted)).toEqual({
			ok: true,
			fields: [accepted],
		})
		expect(parseSupportDisplayFields(refused)).toEqual({
			ok: false,
			error: `La clé « ${refused} » dépasse 64 caractères.`,
		})
	})
})

describe('supportDisplayFieldsToText', () => {
	it('joins the keys one per line', () => {
		expect(supportDisplayFieldsToText(['chantier', 'marque'])).toBe(
			'chantier\nmarque',
		)
	})

	it('renders an empty list as an empty text', () => {
		expect(supportDisplayFieldsToText([])).toBe('')
	})

	it('round-trips through the parser', () => {
		const fields = ['numero_contrat', 'chantier', 'marque']

		expect(
			parseSupportDisplayFields(supportDisplayFieldsToText(fields)),
		).toEqual({ ok: true, fields })
	})
})
