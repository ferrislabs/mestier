import { describe, expect, it } from 'vitest'
import {
	buildAnswers,
	draftsFromAnswers,
	toAnswer,
	validateDraft,
} from '#/pages/survey-respond/lib/answer-draft'
import { question } from '#/pages/survey-respond/test/respond-fixtures'

describe('validateDraft', () => {
	it.each([
		['TEXT', '  ', 'Bonjour'],
		['LONG_TEXT', '', 'Un long texte'],
		['NUMBER', '', '12,5'],
		['DATE', '', '2026-11-14'],
		['DATETIME', '', '2026-11-14T09:30'],
		['PHONE', '', '+33 6 12 34 56 78'],
		['EMAIL', '', 'eleve@lycee.fr'],
	] as const)('%s refuses an empty required answer and accepts a filled one', (kind, empty, filled) => {
		const node = question('q', kind, { required: true })
		expect(validateDraft(node, empty, 0)).toBe(
			'Cette question est obligatoire.',
		)
		expect(validateDraft(node, filled, 0)).toBeNull()
	})

	it('accepts an empty optional answer', () => {
		expect(validateDraft(question('q', 'EMAIL'), '', 0)).toBeNull()
	})

	it.each([
		['NUMBER', 'douze', 'Saisissez un nombre.'],
		['NUMBER', 'Infinity', 'Saisissez un nombre.'],
		['DATE', '2026-02-31', 'Saisissez une date valide.'],
		['DATETIME', '2026-11-14', 'Saisissez une date et une heure valides.'],
		[
			'PHONE',
			'12345',
			'Saisissez un numéro de téléphone valide, de 6 à 15 chiffres.',
		],
		[
			'PHONE',
			'06 12 ab',
			'Saisissez un numéro de téléphone valide, de 6 à 15 chiffres.',
		],
		[
			'PHONE',
			'1+23456789',
			'Saisissez un numéro de téléphone valide, de 6 à 15 chiffres.',
		],
		[
			'EMAIL',
			'a@b@c',
			'Saisissez une adresse e-mail valide, par exemple nom@exemple.fr.',
		],
		[
			'EMAIL',
			'nom exemple.fr',
			'Saisissez une adresse e-mail valide, par exemple nom@exemple.fr.',
		],
		[
			'EMAIL',
			'@exemple.fr',
			'Saisissez une adresse e-mail valide, par exemple nom@exemple.fr.',
		],
	] as const)('%s refuses the malformed %s', (kind, value, message) => {
		expect(validateDraft(question('q', kind), value, 0)).toBe(message)
	})

	it('checks a single choice against the options', () => {
		const node = question('q', 'SINGLE_CHOICE', {
			required: true,
			options: ['Oui', 'Non'],
		})
		expect(validateDraft(node, '', 0)).toBe('Choisissez une option.')
		expect(validateDraft(node, 'Peut-être', 0)).toBe(
			'Choisissez une option proposée.',
		)
		expect(validateDraft(node, 'Oui', 0)).toBeNull()
	})

	it('checks multiple choices against the options', () => {
		const node = question('q', 'MULTIPLE_CHOICE', {
			required: true,
			options: ['Bois', 'Métal'],
		})
		expect(validateDraft(node, [], 0)).toBe('Choisissez au moins une option.')
		expect(validateDraft(node, ['Bois', 'Verre'], 0)).toBe(
			'Choisissez une option proposée.',
		)
		expect(validateDraft(node, ['Bois', 'Métal'], 0)).toBeNull()
	})

	it('requires a photo and caps them at max_photos, five by default', () => {
		const required = question('q', 'PHOTO', { required: true, max_photos: 2 })
		expect(validateDraft(required, [], 0)).toBe('Ajoutez au moins une photo.')
		expect(validateDraft(required, ['a', 'b', 'c'], 0)).toBe(
			'Vous pouvez joindre au plus 2 photos.',
		)
		expect(validateDraft(required, ['a'], 0)).toBeNull()
		expect(validateDraft(required, ['a'], 1)).toBe(
			'Attendez la fin de l’envoi des photos.',
		)
		const byDefault = question('q', 'PHOTO')
		expect(validateDraft(byDefault, ['1', '2', '3', '4', '5'], 0)).toBeNull()
		expect(validateDraft(byDefault, ['1', '2', '3', '4', '5', '6'], 0)).toBe(
			'Vous pouvez joindre au plus 5 photos.',
		)
	})
})

describe('answers', () => {
	it('converts each kind to the backend answer shape', () => {
		expect(toAnswer(question('q', 'TEXT'), ' Jean ')).toEqual({
			Text: 'Jean',
		})
		expect(toAnswer(question('q', 'NUMBER'), '12,5')).toEqual({ Number: 12.5 })
		expect(toAnswer(question('q', 'DATE'), '2026-11-14')).toEqual({
			Date: '2026-11-14',
		})
		expect(toAnswer(question('q', 'PHONE'), '06 12 34 56 78')).toEqual({
			Phone: '06 12 34 56 78',
		})
		expect(toAnswer(question('q', 'EMAIL'), ' a@b.fr ')).toEqual({
			Email: 'a@b.fr',
		})
		expect(
			toAnswer(question('q', 'SINGLE_CHOICE', { options: ['Oui'] }), 'Oui'),
		).toEqual({ Choice: 'Oui' })
		expect(
			toAnswer(question('q', 'MULTIPLE_CHOICE', { options: ['A', 'B'] }), [
				'A',
				'B',
			]),
		).toEqual({ Choices: ['A', 'B'] })
		expect(toAnswer(question('q', 'PHOTO'), ['k1', 'k2'])).toEqual({
			Photos: ['k1', 'k2'],
		})
		const dateTime = toAnswer(question('q', 'DATETIME'), '2026-11-14T09:30')
		expect(dateTime).toEqual({
			DateTime: new Date('2026-11-14T09:30').toISOString(),
		})
	})

	it('leaves out blank and invalid drafts so a save is never refused for them', () => {
		const nodes = [
			question('a', 'TEXT'),
			question('b', 'EMAIL'),
			question('c', 'PHONE'),
			question('d', 'MULTIPLE_CHOICE', { options: ['X'] }),
		]
		expect(
			buildAnswers(nodes, { a: 'Jean', b: 'pas-un-mail', c: '', d: [] }),
		).toEqual({ a: { Text: 'Jean' } })
	})

	it('restores drafts from saved answers', () => {
		const nodes = [
			question('a', 'TEXT'),
			question('n', 'NUMBER'),
			question('t', 'DATETIME'),
			question('m', 'MULTIPLE_CHOICE', { options: ['X', 'Y'] }),
			question('p', 'PHOTO'),
		]
		const iso = new Date('2026-11-14T09:30').toISOString()
		expect(
			draftsFromAnswers(nodes, {
				a: { Text: 'Jean' },
				n: { Number: 3 },
				t: { DateTime: iso },
				m: { Choices: ['Y'] },
				p: { Photos: ['k1'] },
				ghost: { Text: 'x' },
			}),
		).toEqual({
			a: 'Jean',
			n: '3',
			t: '2026-11-14T09:30',
			m: ['Y'],
			p: ['k1'],
		})
	})
})
