import { describe, expect, it } from 'vitest'
import { labelConflict, uniqueLabel } from '#/pages/automation/lib/references'

describe('uniqueLabel', () => {
	it('keeps a free label', () => {
		expect(uniqueLabel('Envoyer', ['Autre'])).toBe('Envoyer')
	})

	it('numbers a taken label from 2', () => {
		expect(uniqueLabel('Envoyer', ['Envoyer', 'Envoyer 2'])).toBe('Envoyer 3')
	})

	it('treats labels that differ only by case or surrounding spaces as taken, as the server does', () => {
		expect(uniqueLabel('Envoyer', ['envoyer'])).toBe('Envoyer 2')
		expect(uniqueLabel('Envoyer', [' ENVOYER '])).toBe('Envoyer 2')
		expect(uniqueLabel('Envoyer', ['ENVOYER', 'envoyer 2'])).toBe('Envoyer 3')
	})

	it('keeps accents significant', () => {
		expect(uniqueLabel('Envoyé', ['Envoye'])).toBe('Envoyé')
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

	it('refuses the name of another node whatever its case and surrounding spaces', () => {
		expect(labelConflict('deux', 'a', nodes)).toBe(
			'Un autre nœud porte déjà ce nom.',
		)
		expect(labelConflict('  DEUX ', 'a', nodes)).toBe(
			'Un autre nœud porte déjà ce nom.',
		)
	})

	it('lets a node change the case of its own name', () => {
		expect(labelConflict('UN', 'a', nodes)).toBeNull()
		expect(labelConflict(' un ', 'a', nodes)).toBeNull()
	})
})
