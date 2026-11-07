import { describe, expect, it } from 'vitest'
import { emptyQuoteLine } from '#/pages/quotes/types'
import {
	addDays,
	canChangeClient,
	DEFAULT_VALIDITY_DAYS,
	defaultValidUntil,
	NO_OWNER,
	ownerFromSelect,
	ownerToSelect,
	parsePercent,
	validityNote,
} from './quote-draft'

const TODAY = '2026-11-07'

describe('addDays', () => {
	it('crosses month and year ends', () => {
		expect(addDays('2026-11-07', 30)).toBe('2026-12-07')
		expect(addDays('2026-12-20', 15)).toBe('2027-01-04')
		expect(addDays('2026-11-07', -90)).toBe('2026-08-09')
	})
})

describe('defaultValidUntil', () => {
	it('is the default validity after today', () => {
		expect(defaultValidUntil(TODAY)).toBe(addDays(TODAY, DEFAULT_VALIDITY_DAYS))
	})
})

describe('parsePercent', () => {
	it('reads a decimal percent with a comma or a dot', () => {
		expect(parsePercent('30')).toBe(30)
		expect(parsePercent('12,5')).toBe(12.5)
		expect(parsePercent('0')).toBe(0)
	})

	it('refuses anything outside 0 to 100', () => {
		expect(parsePercent('-1')).toBeNull()
		expect(parsePercent('100,5')).toBeNull()
		expect(parsePercent('abc')).toBeNull()
		expect(parsePercent('')).toBeNull()
	})
})

describe('owner select', () => {
	it('stands for no owner with a sentinel', () => {
		expect(ownerToSelect(null)).toBe(NO_OWNER)
		expect(ownerFromSelect(NO_OWNER)).toBeNull()
		expect(ownerFromSelect('m1')).toBe('m1')
		expect(ownerToSelect('m1')).toBe('m1')
	})
})

describe('canChangeClient', () => {
	it('is open while the quote has no line, or only blank ones', () => {
		expect(canChangeClient([])).toBe(true)
		expect(canChangeClient([emptyQuoteLine()])).toBe(true)
	})

	it('locks as soon as a line carries a label, a price or a catalogue item', () => {
		expect(canChangeClient([{ ...emptyQuoteLine(), label: 'Pose' }])).toBe(
			false,
		)
		expect(canChangeClient([{ ...emptyQuoteLine(), unitPrice: '10' }])).toBe(
			false,
		)
		expect(
			canChangeClient([{ ...emptyQuoteLine(), catalogItemId: 'c1' }]),
		).toBe(false)
	})
})

describe('validityNote', () => {
	it('promises the default validity while no date is set', () => {
		expect(validityNote('SENT', null, TODAY)).toEqual({
			text: `Valable ${DEFAULT_VALIDITY_DAYS} jours après l’envoi`,
			expired: false,
		})
	})

	it('names the date', () => {
		const note = validityNote('SENT', '2026-12-01', TODAY)

		expect(note.text).toMatch(/^Valable jusqu’au 1(er)? déc/)
		expect(note.expired).toBe(false)
	})

	it('flags a sent quote past its date as expired', () => {
		expect(validityNote('SENT', '2026-11-01', TODAY).expired).toBe(true)
	})

	it('also flags a draft whose date has already passed', () => {
		expect(validityNote('DRAFT', '2026-11-01', TODAY).expired).toBe(true)
	})

	it('does not flag an accepted quote', () => {
		expect(validityNote('ACCEPTED', '2026-11-01', TODAY).expired).toBe(false)
	})
})
