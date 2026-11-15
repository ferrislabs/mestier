import { describe, expect, it } from 'vitest'
import { conditionNodeId } from '#/pages/survey/lib/form-graph'
import {
	FORM_MESSAGES,
	formProblems,
	problemsByQuestion,
	questionOfNode,
} from '#/pages/survey/lib/form-validation'
import {
	type FormQuestion,
	newQuestion,
	type SurveyForm,
} from '#/pages/survey/lib/survey-form'
import type { QuestionKind } from '#/pages/survey/types'

function question(
	id: string,
	kind: QuestionKind,
	patch: Partial<FormQuestion> = {},
): FormQuestion {
	return { ...newQuestion(id, kind), label: `Question ${id}`, ...patch }
}

function form(questions: FormQuestion[]): SurveyForm {
	return { questions, endId: 'end', endMessage: '' }
}

function messagesFor(value: SurveyForm, id: string): string[] {
	return problemsByQuestion(formProblems(value, false)).get(id) ?? []
}

describe('form problems', () => {
	it('accepts a complete form', () => {
		const value = form([
			question('a', 'SINGLE_CHOICE', { options: ['Oui', 'Non'] }),
			question('b', 'TEXT', {
				rule: {
					join: 'ALL',
					conditions: [{ question: 'a', option: 'Oui', checked: true }],
				},
			}),
		])
		expect(formProblems(value, true)).toEqual([])
	})

	it('allows an empty draft and refuses an empty opened survey', () => {
		expect(formProblems(form([]), false)).toEqual([])
		expect(formProblems(form([]), true)).toEqual([
			{ questionId: null, message: FORM_MESSAGES.empty },
		])
	})

	it('flags labels, options, photos and identifiers on their card', () => {
		const value = form([
			question('a', 'SINGLE_CHOICE', { label: ' ' }),
			question('b', 'MULTIPLE_CHOICE', { options: ['X', 'X', ''] }),
			question('c', 'PHOTO', { maxPhotos: 25 }),
			question('d', 'TEXT', { identifier: true }),
			question('e', 'EMAIL', { identifier: true }),
		])
		expect(messagesFor(value, 'a')).toEqual([
			FORM_MESSAGES.label,
			FORM_MESSAGES.noOption,
		])
		expect(messagesFor(value, 'b')).toEqual([
			FORM_MESSAGES.blankOption,
			FORM_MESSAGES.duplicateOption,
		])
		expect(messagesFor(value, 'c')).toEqual([FORM_MESSAGES.maxPhotos])
		expect(messagesFor(value, 'd')).toEqual([])
		expect(messagesFor(value, 'e')).toEqual([FORM_MESSAGES.secondIdentifier])
	})

	it('flags incomplete and invalid rules', () => {
		const value = form([
			question('a', 'SINGLE_CHOICE', { options: ['Oui'] }),
			question('b', 'TEXT', {
				rule: {
					join: 'ALL',
					conditions: [{ question: '', option: '', checked: true }],
				},
			}),
			question('c', 'TEXT', {
				rule: {
					join: 'ALL',
					conditions: [{ question: 'a', option: 'Non', checked: true }],
				},
			}),
			question('d', 'TEXT', {
				rule: {
					join: 'ALL',
					conditions: [{ question: 'e', option: 'Oui', checked: true }],
				},
			}),
			question('e', 'SINGLE_CHOICE', { options: ['Oui'] }),
		])
		expect(messagesFor(value, 'b')).toEqual([FORM_MESSAGES.conditionIncomplete])
		expect(messagesFor(value, 'c')).toEqual([FORM_MESSAGES.conditionOption])
		expect(messagesFor(value, 'd')).toEqual([FORM_MESSAGES.conditionSource])
	})

	it('maps a condition node to the question that owns it', () => {
		const value = form([
			question('a', 'SINGLE_CHOICE', { options: ['Oui'] }),
			question('b', 'TEXT', {
				rule: {
					join: 'ALL',
					conditions: [{ question: 'a', option: 'Oui', checked: true }],
				},
			}),
		])
		expect(questionOfNode(value, conditionNodeId('b', 0))).toBe('b')
		expect(questionOfNode(value, 'a')).toBe('a')
		expect(questionOfNode(value, 'end')).toBeNull()
	})

	it('refuses a source that sits in a block shown under condition', () => {
		const rule = {
			join: 'ALL' as const,
			conditions: [{ question: 'a', option: 'Oui', checked: true }],
		}
		const value = form([
			question('a', 'SINGLE_CHOICE', { options: ['Oui'] }),
			question('b', 'TEXT', { block: 'B', rule }),
			question('c', 'SINGLE_CHOICE', { options: ['X'], block: 'B' }),
			question('d', 'TEXT', {
				rule: {
					join: 'ALL',
					conditions: [{ question: 'c', option: 'X', checked: true }],
				},
			}),
		])
		expect(messagesFor(value, 'd')).toEqual([FORM_MESSAGES.conditionSource])
		expect(messagesFor(value, 'b')).toEqual([])
	})
})
