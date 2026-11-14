import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render } from '@testing-library/react'
import type { ReactNode } from 'react'
import { vi } from 'vitest'

export interface RecordedCall {
	method: string
	url: string
	headers: Headers
	credentials: RequestCredentials | undefined
	body: unknown
}

export interface Reply {
	status: number
	body?: unknown
}

type Handler = (call: RecordedCall) => Reply

export function data(value: unknown, status = 200): Reply {
	return { status, body: { data: value, pagination: null } }
}

export function refusal(status: number, reason?: string): Reply {
	return {
		status,
		body: {
			code: 'E_REFUSED',
			status,
			message: 'internal message that must never show',
			...(reason ? { details: { reason } } : {}),
		},
	}
}

export function installServer(routes: Record<string, Handler>) {
	const calls: RecordedCall[] = []
	const fetchMock = vi.fn(
		async (input: RequestInfo | URL, init?: RequestInit) => {
			const method = (init?.method ?? 'GET').toUpperCase()
			const url = String(input)
			const call: RecordedCall = {
				method,
				url,
				headers: new Headers(init?.headers),
				credentials: init?.credentials,
				body:
					typeof init?.body === 'string' ? JSON.parse(init.body) : init?.body,
			}
			calls.push(call)
			const handler = routes[`${method} ${url}`]
			if (!handler) throw new TypeError(`unmocked ${method} ${url}`)
			const reply = handler(call)
			return new Response(
				reply.body === undefined ? null : JSON.stringify(reply.body),
				{
					status: reply.status,
					headers: { 'Content-Type': 'application/json' },
				},
			)
		},
	)
	vi.stubGlobal('fetch', fetchMock)
	window.apiUrl = ''
	return { calls, fetchMock }
}

export function renderWithQuery(ui: ReactNode) {
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	return render(
		<QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>,
	)
}
