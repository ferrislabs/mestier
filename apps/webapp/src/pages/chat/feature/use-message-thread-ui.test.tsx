import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, renderHook, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { TanstackQueryApiClient } from '#/api/api.tanstack'
import { GatewayProvider } from '#/hooks/use-gateway'
import { clearAuth, setAuth } from '#/store/auth.store'
import { useMessageThreadUi } from './use-message-thread-ui'

class FakeWebSocket {
	onopen: (() => void) | null = null
	onmessage: ((event: { data: string }) => void) | null = null
	onclose: (() => void) | null = null
	onerror: (() => void) | null = null
	send() {}
	close() {}
}

function envelope<T>(data: T) {
	return { data, pagination: null }
}

function message(overrides: Partial<Record<string, unknown>> = {}) {
	return {
		id: 'm-1',
		organization_id: 'org-1',
		channel_id: 'ch-1',
		author_type: 'USER',
		author_user_id: 'me',
		author_webhook_id: null,
		content: 'hello',
		components: null,
		mention_user_ids: [],
		mention_role_ids: [],
		mention_channel_ids: [],
		mention_everyone: false,
		reactions: [],
		attachments: [],
		edited_at: null,
		created_at: '2026-01-01T00:00:00Z',
		...overrides,
	}
}

/** Simulates the container being scrolled to the bottom — jsdom has no
 * layout, so `scrollHeight`/`clientHeight` default to 0 and must be forced. */
function markAtBottom(container: HTMLElement) {
	Object.defineProperty(container, 'scrollHeight', {
		configurable: true,
		value: 1000,
	})
	Object.defineProperty(container, 'clientHeight', {
		configurable: true,
		value: 500,
	})
	Object.defineProperty(container, 'scrollTop', {
		configurable: true,
		value: 500,
	})
}

let apiGet: ReturnType<typeof vi.fn>
let apiPut: ReturnType<typeof vi.fn>

function wrapper({ children }: { children: ReactNode }) {
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false } },
	})
	return (
		<QueryClientProvider client={queryClient}>
			<GatewayProvider>{children}</GatewayProvider>
		</QueryClientProvider>
	)
}

beforeEach(() => {
	vi.stubGlobal('WebSocket', FakeWebSocket)
	clearAuth()
	apiGet = vi
		.fn()
		.mockResolvedValue(
			envelope([message({ id: 'm-2' }), message({ id: 'm-1' })]),
		)
	apiPut = vi.fn().mockResolvedValue({ status: 204, data: undefined })
	window.api = { get: apiGet, put: vi.fn() } as never
	// `useMarkChannelRead` goes through `window.tanstackApi.mutation('put', ...)`,
	// which calls its *own* client — a different object from `window.api`
	// above (which backs the hand-written `useMessages` calls only).
	window.tanstackApi = new TanstackQueryApiClient({
		get: vi.fn().mockResolvedValue({ data: [] }),
		put: apiPut,
	} as never)
	act(() => {
		setAuth({ accessToken: 'tok', isAuthenticated: true })
	})
})

describe('useMessageThreadUi — read state', () => {
	it('does not mark the channel read from the initial auto-scroll alone', async () => {
		const { result } = renderHook(() => useMessageThreadUi('ch-1'), {
			wrapper,
		})
		await waitFor(() =>
			expect(result.current.messageListProps.isLoadingInitial).toBe(false),
		)

		const container = document.createElement('div')
		result.current.messageListProps.scrollContainerRef.current = container
		markAtBottom(container)

		// A plain `scroll` event — indistinguishable, on its own, from the
		// programmatic auto-scroll the component does on mount.
		act(() => {
			result.current.messageListProps.onScroll({
				currentTarget: container,
			} as never)
		})

		expect(
			apiPut.mock.calls.filter(
				(call) => call[0] === '/api/v1/chat/channels/{channel_id}/read',
			),
		).toHaveLength(0)
	})

	it('marks the channel read once the reader scrolls to the bottom themselves', async () => {
		const { result } = renderHook(() => useMessageThreadUi('ch-1'), {
			wrapper,
		})
		await waitFor(() =>
			expect(result.current.messageListProps.isLoadingInitial).toBe(false),
		)

		const container = document.createElement('div')
		markAtBottom(container)

		// `onWheel`/`onTouchMove` are real reader input, unlike `onScroll`
		// (which also fires for the mount-time programmatic auto-scroll).
		act(() => {
			result.current.messageListProps.onWheel({
				currentTarget: container,
			} as never)
		})

		await waitFor(() => {
			expect(apiPut).toHaveBeenCalledWith(
				'/api/v1/chat/channels/{channel_id}/read',
				expect.objectContaining({
					path: { channel_id: 'ch-1' },
					body: { message_id: 'm-2' },
				}),
			)
		})
	})
})
