import { act, renderHook } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { useDebouncedSearch } from '#/pages/cursus/shared/feature/use-debounced-search'

beforeEach(() => {
	vi.useFakeTimers()
})

afterEach(() => {
	vi.useRealTimers()
})

describe('useDebouncedSearch', () => {
	it('echoes the text at once and commits the trimmed query after the delay', () => {
		const onCommit = vi.fn()
		const { result } = renderHook(() => useDebouncedSearch('', onCommit))

		act(() => result.current.setText(' CAP '))

		expect(result.current.text).toBe(' CAP ')
		expect(result.current.query).toBe('')
		act(() => vi.advanceTimersByTime(300))
		expect(result.current.query).toBe('CAP')
		expect(onCommit).toHaveBeenCalledExactlyOnceWith('CAP')
	})

	it('only commits the last text of a burst', () => {
		const onCommit = vi.fn()
		const { result } = renderHook(() => useDebouncedSearch('', onCommit))

		act(() => result.current.setText('C'))
		act(() => vi.advanceTimersByTime(100))
		act(() => result.current.setText('CA'))
		act(() => vi.advanceTimersByTime(300))

		expect(onCommit).toHaveBeenCalledExactlyOnceWith('CA')
	})

	it('resets text and query at once', () => {
		const onCommit = vi.fn()
		const { result } = renderHook(() => useDebouncedSearch('CAP', onCommit))

		act(() => result.current.reset())

		expect(result.current.text).toBe('')
		expect(result.current.query).toBe('')
	})
})
