import { describe, expect, it } from 'vitest'
import {
	candidateFieldKeys,
	toggleDisplayField,
} from '#/pages/support/lib/display-fields'

describe('candidateFieldKeys', () => {
	it('merges the configured keys with the keys found on tickets, sorted', () => {
		expect(
			candidateFieldKeys(
				[
					['ville', 'etage'],
					['ville', 'pays'],
				],
				['zone'],
			),
		).toEqual(['etage', 'pays', 'ville', 'zone'])
	})

	it('is empty without tickets and without configured keys', () => {
		expect(candidateFieldKeys([], [])).toEqual([])
	})
})

describe('toggleDisplayField', () => {
	const candidates = ['etage', 'pays', 'ville']

	it('adds a key at its place among the candidates', () => {
		expect(toggleDisplayField(['ville'], candidates, 'etage', true)).toEqual([
			'etage',
			'ville',
		])
	})

	it('removes a key', () => {
		expect(
			toggleDisplayField(['etage', 'ville'], candidates, 'etage', false),
		).toEqual(['ville'])
	})

	it('keeps a configured key that is not a candidate when adding another', () => {
		expect(toggleDisplayField(['zone'], candidates, 'ville', true)).toEqual([
			'ville',
		])
	})
})
