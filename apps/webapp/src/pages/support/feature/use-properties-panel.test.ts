import { act, renderHook } from '@testing-library/react'
import { beforeEach, describe, expect, it } from 'vitest'
import { usePropertiesPanel } from '#/pages/support/feature/use-properties-panel'

describe('usePropertiesPanel', () => {
	beforeEach(() => localStorage.clear())

	it('starts open', () => {
		const { result } = renderHook(() => usePropertiesPanel())

		expect(result.current[0]).toBe(true)
	})

	it('remembers a closed panel', () => {
		const { result, unmount } = renderHook(() => usePropertiesPanel())

		act(() => result.current[1]())
		unmount()

		expect(renderHook(() => usePropertiesPanel()).result.current[0]).toBe(false)
	})

	it('opens again on a second toggle', () => {
		const { result } = renderHook(() => usePropertiesPanel())

		act(() => result.current[1]())
		act(() => result.current[1]())

		expect(result.current[0]).toBe(true)
	})
})
