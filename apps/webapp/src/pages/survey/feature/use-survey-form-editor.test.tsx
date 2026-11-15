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
