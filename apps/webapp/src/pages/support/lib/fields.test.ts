import { describe, expect, it } from 'vitest'
import {
	displayFieldValue,
	ticketNumberLabel,
} from '#/pages/support/lib/fields'

describe('displayFieldValue', () => {
	it('is empty when the custom fields are not an object', () => {
		expect(displayFieldValue(null, 'ville')).toBe('')
		expect(displayFieldValue(undefined, 'ville')).toBe('')
		expect(displayFieldValue('ville', 'ville')).toBe('')
		expect(displayFieldValue(12, 'ville')).toBe('')
		expect(displayFieldValue(true, 'ville')).toBe('')
		expect(displayFieldValue(['Lyon'], 'length')).toBe('')
	})

	it('is empty when the key is missing', () => {
		expect(displayFieldValue({ pays: 'France' }, 'ville')).toBe('')
		expect(displayFieldValue({}, 'ville')).toBe('')
	})

	it('is empty when the value is null', () => {
		expect(displayFieldValue({ ville: null }, 'ville')).toBe('')
	})

	it('returns a string as it is', () => {
		expect(displayFieldValue({ ville: 'Lyon' }, 'ville')).toBe('Lyon')
		expect(displayFieldValue({ ville: '' }, 'ville')).toBe('')
	})

	it('renders numbers and booleans', () => {
		expect(displayFieldValue({ etage: 3 }, 'etage')).toBe('3')
		expect(displayFieldValue({ surface: 12.5 }, 'surface')).toBe('12.5')
		expect(displayFieldValue({ zero: 0 }, 'zero')).toBe('0')
		expect(displayFieldValue({ urgent: true }, 'urgent')).toBe('true')
		expect(displayFieldValue({ urgent: false }, 'urgent')).toBe('false')
	})

	it('renders objects and arrays as JSON', () => {
		expect(displayFieldValue({ adresse: { rue: 'Rue A' } }, 'adresse')).toBe(
			'{"rue":"Rue A"}',
		)
		expect(displayFieldValue({ tags: ['a', 'b'] }, 'tags')).toBe('["a","b"]')
	})
})

describe('ticketNumberLabel', () => {
	it('prefixes the number with a hash', () => {
		expect(ticketNumberLabel(42)).toBe('#42')
		expect(ticketNumberLabel(1)).toBe('#1')
	})
})
