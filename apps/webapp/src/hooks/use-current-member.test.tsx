import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { renderHook, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { TanstackQueryApiClient } from '#/api/api.tanstack'
import { useCurrentMember } from '#/hooks/use-current-member'

function wrapper({ children }: { children: ReactNode }) {
	const queryClient = new QueryClient()
	return (
		<QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
	)
}

let get: ReturnType<typeof vi.fn>

beforeEach(() => {
	get = vi.fn()
	window.tanstackApi = new TanstackQueryApiClient({ get } as never)
})

describe('useCurrentMember', () => {
	it('spec_rh_7_6 reads the caller seat from members/me', async () => {
		get.mockResolvedValue({
			data: {
				member_id: 'm-1',
				organization_id: 'org-1',
				display_name: 'Nova',
			},
			pagination: null,
		})
		const { result } = renderHook(() => useCurrentMember('org-1'), { wrapper })
		await waitFor(() => expect(result.current.isSuccess).toBe(true))
		expect(result.current.data?.data.member_id).toBe('m-1')
		expect(get.mock.calls[0][0]).toBe(
			'/api/v1/organizations/{organization_id}/members/me',
		)
	})

	it('spec_rh_7_6 does not retry a caller who holds no seat', async () => {
		get.mockRejectedValue(
			Object.assign(new Error('Not found'), { status: 404 }),
		)
		const { result } = renderHook(() => useCurrentMember('org-1'), { wrapper })
		await waitFor(() => expect(result.current.isError).toBe(true))
		expect(get).toHaveBeenCalledTimes(1)
	})
})
