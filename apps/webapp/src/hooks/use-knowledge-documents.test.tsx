import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, renderHook } from '@testing-library/react'
import type { ReactNode } from 'react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { useDocumentSession } from '#/pages/knowledge-documents/feature/use-document-session'
import type { DocumentContent } from '#/pages/knowledge-documents/lib/document-api-types'

const BASE = '/api/v1/organizations/org-1/knowledge/articles/doc-1'
const CONTENT: DocumentContent = {
	version: 1,
	blocks: [
		{
			id: 'a',
			type: 'paragraph',
			props: {},
			content: [{ type: 'text', text: 'Bonjour', styles: {} }],
			children: [],
		},
	],
}

interface Call {
	method: string
	url: string
	body: unknown
	keepalive: boolean | undefined
}

let calls: Call[]
let respond: (call: Call) => Response

function json(status: number, body: unknown): Response {
	return new Response(JSON.stringify(body), {
		status,
		headers: { 'Content-Type': 'application/json' },
	})
}

function wrapper({ children }: { children: ReactNode }) {
	return (
		<QueryClientProvider client={new QueryClient()}>
			{children}
		</QueryClientProvider>
	)
}

function session(canEdit = true) {
	return renderHook(
		() =>
			useDocumentSession({
				organizationId: 'org-1',
				articleId: 'doc-1',
				canEdit,
				loadedRevision: 0,
			}),
		{ wrapper },
	)
}

beforeEach(() => {
	vi.useFakeTimers()
	calls = []
	window.apiUrl = 'http://api.test'
	respond = (call) => {
		if (call.method === 'POST' && call.url.endsWith('/lock')) {
			return json(200, {
				holder_id: 'me',
				expires_at: '2026-11-11T10:01:30Z',
				revision: 0,
			})
		}
		if (call.method === 'PUT') {
			const sent = call.body as { revision: number }
			return json(200, {
				revision: sent.revision + 1,
				updated_at: '2026-11-11T10:00:00Z',
			})
		}
		return new Response(null, { status: 204 })
	}
	vi.stubGlobal(
		'fetch',
		vi.fn(async (url: string, init: RequestInit) => {
			const call: Call = {
				method: String(init.method),
				url: String(url).replace('http://api.test', ''),
				body: typeof init.body === 'string' ? JSON.parse(init.body) : null,
				keepalive: init.keepalive,
			}
			calls.push(call)
			return respond(call)
		}),
	)
})

afterEach(() => {
	vi.useRealTimers()
	vi.unstubAllGlobals()
})

describe('useDocumentSession against the API', () => {
	it('takes the lock on open and renews it every 30 seconds', async () => {
		const { result } = session()
		await act(async () => {
			await vi.advanceTimersByTimeAsync(0)
		})
		expect(result.current.state.phase).toBe('editing')
		expect(calls.map((call) => `${call.method} ${call.url}`)).toEqual([
			`POST ${BASE}/lock`,
		])
		await act(async () => {
			await vi.advanceTimersByTimeAsync(30_000)
		})
		await act(async () => {
			await vi.advanceTimersByTimeAsync(30_000)
		})
		expect(calls.filter((call) => call.method === 'POST')).toHaveLength(3)
	})

	it('saves the Mestier format with the revision, one second after the change', async () => {
		const { result } = session()
		await act(async () => {
			await vi.advanceTimersByTimeAsync(0)
		})
		act(() => result.current.change(() => CONTENT))
		await act(async () => {
			await vi.advanceTimersByTimeAsync(1000)
		})
		const put = calls.find((call) => call.method === 'PUT')
		expect(put?.url).toBe(`${BASE}/content`)
		expect(put?.body).toEqual({ content: CONTENT, revision: 0 })
		expect(result.current.state.saveState).toBe('saved')
	})

	it('goes read-only when the server says the lock was lost', async () => {
		const { result } = session()
		await act(async () => {
			await vi.advanceTimersByTimeAsync(0)
		})
		respond = () => json(409, { reason: 'lock_lost' })
		act(() => result.current.change(() => CONTENT))
		await act(async () => {
			await vi.advanceTimersByTimeAsync(1000)
		})
		expect(result.current.state.phase).toBe('conflict')
		expect(result.current.state.conflict).toBe('lock_lost')
	})

	it('reads the holder from a 409 on the lock', async () => {
		respond = () =>
			json(409, { holder_id: 'user-9', expires_at: '2026-11-11T10:01:30Z' })
		const { result } = session()
		await act(async () => {
			await vi.advanceTimersByTimeAsync(0)
		})
		expect(result.current.state.phase).toBe('locked')
		expect(result.current.state.holderId).toBe('user-9')
	})

	it('releases the lock when the page is left', async () => {
		const { unmount } = session()
		await act(async () => {
			await vi.advanceTimersByTimeAsync(0)
		})
		unmount()
		await act(async () => {
			await vi.advanceTimersByTimeAsync(0)
		})
		expect(calls.at(-1)).toMatchObject({
			method: 'DELETE',
			url: `${BASE}/lock`,
		})
	})

	it('releases the lock with a keepalive request on pagehide', async () => {
		session()
		await act(async () => {
			await vi.advanceTimersByTimeAsync(0)
		})
		act(() => {
			window.dispatchEvent(new Event('pagehide'))
		})
		expect(calls.at(-1)).toMatchObject({ method: 'DELETE', keepalive: true })
	})

	it('does nothing on the lock without the right to edit', async () => {
		const { result } = session(false)
		await act(async () => {
			await vi.advanceTimersByTimeAsync(0)
		})
		expect(result.current.state.phase).toBe('viewing')
		expect(calls).toEqual([])
	})
})
