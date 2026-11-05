import { describe, expect, it } from 'vitest'
import { labelConflict, uniqueLabel } from '#/pages/automation/lib/references'

describe('uniqueLabel', () => {
	it('keeps a free label', () => {
		expect(uniqueLabel('Envoyer', ['Autre'])).toBe('Envoyer')
	})

	it('numbers a taken label from 2', () => {
		expect(uniqueLabel('Envoyer', ['Envoyer', 'Envoyer 2'])).toBe('Envoyer 3')
	})
})

describe('labelConflict', () => {
	const nodes = [
		{ id: 'a', data: { label: 'Un' } },
		{ id: 'b', data: { label: 'Deux' } },
	]

	it('refuses an empty name', () => {
		expect(labelConflict('  ', 'a', nodes)).toBe(
			'Le nom ne peut pas être vide.',
		)
	})

	it('refuses the name of another node but not its own', () => {
		expect(labelConflict('Deux', 'a', nodes)).toBe(
			'Un autre nœud porte déjà ce nom.',
		)
		expect(labelConflict('Un', 'a', nodes)).toBeNull()
	})
})
