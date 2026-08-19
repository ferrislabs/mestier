import { renderHook } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { useCatalogItems } from '#/hooks/use-catalog-items'

/**
 * The catalogue is derived from two queries, and callers reach for it before
 * those resolve. What matters then is that the derived array keeps its identity
 * across renders: the quote edit screen has an effect keyed on it that calls
 * `setValues`, so a fresh array on every render is an infinite render loop.
 */
describe('useCatalogItems', () => {
	it('keeps one identity while the catalogue has not loaded', () => {
		const { result, rerender } = renderHook(() =>
			useCatalogItems(undefined, undefined),
		)
		const first = result.current

		rerender()
		rerender()

		expect(result.current).toBe(first)
	})

	/**
	 * The loaded side comes from the query cache, so it keeps its reference.
	 * Hoisted here for the same reason: an inline `[]` would be a new array per
	 * render, which is the caller mistake this hook now absorbs rather than a
	 * property of the hook itself.
	 */
	it('keeps one identity while only one of the two has loaded', () => {
		const loaded: never[] = []
		const { result, rerender } = renderHook(() =>
			useCatalogItems(loaded, undefined),
		)
		const first = result.current

		rerender()

		expect(result.current).toBe(first)
	})

	it('still builds the items once both have loaded', () => {
		const serviceRates = [
			{
				id: 'sr-1',
				organization_id: 'org-1',
				label: 'Taille de haie',
				unit: 'HOUR' as const,
				rate_cents: 4500,
				created_at: '2026-01-01T00:00:00Z',
				updated_at: '2026-01-01T00:00:00Z',
			},
		]
		const { result } = renderHook(() => useCatalogItems(serviceRates, []))

		expect(result.current).toHaveLength(1)
		expect(result.current[0]?.sourceId).toBe('sr-1')
	})
})
