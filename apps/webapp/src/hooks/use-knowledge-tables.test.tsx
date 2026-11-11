import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, renderHook } from '@testing-library/react'
import type { ReactNode } from 'react'
import { describe, expect, it, vi } from 'vitest'
import { useUpdateTableRow } from '#/hooks/use-knowledge-tables'

const ORG = 'org'
const TABLE = 'table'
const KEY = ['knowledge-tables', ORG, 'rows', TABLE, 'view'] as const

function seeded(): QueryClient {
	const client = new QueryClient({
		defaultOptions: { mutations: { retry: false } },
	})
	client.setQueryData(KEY, {
		pageParams: [0],
		pages: [
			{
				rows: [{ id: 'r1', values: { a: 'old', b: 'old' } }],
				total: 1,
				limit: 100,
				offset: 0,
			},
		],
	})
	return client
}

function cached(client: QueryClient): Record<string, unknown> {
	const data = client.getQueryData<{
		pages: { rows: { values: Record<string, unknown> }[] }[]
	}>(KEY)
	return data?.pages[0].rows[0].values ?? {}
}

describe('useUpdateTableRow', () => {
	it('restores only the failed cell, not a later successful edit', async () => {
		const client = seeded()
		let reject: (error: Error) => void = () => {}
		const patch = vi
			.fn()
			.mockImplementationOnce(() => new Promise((_, fail) => (reject = fail)))
			.mockResolvedValueOnce({
				data: { id: 'r1', values: { a: 'x', b: 'y' } },
			})
		window.api = { patch } as never
		const wrapper = ({ children }: { children: ReactNode }) => (
			<QueryClientProvider client={client}>{children}</QueryClientProvider>
		)
		const { result } = renderHook(() => useUpdateTableRow(ORG, TABLE), {
			wrapper,
		})

		let first: Promise<unknown> = Promise.resolve()
		await act(async () => {
			first = result.current
				.mutateAsync({ rowId: 'r1', values: { a: 'x' } })
				.catch(() => undefined)
		})
		await act(async () => {
			await result.current.mutateAsync({ rowId: 'r1', values: { b: 'y' } })
		})
		await act(async () => {
			reject(new Error('refused'))
			await first
		})

		expect(cached(client)).toEqual({ a: 'old', b: 'y' })
	})
})
