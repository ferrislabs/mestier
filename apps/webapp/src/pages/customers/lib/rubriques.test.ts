import { describe, expect, it } from 'vitest'
import {
	activitySummary,
	addressesSummary,
	contactsSummary,
	formatSiren,
	identitySummary,
	isClientTab,
	validateClientSearch,
} from '#/pages/customers/lib/rubriques'

describe('tabs', () => {
	it('accepts only the four rubriques', () => {
		expect(isClientTab('contacts')).toBe(true)
		expect(isClientTab('devis')).toBe(false)
		expect(validateClientSearch({ onglet: 'activite' })).toEqual({
			onglet: 'activite',
		})
		expect(validateClientSearch({ onglet: 'x' })).toEqual({})
	})
})

describe('summaries', () => {
	it('words counts', () => {
		expect(contactsSummary(0)).toBe('Aucun contact')
		expect(contactsSummary(1)).toBe('1 contact')
		expect(contactsSummary(3)).toBe('3 contacts')
		expect(addressesSummary(2)).toBe('2 adresses')
		expect(activitySummary(0)).toBe('Aucun encours')
	})

	it('formats a SIREN and describes the identity', () => {
		expect(formatSiren('823456789')).toBe('823 456 789')
		expect(formatSiren('n/a')).toBe('n/a')
		expect(
			identitySummary({ registration_number: '823456789' }, 'professionnel', [
				{ city: ' Rezé ' },
			]),
		).toBe('SIREN 823 456 789 · Rezé')
		expect(
			identitySummary({ registration_number: null }, 'professionnel', []),
		).toBe('SIREN non renseigné')
		expect(
			identitySummary({ registration_number: null }, 'particulier', []),
		).toBe('Particulier')
	})
})
