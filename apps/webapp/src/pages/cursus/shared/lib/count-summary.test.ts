import { describe, expect, it } from 'vitest'
import {
	archiveQualifier,
	countSummary,
	ESTABLISHMENT_NOUN,
	PARTICIPANT_NOUN,
	PROMOTION_NOUN,
	TRIAD_NOUN,
} from '#/pages/cursus/shared/lib/count-summary'

describe('countSummary', () => {
	it('agrees a feminine noun and its qualifier in the plural', () => {
		expect(countSummary(3, PROMOTION_NOUN, 'active')).toBe(
			'3 promotions actives',
		)
	})

	it('keeps the singular for one item', () => {
		expect(countSummary(1, PROMOTION_NOUN, 'archived')).toBe(
			'1 promotion archivée',
		)
	})

	it('drops the qualifier when every item is listed', () => {
		expect(countSummary(12, PROMOTION_NOUN)).toBe('12 promotions')
	})

	it('agrees a masculine noun', () => {
		expect(countSummary(32, TRIAD_NOUN, 'open')).toBe('32 trinômes ouverts')
		expect(countSummary(2, ESTABLISHMENT_NOUN, 'active')).toBe(
			'2 établissements actifs',
		)
		expect(countSummary(4, TRIAD_NOUN, 'closed')).toBe('4 trinômes clos')
	})

	it('keeps an invariable qualifier', () => {
		expect(countSummary(5, PARTICIPANT_NOUN, 'pending')).toBe(
			'5 participants en attente',
		)
	})

	it('says none with the right gender', () => {
		expect(countSummary(0, PROMOTION_NOUN, 'active')).toBe(
			'Aucune promotion active',
		)
		expect(countSummary(0, TRIAD_NOUN)).toBe('Aucun trinôme')
	})
})

describe('archiveQualifier', () => {
	it('maps the archive filter to a qualifier', () => {
		expect(archiveQualifier('active')).toBe('active')
		expect(archiveQualifier('archived')).toBe('archived')
		expect(archiveQualifier('all')).toBeUndefined()
	})
})
