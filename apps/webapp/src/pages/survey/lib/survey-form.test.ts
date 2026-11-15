import { describe, expect, it } from 'vitest'
import {
	addCondition,
	addToBlock,
	blockMembers,
	changeKind,
	dependentsOf,
	dropQuestion,
	duplicateQuestion,
	governingRule,
	groupWithPrevious,
	insertBlock,
	insertQuestion,
	kindChangeDrops,
	moveBlock,
	moveQuestion,
	newQuestion,
	removeCondition,
	removeOption,
	removeQuestion,
	renameOption,
	ruleSources,
	type SurveyForm,
	ungroupQuestion,
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

	it('reads a rule back as all once a single condition remains', () => {
		const form: SurveyForm = {
			...base(),
			questions: base().questions.map((item) =>
				item.id === 'c'
					? {
							...item,
							rule: {
								join: 'ANY',
								conditions: [
									{ question: 'a', option: 'Oui', checked: true },
									{ question: 'a', option: 'Non', checked: true },
								],
							},
						}
					: item,
			),
		}
		expect(removeCondition(form, 'c', 1).questions[2].rule).toEqual({
			join: 'ALL',
			conditions: [{ question: 'a', option: 'Oui', checked: true }],
		})
	})

	it('drops a question onto another through the move rules', () => {
		expect(dropQuestion(base(), 'b', 'b')).toBeNull()
		expect(dropQuestion(base(), 'b', null)).toBeNull()
		const moved = dropQuestion(base(), 'b', 'a')
		expect(moved?.kind).toBe('moved')
		if (moved?.kind !== 'moved') throw new Error('moved expected')
		expect(moved.form.questions.map((item) => item.id)).toEqual(['b', 'a', 'c'])
		expect(moved.position).toBe(0)
		expect(dropQuestion(base(), 'c', 'a')?.kind).toBe('refused')
		expect(dropQuestion(base(), 'a', 'c')?.kind).toBe('refused')
	})

	it('drops the conditions of a removed option', () => {
		const result = removeOption(base(), 'a', 0)
		expect(result.removed).toBe(1)
		expect(result.form.questions[0].options).toEqual(['Non'])
		expect(result.form.questions[2].rule).toBeNull()
	})

	it('does not move conditions between duplicate option texts', () => {
		const form: SurveyForm = {
			...base(),
			questions: base().questions.map((item) =>
				item.id === 'a' ? { ...item, options: ['Oui', 'Oui', 'Non'] } : item,
			),
		}
		const next = renameOption(form, 'a', 1, 'Peut-être')
		expect(next.questions[0].options).toEqual(['Oui', 'Peut-être', 'Non'])
		expect(next.questions[2].rule?.conditions[0].option).toBe('Oui')
	})

	it('keeps the typed options across kind changes and prunes only dependent conditions', () => {
		const multiple = changeKind(base(), 'a', 'MULTIPLE_CHOICE')
		expect(multiple.removed).toBe(0)
		expect(multiple.form.questions[0].options).toEqual(['Oui', 'Non'])
		expect(kindChangeDrops(base(), 'a', 'TEXT')).toBe(1)
		expect(kindChangeDrops(base(), 'a', 'MULTIPLE_CHOICE')).toBe(0)
		expect(kindChangeDrops(base(), 'b', 'DATE')).toBe(0)
		const text = changeKind(base(), 'a', 'TEXT')
		expect(text.removed).toBe(1)
		expect(text.form.questions[0].options).toEqual(['Oui', 'Non'])
		expect(text.form.questions[2].rule).toBeNull()
		const back = changeKind(text.form, 'a', 'SINGLE_CHOICE')
		expect(back.form.questions[0].options).toEqual(['Oui', 'Non'])
		const dated = changeKind(
			{
				...base(),
				questions: [{ ...newQuestion('p', 'PHOTO'), maxPhotos: 3 }],
			},
			'p',
			'DATE',
		)
		expect(
			changeKind(dated.form, 'p', 'PHOTO').form.questions[0].maxPhotos,
		).toBe(3)
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

const RULE = {
	join: 'ALL' as const,
	conditions: [{ question: 'a', option: 'Oui', checked: true }],
}

function blocky(): SurveyForm {
	return {
		endId: 'end',
		endMessage: '',
		questions: [
			{ ...newQuestion('a', 'SINGLE_CHOICE'), options: ['Oui', 'Non'] },
			{ ...newQuestion('b', 'TEXT'), block: 'B', rule: RULE },
			{ ...newQuestion('c', 'TEXT'), block: 'B' },
			{ ...newQuestion('d', 'TEXT'), block: 'B' },
			{ ...newQuestion('e', 'SINGLE_CHOICE'), options: ['X'] },
			newQuestion('f', 'TEXT'),
		],
	}
}

function ids(form: SurveyForm): string[] {
	return form.questions.map((item) =>
		item.block ? `${item.id}:${item.block}` : item.id,
	)
}

describe('survey form blocks', () => {
	it('creates a question outside any block', () => {
		expect(newQuestion('x', 'TEXT').block).toBeNull()
	})

	it('inserts a block of two empty questions', () => {
		const next = insertBlock(base(), ['x', 'y'], 'N', 1)
		expect(ids(next)).toEqual(['a', 'x:N', 'y:N', 'b', 'c'])
		expect(next.questions[1].label).toBe('')
		expect(next.questions[1].kind).toBe('TEXT')
	})

	it('adds a question at the end of a block', () => {
		const next = addToBlock(blocky(), 'B', newQuestion('x', 'DATE'))
		expect(ids(next)).toEqual(['a', 'b:B', 'c:B', 'd:B', 'x:B', 'e', 'f'])
		expect(next.questions[4].rule).toBeNull()
	})

	it('reads the rule of a member from the first member of its block', () => {
		expect(governingRule(blocky(), 'd')).toEqual(RULE)
		expect(governingRule(blocky(), 'e')).toBeNull()
		expect(blockMembers(blocky(), 'c').map((item) => item.id)).toEqual([
			'b',
			'c',
			'd',
		])
		expect(blockMembers(blocky(), 'e')).toEqual([])
	})

	it('offers as sources only always-asked choice questions before the block', () => {
		const form = addToBlock(blocky(), 'B', {
			...newQuestion('x', 'SINGLE_CHOICE'),
			options: ['Z'],
		})
		expect(ruleSources(form, 'b').map((item) => item.id)).toEqual(['a'])
		expect(ruleSources(form, 'f').map((item) => item.id)).toEqual(['a', 'e'])
	})

	it('groups a question with a lone previous question into a new block', () => {
		const result = groupWithPrevious(blocky(), 'f', 'N')
		expect(result.kind).toBe('grouped')
		if (result.kind !== 'grouped') return
		expect(ids(result.form)).toEqual(['a', 'b:B', 'c:B', 'd:B', 'e:N', 'f:N'])
		expect(result.size).toBe(2)
	})

	it('groups a question into the block of the previous question', () => {
		const result = groupWithPrevious(blocky(), 'e', 'N')
		expect(result.kind).toBe('grouped')
		if (result.kind !== 'grouped') return
		expect(ids(result.form)).toEqual(['a', 'b:B', 'c:B', 'd:B', 'e:B', 'f'])
		expect(result.size).toBe(4)
	})

	it('refuses to group the first question, a question with its own rule, or a source into a conditional block', () => {
		expect(groupWithPrevious(blocky(), 'a', 'N').kind).toBe('refused')
		const ruled = { ...base() }
		expect(groupWithPrevious(ruled, 'c', 'N').kind).toBe('refused')
		const sourced: SurveyForm = {
			...blocky(),
			questions: blocky().questions.map((item) =>
				item.id === 'f'
					? {
							...item,
							rule: {
								join: 'ALL',
								conditions: [{ question: 'e', option: 'X', checked: true }],
							},
						}
					: item,
			),
		}
		expect(groupWithPrevious(sourced, 'e', 'N').kind).toBe('refused')
	})

	it('takes a middle member out after its block', () => {
		const result = ungroupQuestion(blocky(), 'c')
		expect(ids(result.form)).toEqual(['a', 'b:B', 'd:B', 'c', 'e', 'f'])
		expect(result.form.questions[3].rule).toEqual(RULE)
		expect(result.dissolved).toBe(false)
	})

	it('takes the first member out before its block and keeps the rule on the block', () => {
		const result = ungroupQuestion(blocky(), 'b')
		expect(ids(result.form)).toEqual(['a', 'b', 'c:B', 'd:B', 'e', 'f'])
		expect(result.form.questions[1].rule).toEqual(RULE)
		expect(result.form.questions[2].rule).toEqual(RULE)
		expect(result.form.questions[3].rule).toBeNull()
	})

	it('dissolves a block that falls to one member', () => {
		const once = ungroupQuestion(blocky(), 'd').form
		const twice = ungroupQuestion(once, 'c')
		expect(twice.dissolved).toBe(true)
		expect(ids(twice.form)).toEqual(['a', 'b', 'c', 'd', 'e', 'f'])
		expect(twice.form.questions[1].rule).toEqual(RULE)
	})

	it('dissolves a block when deleting members and moves the rule to the new first member', () => {
		const first = removeQuestion(blocky(), 'b')
		expect(ids(first)).toEqual(['a', 'c:B', 'd:B', 'e', 'f'])
		expect(first.questions[1].rule).toEqual(RULE)
		const second = removeQuestion(first, 'd')
		expect(ids(second)).toEqual(['a', 'c', 'e', 'f'])
		expect(second.questions[1].rule).toEqual(RULE)
	})

	it('duplicates a member inside its block without a rule', () => {
		const next = duplicateQuestion(blocky(), 'b', 'b2')
		expect(ids(next)).toEqual(['a', 'b:B', 'b2:B', 'c:B', 'd:B', 'e', 'f'])
		expect(next.questions[2].rule).toBeNull()
	})

	it('moves a member only inside its block and keeps the rule first', () => {
		const down = moveQuestion(blocky(), 'b', 1)
		expect(down.kind).toBe('moved')
		if (down.kind !== 'moved') return
		expect(ids(down.form)).toEqual(['a', 'c:B', 'b:B', 'd:B', 'e', 'f'])
		expect(down.form.questions[1].rule).toEqual(RULE)
		expect(down.form.questions[2].rule).toBeNull()
		expect(moveQuestion(blocky(), 'b', -1).kind).toBe('refused')
		expect(moveQuestion(blocky(), 'd', 1).kind).toBe('refused')
	})

	it('moves a lone question over a whole block', () => {
		const result = moveQuestion(blocky(), 'e', -1)
		expect(result.kind).toBe('moved')
		if (result.kind !== 'moved') return
		expect(ids(result.form)).toEqual(['a', 'e', 'b:B', 'c:B', 'd:B', 'f'])
		expect(result.position).toBe(1)
	})

	it('moves a block as a unit and refuses to lift it above its source', () => {
		const down = moveBlock(blocky(), 'B', 1)
		expect(down.kind).toBe('moved')
		if (down.kind !== 'moved') return
		expect(ids(down.form)).toEqual(['a', 'e', 'b:B', 'c:B', 'd:B', 'f'])
		expect(down.position).toBe(2)
		expect(moveBlock(blocky(), 'B', -1).kind).toBe('refused')
	})

	it('drops a block onto a question, a question onto a block, and keeps members inside', () => {
		const block = dropQuestion(blocky(), 'B', 'f')
		expect(block?.kind).toBe('moved')
		if (block?.kind === 'moved') {
			expect(ids(block.form)).toEqual(['a', 'e', 'f', 'b:B', 'c:B', 'd:B'])
		}
		const lone = dropQuestion(blocky(), 'f', 'c')
		expect(lone?.kind).toBe('moved')
		if (lone?.kind === 'moved') {
			expect(ids(lone.form)).toEqual(['a', 'f', 'b:B', 'c:B', 'd:B', 'e'])
		}
		const inside = dropQuestion(blocky(), 'd', 'c')
		expect(inside?.kind).toBe('moved')
		if (inside?.kind === 'moved') {
			expect(ids(inside.form)).toEqual(['a', 'b:B', 'd:B', 'c:B', 'e', 'f'])
		}
		expect(dropQuestion(blocky(), 'c', 'f')?.kind).toBe('refused')
	})
})
