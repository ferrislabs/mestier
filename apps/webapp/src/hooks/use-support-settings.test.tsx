import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, renderHook, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { describe, expect, it } from 'vitest'
import { useUpdateSupportSettings } from '#/hooks/use-support-settings'

const SETTINGS_PATH = '/api/v1/organizations/{organization_id}/support/settings'

interface RecordedCall {
	method: string
	path: string
	params: unknown
}

function installFakeApi(respond: (params: unknown) => unknown) {
	const calls: RecordedCall[] = []

	const fakeApi = {
		get(path: string, params: unknown) {
			const queryKey = [
				{ _id: path, path: (params as { path?: unknown } | undefined)?.path },
			]
			return { queryKey, queryOptions: { queryKey } }
		},
		mutation(method: string, path: string) {
			const mutationKey = [{ method, path }]
			return {
				mutationKey,
				mutationOptions: {
					mutationKey,
					mutationFn: async (params: unknown) => {
						calls.push({ method, path, params })
						return respond(params)
					},
				},
			}
		},
	}

	Object.assign(window, { tanstackApi: fakeApi })

	return calls
}

function settingsKey(organizationId: string) {
	return [{ _id: SETTINGS_PATH, path: { organization_id: organizationId } }]
}

function renderUpdateHook(organizationId: string) {
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	queryClient.setQueryData(settingsKey('org-1'), {
		data: { display_fields: ['chantier'] },
	})
	queryClient.setQueryData(settingsKey('org-2'), {
		data: { display_fields: ['marque'] },
	})

	function wrapper({ children }: { children: ReactNode }) {
		return (
			<QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
		)
	}

	const hook = renderHook(() => useUpdateSupportSettings(organizationId), {
		wrapper,
	})
	return { ...hook, queryClient }
}

describe('useUpdateSupportSettings', () => {
	it('sends PUT on the settings path with the raw client params', async () => {
		const calls = installFakeApi((params) => ({
			data: (params as { body: unknown }).body,
		}))
		const { result } = renderUpdateHook('org-1')

		await act(async () => {
			await result.current.mutateAsync({
				path: { organization_id: 'org-1' },
				body: { display_fields: ['chantier', 'marque'] },
			})
		})

		expect(calls).toEqual([
			{
				method: 'put',
				path: SETTINGS_PATH,
				params: {
					path: { organization_id: 'org-1' },
					body: { display_fields: ['chantier', 'marque'] },
				},
			},
		])
	})

	it('invalidates the settings query of that organization once the request succeeds', async () => {
		installFakeApi((params) => ({ data: (params as { body: unknown }).body }))
		const { result, queryClient } = renderUpdateHook('org-1')

		expect(queryClient.getQueryState(settingsKey('org-1'))?.isInvalidated).toBe(
			false,
		)

		await act(async () => {
			await result.current.mutateAsync({
				path: { organization_id: 'org-1' },
				body: { display_fields: ['chantier'] },
			})
		})

		await waitFor(() => {
			expect(
				queryClient.getQueryState(settingsKey('org-1'))?.isInvalidated,
			).toBe(true)
		})
		expect(queryClient.getQueryState(settingsKey('org-2'))?.isInvalidated).toBe(
			false,
		)
	})

	it('invalidates the settings query even when the request is rejected', async () => {
		installFakeApi(() => {
			throw new Error('La liste est invalide.')
		})
		const { result, queryClient } = renderUpdateHook('org-1')

		await act(async () => {
			await result.current
				.mutateAsync({
					path: { organization_id: 'org-1' },
					body: { display_fields: ['chantier'] },
				})
				.catch(() => undefined)
		})

		await waitFor(() => {
			expect(result.current.error?.message).toBe('La liste est invalide.')
			expect(
				queryClient.getQueryState(settingsKey('org-1'))?.isInvalidated,
			).toBe(true)
		})
	})
})
