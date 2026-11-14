import { describe, expect, it } from 'vitest'
import {
	CONDITION_ENTRY,
	END_ENTRY,
	isChoiceKind,
	nodeChoices,
	QUESTION_KINDS,
	questionKindEntry,
} from '#/pages/survey/lib/question-kinds'
import type { QuestionKind } from '#/pages/survey/types'

const ALL_KINDS: QuestionKind[] = [
	'TEXT',
	'LONG_TEXT',
	'NUMBER',
	'SINGLE_CHOICE',
	'MULTIPLE_CHOICE',
	'DATE',
	'DATETIME',
	'PHONE',
	'EMAIL',
	'PHOTO',
]

describe('QUESTION_KINDS', () => {
	it('lists the ten question kinds exactly once', () => {
		expect(QUESTION_KINDS.map((entry) => entry.kind).sort()).toEqual(
			[...ALL_KINDS].sort(),
		)
	})

	it('gives every kind an icon, a French title and a description', () => {
		for (const entry of [...QUESTION_KINDS, CONDITION_ENTRY, END_ENTRY]) {
			expect(entry.icon).toBeTruthy()
			expect(entry.title.trim()).not.toBe('')
			expect(entry.description.trim()).not.toBe('')
		}
	})

	it('never repeats a title nor an icon', () => {
		const entries = [...QUESTION_KINDS, CONDITION_ENTRY, END_ENTRY]
		expect(new Set(entries.map((entry) => entry.title)).size).toBe(
			entries.length,
		)
		expect(new Set(entries.map((entry) => entry.icon)).size).toBe(
			entries.length,
		)
	})

	it('finds the entry of a kind', () => {
		expect(questionKindEntry('PHOTO').kind).toBe('PHOTO')
	})

	it('tells the choice kinds apart', () => {
		expect(ALL_KINDS.filter(isChoiceKind)).toEqual([
			'SINGLE_CHOICE',
			'MULTIPLE_CHOICE',
		])
	})

	it('offers the ten kinds then the condition and the end in the picker', () => {
		expect(nodeChoices().map((entry) => entry.choice)).toEqual([
			...QUESTION_KINDS.map((entry) => entry.kind),
			'CONDITION',
			'END',
		])
	})
})
