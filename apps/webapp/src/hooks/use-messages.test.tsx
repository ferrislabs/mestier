import { act, renderHook, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useMessages } from '#/hooks/use-chat'
import { GatewayProvider } from '#/hooks/use-gateway'
import { clearAuth } from '#/store/auth.store'

class FakeWebSocket {
	onopen: (() => void) | null = null
	onmessage: ((event: { data: string }) => void) | null = null
	onclose: (() => void) | null = null
	onerror: (() => void) | null = null
	send() {}
	close() {}
}

function wrapper({ children }: { children: ReactNode }) {
	return <GatewayProvider>{children}</GatewayProvider>
}

/** Every REST response through `window.api` is wrapped in `{ data, pagination }`
 * (see `use-chat.ts`'s comment on `fetchMessagesPage`) — the mocks below
 * must resolve to that same shape, not the bare resource. */
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

let apiGet: ReturnType<typeof vi.fn>
let apiPost: ReturnType<typeof vi.fn>
let apiPatch: ReturnType<typeof vi.fn>
let apiDelete: ReturnType<typeof vi.fn>
let apiPut: ReturnType<typeof vi.fn>

beforeEach(() => {
	vi.stubGlobal('WebSocket', FakeWebSocket)
	clearAuth()
	apiGet = vi.fn().mockResolvedValue(envelope([]))
	apiPost = vi.fn().mockResolvedValue(envelope(message()))
	apiPatch = vi.fn().mockResolvedValue(envelope(message()))
	apiDelete = vi.fn().mockResolvedValue(undefined)
	apiPut = vi.fn().mockResolvedValue(undefined)
	window.api = {
		get: apiGet,
		post: apiPost,
		patch: apiPatch,
		delete: apiDelete,
		put: apiPut,
	} as never
})

describe('useMessages — initial load', () => {
	it('loads the first page and exposes it oldest-first', async () => {
		apiGet.mockResolvedValueOnce(
			envelope([message({ id: 'm-2' }), message({ id: 'm-1' })]),
		)

		const { result } = renderHook(() => useMessages('ch-1', 'me'), {
			wrapper,
		})

		await waitFor(() => expect(result.current.isLoadingInitial).toBe(false))

		expect(result.current.messages.map((entry) => entry.message.id)).toEqual([
			'm-1',
			'm-2',
		])
		expect(apiGet).toHaveBeenCalledWith(
			'/api/v1/chat/channels/{channel_id}/messages',
			expect.objectContaining({
				path: { channel_id: 'ch-1' },
				query: expect.objectContaining({ limit: 50 }),
			}),
		)
	})

	it('surfaces a load error instead of hanging', async () => {
		apiGet.mockRejectedValueOnce(new Error('network'))

		const { result } = renderHook(() => useMessages('ch-1', 'me'), {
			wrapper,
		})

		await waitFor(() => expect(result.current.isLoadingInitial).toBe(false))
		expect(result.current.initialError).not.toBeNull()
	})
})

