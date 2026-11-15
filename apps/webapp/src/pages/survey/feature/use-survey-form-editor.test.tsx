import { act, renderHook } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useSurveyFormEditor } from '#/pages/survey/feature/use-survey-form-editor'
import { newQuestion, type SurveyForm } from '#/pages/survey/lib/survey-form'

const toastMock = vi.hoisted(() => ({
	success: vi.fn(),
	error: vi.fn(),
	info: vi.fn(),
}))

vi.mock('sonner', () => ({ toast: toastMock }))

beforeEach(() => {
	toastMock.error.mockClear()
})

const FORM: SurveyForm = {
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

function drop(active: string, over: string | null) {
	return {
		active: { id: active } as never,
		over: over === null ? null : ({ id: over } as never),
	}
}

describe('useSurveyFormEditor onDragEnd', () => {
	it('reorders on a valid drop', () => {
		const onEdit = vi.fn()
		const { result } = renderHook(() => useSurveyFormEditor(FORM, onEdit))
		act(() => result.current.onDragEnd(drop('c', 'b')))
		expect(result.current.form.questions.map((item) => item.id)).toEqual([
			'a',
			'c',
			'b',
		])
		expect(onEdit).toHaveBeenCalledTimes(1)
	})

	it('snaps a dependant dropped above its source back with the explanation toast', () => {
		const onEdit = vi.fn()
		const { result } = renderHook(() => useSurveyFormEditor(FORM, onEdit))
		act(() => result.current.onDragEnd(drop('c', 'a')))
		expect(result.current.form).toBe(FORM)
		expect(onEdit).not.toHaveBeenCalled()
		expect(toastMock.error).toHaveBeenCalledWith(
			expect.stringMatching(/^Déplacement refusé/),
			{ duration: 6000 },
		)
	})

	it('ignores a drop outside the list or on itself', () => {
		const { result } = renderHook(() => useSurveyFormEditor(FORM, vi.fn()))
		act(() => result.current.onDragEnd(drop('b', null)))
		act(() => result.current.onDragEnd(drop('b', 'b')))
		expect(result.current.form).toBe(FORM)
		expect(toastMock.error).not.toHaveBeenCalled()
	})
})

const BLOCKY: SurveyForm = {
	endId: 'end',
	endMessage: '',
	questions: [
		{ ...newQuestion('a', 'SINGLE_CHOICE'), options: ['Oui', 'Non'] },
		{ ...newQuestion('b', 'TEXT'), block: 'B' },
		{ ...newQuestion('c', 'TEXT'), block: 'B' },
		newQuestion('d', 'TEXT'),
	],
}

function shape(form: SurveyForm): string[] {
	return form.questions.map((item) =>
		item.block ? `${item.id}:${item.block}` : item.id,
	)
}

describe('useSurveyFormEditor blocks', () => {
	it('adds a block of two empty questions at the end', () => {
		const { result } = renderHook(() => useSurveyFormEditor(FORM, vi.fn()))
		act(() => result.current.insertBlock())
		const added = result.current.form.questions.slice(3)
		expect(added).toHaveLength(2)
		expect(added[0].block).not.toBeNull()
		expect(added[0].block).toBe(added[1].block)
		expect(result.current.announcement).toMatch(/Bloc de 2 questions ajouté/)
	})

	it('groups a question with the previous one and takes it back out', () => {
		const { result } = renderHook(() => useSurveyFormEditor(BLOCKY, vi.fn()))
		act(() => result.current.onQuestionAction('d', { type: 'group' }))
		expect(shape(result.current.form)).toEqual(['a', 'b:B', 'c:B', 'd:B'])
		expect(result.current.announcement).toMatch(/bloc de 3 questions/)
		act(() => result.current.onQuestionAction('c', { type: 'ungroup' }))
		expect(shape(result.current.form)).toEqual(['a', 'b:B', 'd:B', 'c'])
	})

	it('dissolves a block that falls to one question and says so', () => {
		const { result } = renderHook(() => useSurveyFormEditor(BLOCKY, vi.fn()))
		act(() => result.current.onQuestionAction('b', { type: 'ungroup' }))
		expect(shape(result.current.form)).toEqual(['a', 'b', 'c', 'd'])
		expect(result.current.announcement).toMatch(/Bloc dissous/)
		const { result: deleted } = renderHook(() =>
			useSurveyFormEditor(BLOCKY, vi.fn()),
		)
		act(() => deleted.current.onQuestionAction('c', { type: 'delete' }))
		expect(shape(deleted.current.form)).toEqual(['a', 'b', 'd'])
	})

	it('refuses to group the first question with a toast', () => {
		const { result } = renderHook(() => useSurveyFormEditor(BLOCKY, vi.fn()))
		act(() => result.current.onQuestionAction('a', { type: 'group' }))
		expect(result.current.form).toBe(BLOCKY)
		expect(toastMock.error).toHaveBeenCalled()
	})

	it('moves a whole block and adds a question into it', () => {
		const { result } = renderHook(() => useSurveyFormEditor(BLOCKY, vi.fn()))
		act(() => result.current.onBlockAction('B', { type: 'move', delta: 1 }))
		expect(shape(result.current.form)).toEqual(['a', 'd', 'b:B', 'c:B'])
		expect(result.current.announcement).toMatch(/Bloc déplacé/)
		act(() => result.current.onBlockAction('B', { type: 'add', kind: 'DATE' }))
		expect(result.current.form.questions).toHaveLength(5)
		expect(result.current.form.questions[4]).toMatchObject({
			kind: 'DATE',
			block: 'B',
		})
	})

	it('drags a block as a unit and refuses to drag a member out', () => {
		const { result } = renderHook(() => useSurveyFormEditor(BLOCKY, vi.fn()))
		act(() => result.current.onDragEnd(drop('c', 'd')))
		expect(result.current.form).toBe(BLOCKY)
		expect(toastMock.error).toHaveBeenCalledWith(
			expect.stringMatching(/Sortir du bloc/),
			{ duration: 6000 },
		)
		act(() => result.current.onDragEnd(drop('B', 'a')))
		expect(shape(result.current.form)).toEqual(['b:B', 'c:B', 'a', 'd'])
	})

	it('sets the rule of a block on its first member', () => {
		const { result } = renderHook(() => useSurveyFormEditor(BLOCKY, vi.fn()))
		act(() => result.current.onQuestionAction('b', { type: 'add-condition' }))
		expect(result.current.form.questions[1].rule?.conditions).toEqual([
			{ question: 'a', option: 'Oui', checked: true },
		])
		expect(result.current.form.questions[2].rule).toBeNull()
	})
})
