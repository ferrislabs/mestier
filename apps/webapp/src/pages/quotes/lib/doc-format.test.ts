import { describe, expect, it } from 'vitest'
import {
	dueLabel,
	formatDocumentDate,
	formatSiret,
	initialsOf,
	nameHue,
	normalizeText,
	plural,
	postalAddressLines,
} from './doc-format'

describe('formatDocumentDate', () => {
	it('prints a dash when there is no date', () => {
		expect(formatDocumentDate(null)).toBe('—')
		expect(formatDocumentDate(undefined)).toBe('—')
		expect(formatDocumentDate('not a date')).toBe('—')
	})

	it('reads a date-only value in local time so it never shifts a day', () => {
		expect(formatDocumentDate('2026-03-05')).toContain('5')
		expect(formatDocumentDate('2026-03-05')).toContain('2026')
	})
})

describe('plural', () => {
	it('keeps the singular for zero and one', () => {
		expect(plural(0, 'jour', 'jours')).toBe('jour')
		expect(plural(1, 'jour', 'jours')).toBe('jour')
		expect(plural(2, 'jour', 'jours')).toBe('jours')
	})
})

describe('dueLabel', () => {
	const now = new Date(2026, 5, 10, 15, 0)

	it('says there is no due date', () => {
		expect(dueLabel(null, now)).toBe('Sans échéance')
	})

	it('counts the days before and after the due date', () => {
		expect(dueLabel('2026-06-10', now)).toBe('Échéance aujourd’hui')
		expect(dueLabel('2026-06-09', now)).toBe('Échue depuis 1 jour')
		expect(dueLabel('2026-06-05', now)).toBe('Échue depuis 5 jours')
		expect(dueLabel('2026-06-12', now)).toBe('À régler dans 2 jours')
	})
})

describe('formatSiret', () => {
	it('groups a fourteen digit identifier', () => {
		expect(formatSiret('12345678901234')).toBe('123 456 789 01234')
	})

	it('leaves anything else untouched', () => {
		expect(formatSiret('FR123')).toBe('FR123')
	})
})

describe('postalAddressLines', () => {
	it('drops empty parts and hides France', () => {
		expect(
			postalAddressLines({
				street: '5 rue des Forges',
				postalCode: '69001',
				city: 'Lyon',
				country: 'France',
			}),
		).toEqual(['5 rue des Forges', '69001 Lyon'])
	})

	it('keeps a foreign country', () => {
		expect(
			postalAddressLines({
				street: 'Rue 1',
				city: 'Gand',
				country: 'Belgique',
			}),
		).toEqual(['Rue 1', 'Gand', 'Belgique'])
	})
})

describe('names', () => {
	it('derives initials without the civility prefix', () => {
		expect(initialsOf('Mme Claire Martin')).toBe('CM')
		expect(initialsOf('Menuiserie Dupont')).toBe('MD')
	})

	it('derives a stable hue', () => {
		expect(nameHue('Dupont')).toBe(nameHue('Dupont'))
		expect(nameHue('Dupont')).toBeLessThan(360)
	})

	it('strips accents for searching', () => {
		expect(normalizeText(' Éléonore ')).toBe('eleonore')
	})
})
