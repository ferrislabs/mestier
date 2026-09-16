import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { renderHook } from '@testing-library/react'
import type { ReactNode } from 'react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { useRunPolling } from '#/hooks/use-automation'

function installFakeApi(statusAtCall: (call: number) => string) {
	let calls = 0

	const fakeApi = {
		get(path: string, params: unknown) {
			const p = (params ?? {}) as { path?: unknown }
			const queryKey = [{ _id: path, path: p.path }]
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						calls += 1
						return {
							data: { id: 'run-1', status: statusAtCall(calls), steps: [] },
							pagination: null,
						}
					},
				},
			}
		},
		mutation() {
			throw new Error('unused in this test')
		},
	}

	// biome-ignore lint/suspicious/noExplicitAny: test-only fake, shape matches TanstackQueryApiClient's used surface
	;(window as any).tanstackApi = fakeApi

	return { getCalls: () => calls }
}

function renderPollingHook() {
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false } },
	})
	function wrapper({ children }: { children: ReactNode }) {
		return (
			<QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
		)
	}
	return renderHook(() => useRunPolling('org-1', 'run-1'), { wrapper })
}

describe('useRunPolling', () => {
	beforeEach(() => {
		vi.useFakeTimers()
	})

	afterEach(() => {
		vi.useRealTimers()
	})

	it('keeps polling GET /runs/{id} every two seconds while the run is not terminal', async () => {
		const { getCalls } = installFakeApi(() => 'running')
		renderPollingHook()

		await vi.advanceTimersByTimeAsync(0)
		expect(getCalls()).toBe(1)

		await vi.advanceTimersByTimeAsync(2000)
		expect(getCalls()).toBe(2)

		await vi.advanceTimersByTimeAsync(2000)
		expect(getCalls()).toBe(3)
	})

	it('stops polling once the run reaches a terminal status', async () => {
		const { getCalls } = installFakeApi((call) =>
			call === 1 ? 'running' : 'succeeded',
		)
		renderPollingHook()

		await vi.advanceTimersByTimeAsync(0)
		expect(getCalls()).toBe(1)

		await vi.advanceTimersByTimeAsync(2000)
		expect(getCalls()).toBe(2)

		await vi.advanceTimersByTimeAsync(10_000)
		expect(getCalls()).toBe(2)
	})

	it('never fetches when there is no run to follow', async () => {
		const { getCalls } = installFakeApi(() => 'running')
		const queryClient = new QueryClient({
			defaultOptions: { queries: { retry: false } },
		})
		function wrapper({ children }: { children: ReactNode }) {
			return (
				<QueryClientProvider client={queryClient}>
					{children}
				</QueryClientProvider>
			)
		}
		renderHook(() => useRunPolling('org-1', null), { wrapper })

		await vi.advanceTimersByTimeAsync(5000)
		expect(getCalls()).toBe(0)
	})
})
