import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { CalloutBlock } from '#/pages/knowledge-documents/ui/callout-block'
import { DocumentBanner } from '#/pages/knowledge-documents/ui/document-banner'
import { FileBlock } from '#/pages/knowledge-documents/ui/file-block'
import { ImageBlock } from '#/pages/knowledge-documents/ui/image-block'
import { MentionChip } from '#/pages/knowledge-documents/ui/mention-chip'
import { SaveIndicator } from '#/pages/knowledge-documents/ui/save-indicator'
import { TableViewBlock } from '#/pages/knowledge-documents/ui/table-view-block'
import type { TableColumn, TableRow } from '#/pages/knowledge-tables/lib/types'

const nameColumn = {
	id: 'col-name',
	name: 'Nom',
	kind: 'text',
	is_primary: true,
	position: 0,
	config: {},
} as unknown as TableColumn
const rows = [
	{ id: 'r1', values: { 'col-name': 'Haie de charmille' }, labels: {} },
	{ id: 'r2', values: { 'col-name': 'Pelouse' }, labels: {} },
] as unknown as TableRow[]

function tableView(
	overrides: Partial<Parameters<typeof TableViewBlock>[0]> = {},
) {
	const props: Parameters<typeof TableViewBlock>[0] = {
		state: 'ready',
		tableName: 'Prestations',
		viewName: 'Toutes',
		columns: [nameColumn],
		rows,
		total: 2,
		isLoadingRows: false,
		editable: false,
		onOpen: vi.fn(),
		onPick: vi.fn(),
		...overrides,
	}
	render(<TableViewBlock {...props} />)
	return props
}

describe('DocumentBanner', () => {
	it('names the person who is editing', () => {
		render(
			<DocumentBanner
				banner={{ kind: 'locked', holderName: 'Erwan Treyve' }}
				onReload={vi.fn()}
			/>,
		)
		expect(
			screen.getByText('En cours de modification par Erwan Treyve'),
		).toBeTruthy()
	})

	it('says the document was changed elsewhere and offers to reload', async () => {
		const onReload = vi.fn()
		render(
			<DocumentBanner
				banner={{ kind: 'conflict', conflict: 'lock_lost' }}
				onReload={onReload}
			/>,
		)
		expect(screen.getByText('Ce document a été modifié ailleurs')).toBeTruthy()
		await userEvent.click(screen.getByRole('button', { name: 'Recharger' }))
		expect(onReload).toHaveBeenCalled()
	})
})

describe('SaveIndicator', () => {
	it('shows saving then saved', () => {
		const { rerender } = render(<SaveIndicator state="saving" error={null} />)
		expect(screen.getByText('Enregistrement…')).toBeTruthy()
		rerender(<SaveIndicator state="saved" error={null} />)
		expect(screen.getByText('Enregistré')).toBeTruthy()
	})

	it('shows the saving label while a change is waiting for its save', () => {
		render(<SaveIndicator state="dirty" error={null} />)
		expect(screen.getByText('Enregistrement…')).toBeTruthy()
	})

	it('shows nothing before any change and the reason after a failure', () => {
		const { container, rerender } = render(
			<SaveIndicator state="idle" error={null} />,
		)
		expect(container.textContent).toBe('')
		rerender(<SaveIndicator state="error" error="Trop de blocs." />)
		expect(screen.getByText('Trop de blocs.')).toBeTruthy()
	})
})

describe('TableViewBlock', () => {
	it('shows the rows of the view and opens the table', async () => {
		const props = tableView()
		expect(screen.getByText('Haie de charmille')).toBeTruthy()
		expect(screen.getByText('Pelouse')).toBeTruthy()
		await userEvent.click(
			screen.getByRole('button', { name: /Ouvrir la table/ }),
		)
		expect(props.onOpen).toHaveBeenCalled()
	})

	it('shows only the first 50 rows and says how many there are', () => {
		const many = Array.from({ length: 60 }, (_, index) => ({
			id: `r${index}`,
			values: { 'col-name': `Ligne ${index}` },
			labels: {},
		})) as unknown as TableRow[]
		tableView({ rows: many, total: 120 })
		expect(screen.getAllByRole('row')).toHaveLength(51)
		expect(screen.getByText('50 premières lignes sur 120.')).toBeTruthy()
	})

	it('says the element was deleted when the table or view is gone', () => {
		tableView({ state: 'missing' })
		expect(screen.getByText('Élément supprimé')).toBeTruthy()
		expect(screen.queryByRole('button', { name: /Ouvrir la table/ })).toBeNull()
	})

	it('offers to pick a view only to someone who can edit', () => {
		tableView({ state: 'empty', editable: false })
		expect(screen.queryByRole('button', { name: /Choisir une vue/ })).toBeNull()
	})
})

