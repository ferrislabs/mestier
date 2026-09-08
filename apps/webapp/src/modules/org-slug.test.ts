import { describe, expect, it } from 'vitest'
import {
	normalizeSlugForPayload,
	normalizeSlugInput,
	slugFromName,
} from '#/modules/org-slug'

describe('slugFromName', () => {
	it('lowercases, strips accents and joins words with dashes', () => {
		expect(slugFromName('Éts Dupont & Fils')).toBe('ets-dupont-fils')
	})

	it('collapses repeated dashes and trims the edges', () => {
		expect(slugFromName('  -- Paysages   Bonnal -- ')).toBe('paysages-bonnal')
	})
})

describe('normalizeSlugInput', () => {
	it('keeps a trailing dash so a slug can still be typed word by word', () => {
		expect(normalizeSlugInput('entreprise-')).toBe('entreprise-')
	})

	it('turns a typed space into a dash instead of swallowing it', () => {
		expect(normalizeSlugInput('entreprise ')).toBe('entreprise-')
	})

	it('drops characters a slug cannot carry', () => {
		expect(normalizeSlugInput('Entreprise Dupont!')).toBe('entreprise-dupont')
	})
})

describe('normalizeSlugForPayload', () => {
	it('trims the dashes the input normalization deliberately leaves', () => {
		expect(normalizeSlugForPayload('-entreprise-dupont-')).toBe(
			'entreprise-dupont',
		)
	})
})
