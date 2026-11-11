import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { renderHook, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { useDriveImageUrl } from '#/hooks/use-knowledge-documents'

let created: string[]
let revoked: string[]

function wrapper({ children }: { children: ReactNode }) {
	return (
		<QueryClientProvider client={new QueryClient()}>
			{children}
		</QueryClientProvider>
	)
}

beforeEach(() => {
	created = []
	revoked = []
	window.apiUrl = 'http://api.test'
	vi.stubGlobal(
		'fetch',
		vi.fn(async () => new Response(new Blob(['png']), { status: 200 })),
	)
	URL.createObjectURL = vi.fn(() => {
		const url = `blob:test-${created.length}`
		created.push(url)
		return url
	})
	URL.revokeObjectURL = vi.fn((url: string) => {
		revoked.push(url)
	})
})

afterEach(() => {
	vi.unstubAllGlobals()
})

describe('useDriveImageUrl', () => {
	it('gives a blob url and revokes it on unmount', async () => {
		const { result, unmount } = renderHook(
			() => useDriveImageUrl('org-1', 'art-1'),
			{ wrapper },
		)
		await waitFor(() => expect(result.current.data).toBe('blob:test-0'))
		expect(revoked).toEqual([])
		unmount()
		expect(revoked).toEqual(['blob:test-0'])
	})

	it('revokes the previous url when the image changes', async () => {
		const { result, rerender } = renderHook(
			({ id }) => useDriveImageUrl('org-1', id),
			{ wrapper, initialProps: { id: 'art-1' } },
		)
		await waitFor(() => expect(result.current.data).toBe('blob:test-0'))
		rerender({ id: 'art-2' })
		await waitFor(() => expect(result.current.data).toBe('blob:test-1'))
		expect(revoked).toContain('blob:test-0')
	})
})
