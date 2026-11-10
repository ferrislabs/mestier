import { fireEvent, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { KnowledgeArticle, KnowledgeFolder } from '#/hooks/use-knowledge'
import { ARTICLE_DRAG_TYPE } from '#/pages/knowledge/lib/folders'
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

const chantiers: KnowledgeFolder = {
	id: 'd1',
	parent_id: null,
	name: 'Chantiers',
	created_at: '2026-11-10T08:00:00Z',
	updated_at: '2026-11-10T08:00:00Z',
}
const parc: KnowledgeFolder = {
	...chantiers,
	id: 'd2',
	parent_id: 'd1',
	name: 'Parc',
}

function renderPage(
	overrides: Partial<Parameters<typeof KnowledgePage>[0]> = {},
) {
	const props: Parameters<typeof KnowledgePage>[0] = {
		folders: [chantiers, parc],
		currentFolderId: null,
		path: [],
		subfolders: [chantiers],
		expanded: new Set(),
		articles: [note, file],
		selected: new Set(),
		canManage: true,
		canCreateFolder: true,
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
		onOpenFolder: vi.fn(),
		onToggleFolder: vi.fn(),
		onNewFolder: vi.fn(),
		onRenameFolder: vi.fn(),
		onMoveFolder: vi.fn(),
		onDeleteFolder: vi.fn(),
		onDropArticles: vi.fn(),
		onToggleSelected: vi.fn(),
		onClearSelection: vi.fn(),
		onMoveSelected: vi.fn(),
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
		expect(screen.getByText('Dépose des documents dans « Drive »')).toBeTruthy()
		await userEvent.click(screen.getByRole('button', { name: /Nouvelle note/ }))
		expect(props.onNewNote).toHaveBeenCalled()
	})

	it('is read-only without the permission to manage', () => {
		renderPage({ canManage: false })
		expect(screen.queryByText('Dépose des documents dans « Drive »')).toBeNull()
		expect(screen.queryByRole('button', { name: /Nouvelle note/ })).toBeNull()
		expect(screen.getByText('Conditions de paiement')).toBeTruthy()
	})

	it('tells an empty base from an empty search', () => {
		renderPage({ articles: [], subfolders: [], total: 0 })
		expect(screen.getByText('Le drive est vide')).toBeTruthy()
	})

	it('says nothing matched when a filter is on', () => {
		renderPage({ articles: [], total: 0, hasFilters: true })
		expect(screen.getByText('Aucun résultat')).toBeTruthy()
	})

	it('lists the subfolders before the articles and opens one', async () => {
		const props = renderPage()
		const rows = screen.getAllByRole('listitem').map((row) => row.textContent)
		expect(rows.findIndex((text) => text?.includes('Chantiers'))).toBeLessThan(
			rows.findIndex((text) => text?.includes('Conditions de paiement')),
		)
		await userEvent.click(
			screen
				.getAllByRole('button', { name: 'Chantiers' })
				.at(-1) as HTMLElement,
		)
		expect(props.onOpenFolder).toHaveBeenCalledWith('d1')
	})

	it('hides the subfolders while searching', () => {
		renderPage({ hasFilters: true })
		expect(
			screen.queryByRole('button', { name: 'Actions du dossier Chantiers' }),
		).toBeNull()
		expect(screen.getByText('Conditions de paiement')).toBeTruthy()
	})

	it('shows the path to the open folder and goes back up', async () => {
		const props = renderPage({
			currentFolderId: 'd2',
			path: [chantiers, parc],
			subfolders: [],
		})
		const crumbs = screen.getByRole('navigation', { name: 'Emplacement' })
		expect(crumbs.textContent).toContain('Chantiers')
		expect(crumbs.textContent).toContain('Parc')
		await userEvent.click(
			screen
				.getAllByRole('button', { name: 'Drive' })
				.find((button) => crumbs.contains(button)) as HTMLElement,
		)
		expect(props.onOpenFolder).toHaveBeenCalledWith(null)
	})

	it('moves the selection through the selection bar', async () => {
		const props = renderPage({ selected: new Set(['n1', 'f1']) })
		expect(screen.getByText('2 éléments sélectionnés')).toBeTruthy()
		await userEvent.click(screen.getByRole('button', { name: /Déplacer vers/ }))
		expect(props.onMoveSelected).toHaveBeenCalled()
	})

	it('drops dragged articles on a folder row', () => {
		const props = renderPage()
		const target = screen
			.getByRole('button', { name: 'Actions du dossier Chantiers' })
			.closest('div') as HTMLElement
		const data = new Map<string, string>([
			[ARTICLE_DRAG_TYPE, JSON.stringify(['n1'])],
		])
		const dataTransfer = {
			types: [...data.keys()],
			getData: (type: string) => data.get(type) ?? '',
			dropEffect: 'none',
		}
		fireEvent.dragOver(target, { dataTransfer })
		fireEvent.drop(target, { dataTransfer })
		expect(props.onDropArticles).toHaveBeenCalledWith('d1', ['n1'])
	})

	it('offers no folder action, selection or drop without the permission', () => {
		renderPage({ canManage: false })
		expect(
			screen.queryByRole('button', { name: 'Actions du dossier Chantiers' }),
		).toBeNull()
		expect(screen.queryByRole('checkbox')).toBeNull()
		expect(screen.queryByRole('button', { name: /Nouveau dossier/ })).toBeNull()
	})
})
