import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { renderHook, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { TanstackQueryApiClient } from '#/api/api.tanstack'
import {
	useChannelPermissions,
	useChannelWebhooks,
	useCreateChannel,
	useCreateWebhook,
	useDeleteWebhook,
	useUpsertEveryoneOverwrite,
} from '#/hooks/use-chat'

function envelope<T>(data: T) {
	return { data, pagination: null }
}

let apiGet: ReturnType<typeof vi.fn>
let apiPost: ReturnType<typeof vi.fn>
let apiPut: ReturnType<typeof vi.fn>
let apiDelete: ReturnType<typeof vi.fn>

function wrapper({ children }: { children: ReactNode }) {
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false } },
	})
	return (
		<QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
	)
}

beforeEach(() => {
	apiGet = vi.fn().mockResolvedValue(envelope([]))
	apiPost = vi.fn().mockResolvedValue(envelope({ id: 'new-1' }))
	apiPut = vi.fn().mockResolvedValue(
		envelope({
			target_type: 'everyone',
			target_id: null,
			allow: 32,
			deny: 0,
			created_at: '',
			updated_at: '',
		}),
	)
	apiDelete = vi.fn().mockResolvedValue({ status: 204, data: undefined })
	window.tanstackApi = new TanstackQueryApiClient({
		get: apiGet,
		post: apiPost,
		put: apiPut,
		delete: apiDelete,
	} as never)
})

describe('useChannelPermissions', () => {
	it('unwraps the DataEnvelope', async () => {
		apiGet.mockResolvedValueOnce(
			envelope([
				{
					target_type: 'everyone',
					target_id: null,
					allow: 32,
					deny: 0,
					created_at: '',
					updated_at: '',
				},
			]),
		)

		const { result } = renderHook(() => useChannelPermissions('ch-1'), {
			wrapper,
		})

		await waitFor(() => expect(result.current.data).toHaveLength(1))
		expect(result.current.data?.[0]?.target_type).toBe('everyone')
	})
})

describe('useChannelWebhooks', () => {
	it('unwraps the DataEnvelope', async () => {
		apiGet.mockResolvedValueOnce(
			envelope([
				{
					id: 'wh-1',
					organization_id: 'org-1',
					channel_id: 'ch-1',
					name: 'CI bot',
					avatar_url: null,
					created_by: 'me',
					created_at: '',
					updated_at: '',
				},
			]),
		)

		const { result } = renderHook(() => useChannelWebhooks('ch-1'), {
			wrapper,
		})

		await waitFor(() => expect(result.current.data).toHaveLength(1))
		expect(result.current.data?.[0]?.name).toBe('CI bot')
	})
})

describe('admin mutations', () => {
	it('useCreateChannel posts to the organization channels endpoint', async () => {
		const { result } = renderHook(() => useCreateChannel('org-1'), {
			wrapper,
		})

		result.current.mutate({
			path: { organization_id: 'org-1' },
			body: { name: 'general', position: 0 },
		} as never)

		await waitFor(() => expect(result.current.isSuccess).toBe(true))
		expect(apiPost).toHaveBeenCalledWith(
			'/api/v1/chat/organizations/{organization_id}/channels',
			expect.objectContaining({
				path: { organization_id: 'org-1' },
				body: { name: 'general', position: 0 },
			}),
		)
	})

	it('useUpsertEveryoneOverwrite puts allow/deny for the everyone target', async () => {
		const { result } = renderHook(() => useUpsertEveryoneOverwrite('ch-1'), {
			wrapper,
		})

		result.current.mutate({
			path: { channel_id: 'ch-1' },
			body: { allow: 32, deny: 0 },
		} as never)

		await waitFor(() => expect(result.current.isSuccess).toBe(true))
		expect(apiPut).toHaveBeenCalledWith(
			'/api/v1/chat/channels/{channel_id}/permissions/everyone',
			expect.objectContaining({
				path: { channel_id: 'ch-1' },
				body: { allow: 32, deny: 0 },
			}),
		)
	})

	it('useCreateWebhook and useDeleteWebhook call the right endpoints', async () => {
		const { result: createResult } = renderHook(
			() => useCreateWebhook('ch-1'),
			{ wrapper },
		)
		createResult.current.mutate({
			path: { channel_id: 'ch-1' },
			body: { name: 'CI bot' },
		} as never)
		await waitFor(() => expect(createResult.current.isSuccess).toBe(true))
		expect(apiPost).toHaveBeenCalledWith(
			'/api/v1/chat/channels/{channel_id}/webhooks',
			expect.objectContaining({ body: { name: 'CI bot' } }),
		)

		const { result: deleteResult } = renderHook(
			() => useDeleteWebhook('ch-1'),
			{ wrapper },
		)
		deleteResult.current.mutate({
			path: { webhook_id: 'wh-1' },
		} as never)
		await waitFor(() => expect(deleteResult.current.isSuccess).toBe(true))
		expect(apiDelete).toHaveBeenCalledWith(
			'/api/v1/chat/webhooks/{webhook_id}',
			expect.objectContaining({ path: { webhook_id: 'wh-1' } }),
		)
	})
})
