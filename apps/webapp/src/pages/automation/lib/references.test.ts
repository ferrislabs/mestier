import { describe, expect, it } from 'vitest'
import {
	labelConflict,
	renameReferencesInText,
	renameReferencesInValue,
	uniqueLabel,
} from '#/pages/automation/lib/references'
import { variableExpression } from '#/pages/automation/lib/variables'

describe('renameReferencesInText', () => {
	it('follows the renamed node and leaves a name that merely starts the same', () => {
		expect(
			renameReferencesInText(
				"Prix {{ $('Tarifs').item.json.body.prix }} et {{ $('Tarifs 2').item.json.x }}",
				'Tarifs',
				'Prix fournisseur',
			),
		).toBe(
			"Prix {{ $('Prix fournisseur').item.json.body.prix }} et {{ $('Tarifs 2').item.json.x }}",
		)
	})

	it('renames every form that reads the node', () => {
		expect(
			renameReferencesInText(
				"{{ $('A').first().json.x }} {{ len($('A').all()) }} {{ $('A').item.json }}",
				'A',
				'B',
			),
		).toBe(
			"{{ $('B').first().json.x }} {{ len($('B').all()) }} {{ $('B').item.json }}",
		)
	})

	it('writes and reads names that hold an apostrophe', () => {
		const before = variableExpression("Prix d'achat", ['prix'])
		expect(renameReferencesInText(before, "Prix d'achat", "Coût d'achat")).toBe(
			variableExpression("Coût d'achat", ['prix']),
		)
	})

	it('renames inside a JSON body kept as text, where the apostrophe escape is escaped again', () => {
		const body = JSON.stringify({
			prix: variableExpression("Prix d'achat", ['prix']),
		})
		const renamed = renameReferencesInText(body, "Prix d'achat", "Coût d'achat")
		expect(JSON.parse(renamed).prix).toBe(
			variableExpression("Coût d'achat", ['prix']),
		)
	})

	it('does not touch the double-quoted form, the case or a text outside $()', () => {
		const text = `{{ $("Tarifs").item.json.x }} {{ $('tarifs').item.json.x }} Tarifs`
		expect(renameReferencesInText(text, 'Tarifs', 'Autre')).toBe(text)
	})

	it('is a no-op when the name does not change', () => {
		const text = "{{ $('Tarifs').item.json.x }}"
		expect(renameReferencesInText(text, 'Tarifs', 'Tarifs')).toBe(text)
	})
})

describe('renameReferencesInValue', () => {
	const rename = (value: unknown) =>
		renameReferencesInValue(value, 'Tarifs', 'Prix fournisseur')

	it('rewrites strings in nested objects and arrays and keeps the rest', () => {
		const config = {
			url: "{{ $('Tarifs').item.json.url }}",
			retries: 3,
			active: true,
			none: null,
			headers: { Authorization: "Bearer {{ $('Tarifs').item.json.token }}" },
			body: {
				lignes: [
					{ code: "{{ $('Tarifs').item.json.code }}", qty: 2 },
					["{{ $('Tarifs').first().json.a }}"],
				],
			},
		}

		expect(rename(config)).toEqual({
			url: "{{ $('Prix fournisseur').item.json.url }}",
			retries: 3,
			active: true,
			none: null,
			headers: {
				Authorization: "Bearer {{ $('Prix fournisseur').item.json.token }}",
			},
			body: {
				lignes: [
					{ code: "{{ $('Prix fournisseur').item.json.code }}", qty: 2 },
					["{{ $('Prix fournisseur').first().json.a }}"],
				],
			},
		})
	})

	it('hands back the very same value when nothing refers to the node', () => {
		const config = { url: 'https://x.test', body: { a: [1, 'b'] } }

		expect(rename(config)).toBe(config)
	})

	it('does not mutate what it was given', () => {
		const config = { body: { a: ["{{ $('Tarifs').item.json.a }}"] } }
		const snapshot = structuredClone(config)

		rename(config)

		expect(config).toEqual(snapshot)
	})
})

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
