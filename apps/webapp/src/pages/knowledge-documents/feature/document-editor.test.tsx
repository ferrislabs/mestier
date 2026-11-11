import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { EditorBlock } from '#/pages/knowledge-documents/lib/blocks'

const customer = vi.hoisted(() => ({
	current: {
		data: { data: { name: 'Jardins du Parc SARL' } } as unknown,
		isError: false,
		error: null as unknown,
	},
}))
const table = vi.hoisted(() => ({
	detail: {
		data: undefined as unknown,
		isError: true,
		error: { status: 404 },
	},
}))

vi.mock('#/hooks/use-customers', () => ({
	useCustomer: () => customer.current,
}))
vi.mock('#/hooks/use-projects', () => ({
	useProject: () => ({ data: undefined, isError: false, error: null }),
}))
vi.mock('#/hooks/use-quotes', () => ({
	useQuote: () => ({ data: undefined, isError: false, error: null }),
}))
vi.mock('#/hooks/use-knowledge-tables', () => ({
	useKnowledgeTable: () => table.detail,
	useKnowledgeTables: () => ({ data: [], isLoading: false }),
	useTableRows: () => ({ data: undefined, isLoading: false }),
}))
vi.mock('#/pages/knowledge/feature/use-resource-options', () => ({
	useResourceOptions: () => [],
	RESOURCE_TYPE_LABELS: {},
	resourceLabel: () => '',
}))

import { DocumentEditor } from '#/pages/knowledge-documents/feature/document-editor'

const environment = {
	organizationId: 'org-1',
	folderId: null,
	editable: false,
	resourceHref: (kind: string, id: string) => `/o/acme/${kind}/${id}`,
	openResource: vi.fn(),
	openTable: vi.fn(),
}

const blocks: EditorBlock[] = [
	{
		id: 'p1',
		type: 'paragraph',
		props: {},
		content: [
			{ type: 'text', text: 'Client : ', styles: {} },
			{
				type: 'mention',
				props: {
					resourceType: 'customer',
					resourceId: 'c1',
					label: 'Ancien nom',
				},
			},
		],
		children: [],
	},
	{
		id: 't1',
		type: 'tableView',
		props: { tableId: 'gone', viewId: 'v1' },
		children: [],
	},
]

function renderEditor(editable: boolean) {
	render(
		<QueryClientProvider client={new QueryClient()}>
			<DocumentEditor
				initialBlocks={blocks}
				editable={editable}
				environment={{ ...environment, editable }}
				onChange={vi.fn()}
			/>
		</QueryClientProvider>,
	)
}

beforeEach(() => {
	customer.current = {
		data: { data: { name: 'Jardins du Parc SARL' } },
		isError: false,
		error: null,
	}
})

describe('DocumentEditor', () => {
	it('shows a mention under the current name of the customer and links to it', async () => {
		renderEditor(false)
		const link = await screen.findByRole('link', {
			name: '@Jardins du Parc SARL',
		})
		expect(link.getAttribute('href')).toBe('/o/acme/customer/c1')
	})

	it('shows the stored name when the customer cannot be read for another reason', async () => {
		customer.current = {
			data: undefined,
			isError: true,
			error: { status: 500 },
		}
		renderEditor(false)
		expect(await screen.findByText('@Ancien nom')).toBeTruthy()
	})

	it('says the element was deleted when the customer is gone', async () => {
		customer.current = {
			data: undefined,
			isError: true,
			error: { status: 404 },
		}
		renderEditor(false)
		await waitFor(() =>
			expect(screen.getAllByText('Élément supprimé').length).toBeGreaterThan(0),
		)
		expect(screen.queryByText('@Ancien nom')).toBeNull()
	})

	it('says the element was deleted when a table view points at a deleted table', async () => {
		renderEditor(false)
		await waitFor(() =>
			expect(screen.getAllByText('Élément supprimé').length).toBeGreaterThan(0),
		)
	})

	it('is read-only when the document is not editable and editable otherwise', async () => {
		renderEditor(false)
		await screen.findByRole('link', { name: '@Jardins du Parc SARL' })
		expect(
			document.querySelector('.bn-editor')?.getAttribute('contenteditable'),
		).toBe('false')
	})

	it('is editable when the lock is held', async () => {
		renderEditor(true)
		await screen.findByRole('link', { name: '@Jardins du Parc SARL' })
		expect(
			document.querySelector('.bn-editor')?.getAttribute('contenteditable'),
		).toBe('true')
	})
})
