import { act, renderHook } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { useConfirmation } from '#/pages/support/ui/use-confirmation'

describe('useConfirmation', () => {
	beforeEach(() => vi.useFakeTimers())
	afterEach(() => vi.useRealTimers())

	it('arms on the first press without acting', () => {
		const action = vi.fn()
		const { result } = renderHook(() => useConfirmation())

		act(() => result.current.press(action))

		expect(result.current.armed).toBe(true)
		expect(action).not.toHaveBeenCalled()
	})

	it('acts and disarms on the second press', () => {
		const action = vi.fn()
		const { result } = renderHook(() => useConfirmation())

		act(() => result.current.press(action))
		act(() => result.current.press(action))

		expect(action).toHaveBeenCalledTimes(1)
		expect(result.current.armed).toBe(false)
	})

	it('disarms by itself after the delay', () => {
		const { result } = renderHook(() => useConfirmation(1000))

		act(() => result.current.press(vi.fn()))
		act(() => vi.advanceTimersByTime(1000))

		expect(result.current.armed).toBe(false)
	})

	it('disarms on cancel', () => {
		const { result } = renderHook(() => useConfirmation())

		act(() => result.current.press(vi.fn()))
		act(() => result.current.cancel())

		expect(result.current.armed).toBe(false)
	})
})