describe('useMessages — sending', () => {
	it('shows the message immediately, then reconciles with the REST response', async () => {
		let resolvePost!: (value: unknown) => void
		apiPost.mockReturnValueOnce(
			new Promise((resolve) => {
				resolvePost = resolve
			}),
		)

		const { result } = renderHook(() => useMessages('ch-1', 'me'), {
			wrapper,
		})
		await waitFor(() => expect(result.current.isLoadingInitial).toBe(false))

		act(() => result.current.sendMessage('hey there'))

		expect(result.current.messages).toHaveLength(1)
		expect(result.current.messages[0]?.status).toBe('sending')

		await act(async () => {
			resolvePost(envelope(message({ id: 'm-real', content: 'hey there' })))
			await Promise.resolve()
		})

		await waitFor(() => {
			expect(result.current.messages[0]?.status).toBe('sent')
			expect(result.current.messages[0]?.message.id).toBe('m-real')
		})
	})

	it('marks a failed send as failed and retry resends the same content', async () => {
		apiPost.mockRejectedValueOnce(new Error('network'))

		const { result } = renderHook(() => useMessages('ch-1', 'me'), {
			wrapper,
		})
		await waitFor(() => expect(result.current.isLoadingInitial).toBe(false))

		act(() => result.current.sendMessage('will fail'))
		await waitFor(() =>
			expect(result.current.messages[0]?.status).toBe('failed'),
		)

		apiPost.mockResolvedValueOnce(
			envelope(message({ id: 'm-real', content: 'will fail' })),
		)
		const tempId = result.current.messages[0]?.tempId
		expect(tempId).toBeDefined()
		act(() => result.current.retrySend(tempId as string))

		await waitFor(() => expect(result.current.messages[0]?.status).toBe('sent'))
		expect(apiPost).toHaveBeenCalledTimes(2)
	})
})

describe('useMessages — editing and deleting', () => {
	it('applies an edit locally once the server confirms it', async () => {
		apiGet.mockResolvedValueOnce(
			envelope([message({ id: 'm-1', content: 'before' })]),
		)
		apiPatch.mockResolvedValueOnce(
			envelope(message({ id: 'm-1', content: 'after' })),
		)

		const { result } = renderHook(() => useMessages('ch-1', 'me'), {
			wrapper,
		})
		await waitFor(() => expect(result.current.isLoadingInitial).toBe(false))

		await act(async () => {
			await result.current.editMessage('m-1', 'after')
		})

		expect(result.current.messages[0]?.message.content).toBe('after')
		expect(apiPatch).toHaveBeenCalledWith(
			'/api/v1/chat/messages/{message_id}',
			expect.objectContaining({
				path: { message_id: 'm-1' },
				body: { content: 'after' },
			}),
		)
	})

	it('removes a message once the server confirms the delete', async () => {
		apiGet.mockResolvedValueOnce(envelope([message({ id: 'm-1' })]))

		const { result } = renderHook(() => useMessages('ch-1', 'me'), {
			wrapper,
		})
		await waitFor(() => expect(result.current.isLoadingInitial).toBe(false))

		await act(async () => {
			await result.current.deleteMessage('m-1')
		})

		expect(result.current.messages).toHaveLength(0)
	})
})

describe('useMessages — reactions', () => {
	it('adds a reaction optimistically and confirms it via PUT', async () => {
		apiGet.mockResolvedValueOnce(
			envelope([message({ id: 'm-1', reactions: [] })]),
		)

		const { result } = renderHook(() => useMessages('ch-1', 'me'), {
			wrapper,
		})
		await waitFor(() => expect(result.current.isLoadingInitial).toBe(false))

		act(() => result.current.toggleReaction('m-1', '👍', false))

		expect(result.current.messages[0]?.message.reactions).toEqual([
			{ emoji: '👍', count: 1, user_ids: ['me'] },
		])
		expect(apiPut).toHaveBeenCalledWith(
			'/api/v1/chat/messages/{message_id}/reactions/{emoji}',
			expect.objectContaining({
				path: { message_id: 'm-1', emoji: '👍' },
			}),
		)
	})

	it('reverts an optimistic reaction when the request fails', async () => {
		apiGet.mockResolvedValueOnce(
			envelope([message({ id: 'm-1', reactions: [] })]),
		)
		apiPut.mockRejectedValueOnce(new Error('network'))

		const { result } = renderHook(() => useMessages('ch-1', 'me'), {
			wrapper,
		})
		await waitFor(() => expect(result.current.isLoadingInitial).toBe(false))

		act(() => result.current.toggleReaction('m-1', '👍', false))
		expect(result.current.messages[0]?.message.reactions).toHaveLength(1)

		await waitFor(() =>
			expect(result.current.messages[0]?.message.reactions).toEqual([]),
		)
	})
})
