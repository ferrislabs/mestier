import { describe, expect, it } from 'vitest'
import {
	addTags,
	normalizeTag,
	normalizeTags,
	parseTagInput,
	removeTag,
	tagsError,
} from '#/pages/photo-library/lib/tags'

describe('normalizeTag', () => {
	it('trims, lowercases and collapses inner spaces', () => {
		expect(normalizeTag('  Buis   Taillé ')).toBe('buis taillé')
	})
})

describe('normalizeTags', () => {
	it('drops empty entries and duplicates, keeping the first order', () => {
		expect(
			normalizeTags(['Buis', ' ', 'buis', 'Terrasse', 'TERRASSE']),
		).toEqual(['buis', 'terrasse'])
	})
})

describe('parseTagInput', () => {
	it('splits on commas, semicolons and line breaks', () => {
		expect(parseTagInput('Buis, terrasse;Allée\nbuis')).toEqual([
			'buis',
			'terrasse',
			'allée',
		])
	})

	it('is empty for a blank input', () => {
		expect(parseTagInput('  , ')).toEqual([])
	})
})

describe('addTags and removeTag', () => {
	it('merges the typed tags without duplicating', () => {
		expect(addTags(['buis'], 'Buis, Haie')).toEqual(['buis', 'haie'])
	})

	it('removes one tag', () => {
		expect(removeTag(['buis', 'haie'], 'buis')).toEqual(['haie'])
	})
})

describe('tagsError', () => {
	it('accepts up to 20 tags of 40 characters', () => {
		const tags = Array.from({ length: 20 }, (_, index) => `t${index}`)
		expect(tagsError(tags)).toBeNull()
		expect(tagsError(['a'.repeat(40)])).toBeNull()
	})

	it('refuses a 21st tag and a 41 characters tag', () => {
		const tags = Array.from({ length: 21 }, (_, index) => `t${index}`)
		expect(tagsError(tags)).toMatch(/20/)
		expect(tagsError(['a'.repeat(41)])).toMatch(/40/)
	})
})