describe('ImageBlock', () => {
	const base: Parameters<typeof ImageBlock>[0] = {
		state: 'ready',
		url: 'blob:image',
		caption: 'Haie taillée',
		width: 50,
		editable: false,
		onPick: vi.fn(),
		onCaptionChange: vi.fn(),
		onWidthChange: vi.fn(),
	}

	it('shows the image with its caption', () => {
		render(<ImageBlock {...base} />)
		expect(screen.getByRole('img').getAttribute('src')).toBe('blob:image')
		expect(screen.getByText('Haie taillée')).toBeTruthy()
	})

	it('says the element was deleted when the image is gone', () => {
		render(<ImageBlock {...base} state="missing" url={null} />)
		expect(screen.getByText('Élément supprimé')).toBeTruthy()
		expect(screen.queryByRole('img')).toBeNull()
	})

	it('lets an editor change the caption and the width', async () => {
		const onCaptionChange = vi.fn()
		render(<ImageBlock {...base} editable onCaptionChange={onCaptionChange} />)
		await userEvent.type(screen.getByLabelText('Légende de l’image'), '!')
		expect(onCaptionChange).toHaveBeenCalledWith('Haie taillée!')
		expect(screen.getByText('50 %')).toBeTruthy()
	})
})

describe('FileBlock', () => {
	it('says the element was deleted when the file is gone', () => {
		render(
			<FileBlock
				state="missing"
				title=""
				detail=""
				editable={false}
				isDownloading={false}
				onPick={vi.fn()}
				onDownload={vi.fn()}
			/>,
		)
		expect(screen.getByText('Élément supprimé')).toBeTruthy()
	})

	it('downloads the file', async () => {
		const onDownload = vi.fn()
		render(
			<FileBlock
				state="ready"
				title="CGV 2026"
				detail="PDF · 2 Ko"
				editable={false}
				isDownloading={false}
				onPick={vi.fn()}
				onDownload={onDownload}
			/>,
		)
		await userEvent.click(screen.getByRole('button', { name: /Télécharger/ }))
		expect(onDownload).toHaveBeenCalled()
	})
})

describe('MentionChip', () => {
	it('links to the record under its current name', async () => {
		const onNavigate = vi.fn()
		render(
			<MentionChip
				label="Jardins du Parc"
				missing={false}
				href="/o/acme/crm/clients/c1"
				onNavigate={onNavigate}
			/>,
		)
		const link = screen.getByRole('link', { name: '@Jardins du Parc' })
		expect(link.getAttribute('href')).toBe('/o/acme/crm/clients/c1')
		await userEvent.click(link)
		expect(onNavigate).toHaveBeenCalled()
	})

	it('says the element was deleted when the record is gone', () => {
		render(
			<MentionChip label="Ancien" missing href="/x" onNavigate={vi.fn()} />,
		)
		expect(screen.getByText('Élément supprimé')).toBeTruthy()
		expect(screen.queryByRole('link')).toBeNull()
	})
})

describe('CalloutBlock', () => {
	it('lets an editor change the tone, and a reader not', async () => {
		const onToneChange = vi.fn()
		const { rerender } = render(
			<CalloutBlock
				tone="info"
				editable
				contentRef={() => {}}
				onToneChange={onToneChange}
			/>,
		)
		await userEvent.click(screen.getByRole('button', { name: 'Attention' }))
		expect(onToneChange).toHaveBeenCalledWith('warning')
		rerender(
			<CalloutBlock
				tone="info"
				editable={false}
				contentRef={() => {}}
				onToneChange={onToneChange}
			/>,
		)
		expect(screen.queryByRole('button', { name: 'Attention' })).toBeNull()
	})
})
