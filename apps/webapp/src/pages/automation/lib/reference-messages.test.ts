import { describe, expect, it } from 'vitest'
import {
	LEGACY_SYNTAX_MESSAGE,
	missingKeyMessage,
	notUpstreamMessage,
	previewErrorMessage,
	translateReferenceError,
	unknownNodeMessage,
} from '#/pages/automation/lib/reference-messages'

describe('the messages about a reference', () => {
	it('say it the way the prototype does', () => {
		expect(unknownNodeMessage('Tarifs')).toBe(
			'Le nœud « Tarifs » n’existe plus.',
		)
		expect(notUpstreamMessage('Tarifs')).toBe(
			'Le nœud « Tarifs » n’est pas relié en amont de ce nœud.',
		)
		expect(missingKeyMessage('a.b', 'Tarifs')).toBe(
			'La clé « a.b » est absente de la sortie de « Tarifs ».',
		)
	})

	it('name the syntax to use for the old one', () => {
		expect(LEGACY_SYNTAX_MESSAGE).toContain('connectors.<id>.output')
		expect(LEGACY_SYNTAX_MESSAGE).toContain("$('Nom du nœud').item.json")
	})
})

describe('translateReferenceError', () => {
	it('turns the refusal of an unknown node into French, with the name of the node', () => {
		expect(
			translateReferenceError(
				'connector `c2` field `url` references unknown node `Rechercher les devis`',
			),
		).toBe('Le nœud « Rechercher les devis » n’existe plus.')
	})

	it('turns the refusal of a node that does not run before into French', () => {
		expect(
			translateReferenceError(
				'connector `c2` field `body.lignes.0.code` references node `Créer un projet`, which does not run before it',
			),
		).toBe('Le nœud « Créer un projet » n’est pas relié en amont de ce nœud.')
	})

	it('keeps a name that holds a back-quote', () => {
		expect(
			translateReferenceError(
				'connector `c2` field `url` references unknown node `Le `bon` nœud`',
			),
		).toBe('Le nœud « Le `bon` nœud » n’existe plus.')
	})

	it('makes a sentence of the refusal of the old syntax and drops its position', () => {
		expect(
			translateReferenceError(
				"cette expression utilise l'ancienne syntaxe `connectors.<id>.output` (position 3) : utilise $('Nom du nœud').item.json…",
			),
		).toBe(LEGACY_SYNTAX_MESSAGE)
	})

	it('leaves any other message as it came', () => {
		expect(translateReferenceError('syntax error at position 3: x')).toBe(
			'syntax error at position 3: x',
		)
	})
})

describe('previewErrorMessage', () => {
	it('says a node is unknown or does not run before', () => {
		expect(
			previewErrorMessage(
				'node `Tarifs` has no output available to this expression',
			),
		).toBe(
			'Le nœud « Tarifs » n’existe pas ou n’est pas relié en amont de ce nœud.',
		)
	})

	it('says a node produced no item', () => {
		expect(previewErrorMessage('node `Tarifs` produced no item')).toBe(
			'Le nœud « Tarifs » n’a produit aucun élément.',
		)
	})

	it('says the old syntax is refused', () => {
		expect(
			previewErrorMessage(
				"cette expression utilise l'ancienne syntaxe `connectors.<id>.output` (position 0) : utilise $('Nom du nœud').item.json…",
			),
		).toBe(LEGACY_SYNTAX_MESSAGE)
	})

	it('leaves any other message as it came', () => {
		expect(previewErrorMessage('missing path `trigger.id`')).toBe(
			'missing path `trigger.id`',
		)
	})
})
