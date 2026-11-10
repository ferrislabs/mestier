import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { KnowledgeArticle } from '#/hooks/use-knowledge'
import { KnowledgePage } from '#/pages/knowledge/ui/knowledge-page'

const note: KnowledgeArticle = {
	id: 'n1',
	kind: 'note',
	title: 'Conditions de paiement',
	body: '30 jours fin de mois',
	file: null,
	tags: ['cgv'],
	created_at: '2026-11-10T08:00:00Z',
	updated_at: '2026-11-10T08:00:00Z',
}
const file: KnowledgeArticle = {
	...note,
	id: 'f1',
	kind: 'file',
	title: 'Tarifs gazon',
	body: null,
	file: { mime_type: 'application/pdf', size_bytes: 2048 },
	tags: [],
}

function renderPage(
	overrides: Partial<Parameters<typeof KnowledgePage>[0]> = {},
) {
	const props: Parameters<typeof KnowledgePage>[0] = {
		articles: [note, file],
		canManage: true,
		uploads: [],
		isUploading: false,
		isLoading: false,
		error: null,
		hasFilters: false,
		page: 1,
		hasNext: false,
		total: 2,
		q: '',
		tagInput: '',
		onFiles: vi.fn(),
		onClearUploads: vi.fn(),
		onNewNote: vi.fn(),
		onQChange: vi.fn(),
		onTagInputChange: vi.fn(),
		onPageChange: vi.fn(),
		onOpen: vi.fn(),
		...overrides,
	}
	render(<KnowledgePage {...props} />)
	return props
}

describe('KnowledgePage', () => {
	it('lists notes and files with their kind, size and tags', () => {
		renderPage()
		expect(screen.getByText('Conditions de paiement')).toBeTruthy()
		expect(screen.getByText('cgv')).toBeTruthy()
		expect(screen.getByText('PDF · 2 Ko')).toBeTruthy()
		expect(screen.getByText('Note')).toBeTruthy()
	})

	it('opens the article that is clicked', async () => {
		const props = renderPage()
		await userEvent.click(screen.getByText('Tarifs gazon'))
		expect(props.onOpen).toHaveBeenCalledWith(file)
	})

	it('offers the drop zone and the new note button to someone who can manage', async () => {
		const props = renderPage()
		expect(screen.getByText('Dépose des documents ici')).toBeTruthy()
		await userEvent.click(screen.getByRole('button', { name: /Nouvelle note/ }))
		expect(props.onNewNote).toHaveBeenCalled()
	})

	it('is read-only without the permission to manage', () => {
		renderPage({ canManage: false })
		expect(screen.queryByText('Dépose des documents ici')).toBeNull()
		expect(screen.queryByRole('button', { name: /Nouvelle note/ })).toBeNull()
		expect(screen.getByText('Conditions de paiement')).toBeTruthy()
	})

	it('tells an empty base from an empty search', () => {
		renderPage({ articles: [], total: 0 })
		expect(screen.getByText('La base de connaissances est vide')).toBeTruthy()
	})

	it('says nothing matched when a filter is on', () => {
		renderPage({ articles: [], total: 0, hasFilters: true })
		expect(screen.getByText('Aucun résultat')).toBeTruthy()
	})
})
