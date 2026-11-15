import { describe, expect, it } from 'vitest'
import {
	addCondition,
	changeKind,
	dependentsOf,
	duplicateQuestion,
	insertQuestion,
	moveQuestion,
	newQuestion,
	removeOption,
	removeQuestion,
	renameOption,
	ruleSources,
	type SurveyForm,
} from '#/pages/survey/lib/survey-form'

function base(): SurveyForm {
	return {
		endId: 'end',
		endMessage: '',
		questions: [
			{ ...newQuestion('a', 'SINGLE_CHOICE'), options: ['Oui', 'Non'] },
			{ ...newQuestion('b', 'TEXT'), label: 'B' },
			{
				...newQuestion('c', 'TEXT'),
				rule: {
					join: 'ALL',
					conditions: [{ question: 'a', option: 'Oui', checked: true }],
				},
			},
		],
	}
}

describe('survey form edition', () => {
	it('creates a choice question without any option', () => {
		expect(newQuestion('x', 'SINGLE_CHOICE').options).toEqual([])
		expect(newQuestion('x', 'MULTIPLE_CHOICE').rule).toBeNull()
	})

	it('inserts a question at a position', () => {
		const next = insertQuestion(base(), newQuestion('x', 'DATE'), 1)
		expect(next.questions.map((item) => item.id)).toEqual(['a', 'x', 'b', 'c'])
	})

	it('offers only earlier always-asked choice questions as sources', () => {
		const form = insertQuestion(
			base(),
			{
				...newQuestion('d', 'MULTIPLE_CHOICE'),
				options: ['X'],
				rule: {
					join: 'ALL',
					conditions: [{ question: 'a', option: 'Oui', checked: true }],
				},
			},
			3,
		)
		expect(ruleSources(form, 'c').map((item) => item.id)).toEqual(['a'])
		expect(ruleSources(form, 'a')).toEqual([])
		const later = insertQuestion(form, newQuestion('e', 'TEXT'), 4)
		expect(ruleSources(later, 'e').map((item) => item.id)).toEqual(['a'])
	})

	it('adds a condition on the first source and option', () => {
		const next = addCondition(base(), 'b')
		expect(next.questions[1].rule).toEqual({
			join: 'ALL',
			conditions: [{ question: 'a', option: 'Oui', checked: true }],
		})
	})

	it('refuses to move a source below its dependant', () => {
		const result = moveQuestion(base(), 'a', 2)
		expect(result.kind).toBe('refused')
		const below = moveQuestion(base(), 'c', -2)
		expect(below.kind).toBe('refused')
		const fine = moveQuestion(base(), 'b', 1)
		expect(fine.kind).toBe('moved')
		if (fine.kind === 'moved') {
			expect(fine.form.questions.map((item) => item.id)).toEqual([
				'a',
				'c',
				'b',
			])
		}
	})

	it('removes a source and the conditions that used it', () => {
		expect(dependentsOf(base(), 'a').map((item) => item.id)).toEqual(['c'])
		const next = removeQuestion(base(), 'a')
		expect(next.questions.map((item) => item.id)).toEqual(['b', 'c'])
		expect(next.questions[1].rule).toBeNull()
	})

	it('follows a renamed option in the rules', () => {
		const next = renameOption(base(), 'a', 0, 'Oui !')
		expect(next.questions[0].options).toEqual(['Oui !', 'Non'])
		expect(next.questions[2].rule?.conditions[0].option).toBe('Oui !')
	})

	it('drops the conditions of a removed option', () => {
		const result = removeOption(base(), 'a', 0)
		expect(result.removed).toBe(1)
		expect(result.form.questions[0].options).toEqual(['Non'])
		expect(result.form.questions[2].rule).toBeNull()
	})

	it('keeps options between choice kinds and clears them otherwise', () => {
		const multiple = changeKind(base(), 'a', 'MULTIPLE_CHOICE')
		expect(multiple.removed).toBe(0)
		expect(multiple.form.questions[0].options).toEqual(['Oui', 'Non'])
		const text = changeKind(base(), 'a', 'TEXT')
		expect(text.removed).toBe(1)
		expect(text.form.questions[0].options).toEqual([])
		expect(text.form.questions[2].rule).toBeNull()
		const photo = changeKind(
			{
				...base(),
				questions: [{ ...newQuestion('a', 'TEXT'), identifier: true }],
			},
			'a',
			'PHOTO',
		)
		expect(photo.form.questions[0].identifier).toBe(false)
	})

	it('duplicates a question after itself without the identifier', () => {
		const form = {
			...base(),
			questions: [
				...base().questions,
				{ ...newQuestion('d', 'EMAIL'), identifier: true },
			],
		}
		const next = duplicateQuestion(form, 'd', 'd2')
		expect(next.questions.map((item) => item.id)).toEqual([
			'a',
			'b',
			'c',
			'd',
			'd2',
		])
		expect(next.questions[4].identifier).toBe(false)
		const rule = duplicateQuestion(base(), 'c', 'c2')
		expect(rule.questions[3].rule).toEqual(base().questions[2].rule)
	})
})
