import { describe, expect, it } from 'vitest'
import {
	nodeAppearance,
	nodeChoiceGroups,
	nodeTitle,
} from '#/pages/survey/lib/node-catalogue'
import { QUESTION_KINDS } from '#/pages/survey/lib/question-kinds'

describe('nodeChoiceGroups', () => {
	it('offers the ten question kinds then the condition and the end', () => {
		const groups = nodeChoiceGroups('')
		expect(groups.map((group) => group.label)).toEqual([
			'Questions',
			'Parcours',
		])
		expect(groups[0].entries.map((entry) => entry.title)).toEqual(
			QUESTION_KINDS.map((entry) => entry.title),
		)
		expect(groups[1].entries.map((entry) => entry.choice)).toEqual([
			'CONDITION',
			'END',
		])
	})

	it('filters on title and description without caring about accents or case', () => {
		const groups = nodeChoiceGroups('telephone')
		expect(groups).toHaveLength(1)
		expect(groups[0].entries.map((entry) => entry.choice)).toEqual(['PHONE'])
		expect(
			nodeChoiceGroups('MESSAGE DE FIN').flatMap((group) =>
				group.entries.map((entry) => entry.choice),
			),
		).toEqual(['END'])
	})

	it('returns nothing when no entry matches', () => {
		expect(nodeChoiceGroups('zzz')).toEqual([])
	})
})

describe('nodeTitle and nodeAppearance', () => {
	it('names a question by its label, or by its kind when it has none', () => {
		const question = {
			type: 'QUESTION' as const,
			id: 'q',
			kind: 'EMAIL' as const,
			label: '  ',
			required: false,
		}
		expect(nodeTitle(question)).toBe('E-mail sans libellé')
		expect(nodeTitle({ ...question, label: 'Votre e-mail' })).toBe(
			'Votre e-mail',
		)
		expect(nodeAppearance(question).typeLabel).toBe('E-mail')
	})

	it('names a condition and an end by their type', () => {
		expect(
			nodeTitle({
				type: 'CONDITION',
				id: 'c',
				question: '',
				option: '',
				checked: true,
			}),
		).toBe('Condition')
		expect(nodeTitle({ type: 'END', id: 'e', message: null })).toBe('Fin')
		expect(nodeTitle(undefined)).toBe('Nœud')
	})
})
