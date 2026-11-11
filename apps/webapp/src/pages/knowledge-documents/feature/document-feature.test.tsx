import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const state = vi.hoisted(() => ({ canManage: true }))

vi.mock('@tanstack/react-router', () => ({
	useNavigate: () => vi.fn(),
}))
vi.mock('#/hooks/use-active-organization', () => ({
	useActiveOrganization: () => ({
		activeOrganization: { id: 'org-1', slug: 'acme' },
	}),
}))
vi.mock('#/hooks/use-permissions', () => ({
	useHasPermission: () => state.canManage,
}))
vi.mock('#/hooks/use-chat', () => ({
	useChatDirectory: () => ({
		data: [{ user_id: 'user-9', display_name: 'Marie Lambert' }],
	}),
}))
vi.mock('#/pages/knowledge/feature/use-resource-options', () => ({
	useResourceOptions: () => [],
	RESOURCE_TYPE_LABELS: {
		project: 'Projet',
		customer: 'Client',
		quote: 'Devis',
	},
	resourceLabel: () => '',
}))

import { DocumentFeature } from '#/pages/knowledge-documents/feature/document-feature'

const BASE = '/api/v1/organizations/org-1/knowledge'

interface Call {
	method: string
	url: string
}

let calls: Call[]
let lockResponse: () => Response
let article: Record<string, unknown>

function json(status: number, body: unknown): Response {
	return new Response(JSON.stringify(body), {
		status,
		headers: { 'Content-Type': 'application/json' },
	})
}

beforeEach(() => {
	calls = []
	state.canManage = true
	window.apiUrl = 'http://api.test'
	article = {
		id: 'doc-1',
		folder_id: null,
		kind: 'document',
		title: 'Conditions de paiement',
		body: 'Bonjour',
		file: null,
		tags: [],
		links: [],
		revision: 4,
		lock: null,
		content: {
			version: 1,
			blocks: [
				{
					id: 'a',
					type: 'paragraph',
					props: {},
					content: [{ type: 'text', text: 'Bonjour le monde', styles: {} }],
					children: [],
				},
			],
		},
		created_at: '2026-11-11T08:00:00Z',
		updated_at: '2026-11-11T08:00:00Z',
	}
	lockResponse = () =>
		json(200, {
			holder_id: 'me',
			expires_at: '2026-11-11T10:01:30Z',
			revision: 4,
		})
	vi.stubGlobal(
		'fetch',
		vi.fn(async (url: string, init: RequestInit) => {
			const path = String(url).replace('http://api.test', '')
			const method = String(init.method)
			calls.push({ method, url: path })
			if (path.endsWith('/lock')) {
				return method === 'POST'
					? lockResponse()
					: new Response(null, { status: 204 })
			}
			if (path === `${BASE}/folder-tree`) return json(200, [])
			if (path === `${BASE}/articles/doc-1`) return json(200, article)
			return json(404, { message: 'not found' })
		}),
	)
})

afterEach(() => {
	vi.unstubAllGlobals()
})

function renderPage() {
	render(
		<QueryClientProvider client={new QueryClient()}>
			<DocumentFeature articleId="doc-1" />
		</QueryClientProvider>,
	)
}

function editor(): Element | null {
	return document.querySelector('.bn-editor')
}

describe('DocumentFeature', () => {
	it('opens an unlocked document for editing and takes the lock', async () => {
		renderPage()
		expect(
			await screen.findByDisplayValue('Conditions de paiement'),
		).toBeTruthy()
		await waitFor(() =>
			expect(editor()?.getAttribute('contenteditable')).toBe('true'),
		)
		expect(screen.getByText('Bonjour le monde')).toBeTruthy()
		expect(
			calls.filter(
				(call) => call.method === 'POST' && call.url.endsWith('/lock'),
			),
		).toHaveLength(1)
		expect(screen.queryByRole('button', { name: 'Recharger' })).toBeNull()
	})

	it('opens read-only with the holder named when someone else is editing', async () => {
		lockResponse = () =>
			json(409, { holder_id: 'user-9', expires_at: '2026-11-11T10:01:30Z' })
		renderPage()
		expect(
			await screen.findByText('En cours de modification par Marie Lambert'),
		).toBeTruthy()
		expect(editor()?.getAttribute('contenteditable')).toBe('false')
		expect(screen.getByRole('button', { name: 'Recharger' })).toBeTruthy()
	})

	it('opens read-only without asking for the lock without the right to manage', async () => {
		state.canManage = false
		renderPage()
		await screen.findByText('Bonjour le monde')
		expect(editor()?.getAttribute('contenteditable')).toBe('false')
		expect(calls.some((call) => call.url.endsWith('/lock'))).toBe(false)
		expect(
			screen.queryByRole('button', { name: 'Supprimer le document' }),
		).toBeNull()
	})

	it('goes read-only with a reload button when the document moved on before the lock', async () => {
		lockResponse = () =>
			json(200, {
				holder_id: 'me',
				expires_at: '2026-11-11T10:01:30Z',
				revision: 5,
			})
		renderPage()
		expect(
			await screen.findByText('Ce document a été modifié ailleurs'),
		).toBeTruthy()
		expect(editor()?.getAttribute('contenteditable')).toBe('false')
		expect(screen.getByRole('button', { name: 'Recharger' })).toBeTruthy()
	})

	it('says so when the element is not a document', async () => {
		article = { ...article, kind: 'file', content: undefined }
		renderPage()
		expect(
			await screen.findByText('Cet élément n’est pas un document.'),
		).toBeTruthy()
	})
})
