import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { CalloutBlock } from '#/pages/knowledge-documents/ui/callout-block'
import { DocumentBanner } from '#/pages/knowledge-documents/ui/document-banner'
import {
	DocumentLoading,
	DocumentPage,
} from '#/pages/knowledge-documents/ui/document-page'
import { DocumentProperties } from '#/pages/knowledge-documents/ui/document-properties'
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

	it('does not spin while saving', () => {
		const { container } = render(<SaveIndicator state="saving" error={null} />)
		expect(container.querySelector('.animate-spin')).toBeNull()
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
		const onWidthChange = vi.fn()
		render(
			<ImageBlock
				{...base}
				editable
				onCaptionChange={onCaptionChange}
				onWidthChange={onWidthChange}
			/>,
		)
		await userEvent.type(screen.getByLabelText('Légende de l’image'), '!')
		expect(onCaptionChange).toHaveBeenCalledWith('Haie taillée!')
		expect(
			screen
				.getByRole('button', { name: 'Largeur 50 %' })
				.getAttribute('aria-pressed'),
		).toBe('true')
		await userEvent.click(screen.getByRole('button', { name: 'Largeur 75 %' }))
		expect(onWidthChange).toHaveBeenCalledWith(75)
	})

	it('shows no width control to a reader', () => {
		render(<ImageBlock {...base} />)
		expect(screen.queryByRole('button', { name: /Largeur/ })).toBeNull()
	})

	it('offers to add an image when the block is empty', async () => {
		const onPick = vi.fn()
		render(
			<ImageBlock
				{...base}
				state="empty"
				url={null}
				editable
				onPick={onPick}
			/>,
		)
		await userEvent.click(
			screen.getByRole('button', { name: 'Ajouter une image' }),
		)
		expect(onPick).toHaveBeenCalled()
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

function documentPage(
	overrides: Partial<Parameters<typeof DocumentPage>[0]> = {},
) {
	const props: Parameters<typeof DocumentPage>[0] = {
		title: 'Conditions de paiement',
		canManage: true,
		path: [
			{ id: 'f1', name: 'Administratif' },
			{ id: 'f2', name: 'Clients' },
		],
		indicator: null,
		properties: null,
		banner: null,
		children: null,
		onTitleChange: vi.fn(),
		onTitleCommit: vi.fn(),
		onOpenFolder: vi.fn(),
		onOpenInfo: vi.fn(),
		onMove: vi.fn(),
		onDelete: vi.fn(),
		...overrides,
	}
	render(<DocumentPage {...props} />)
	return props
}

describe('DocumentPage', () => {
	it('shows where the document lives and opens a folder of the path', async () => {
		const props = documentPage()
		const trail = screen.getByRole('navigation', { name: 'breadcrumb' })
		expect(trail.textContent).toContain('Drive')
		expect(trail.textContent).toContain('Administratif')
		expect(trail.textContent).toContain('Conditions de paiement')
		await userEvent.click(screen.getByRole('button', { name: 'Clients' }))
		expect(props.onOpenFolder).toHaveBeenCalledWith('f2')
		await userEvent.click(screen.getByRole('button', { name: 'Drive' }))
		expect(props.onOpenFolder).toHaveBeenCalledWith(null)
	})

	it('names an untitled document in the path', () => {
		documentPage({ title: '  ' })
		expect(
			screen.getByRole('navigation', { name: 'breadcrumb' }).textContent,
		).toContain('Sans titre')
	})

	it('groups moving and deleting in the actions menu', async () => {
		const props = documentPage()
		await userEvent.click(
			screen.getByRole('button', { name: 'Actions du document' }),
		)
		await userEvent.click(screen.getByRole('menuitem', { name: 'Supprimer' }))
		expect(props.onDelete).toHaveBeenCalled()
		await userEvent.click(
			screen.getByRole('button', { name: 'Actions du document' }),
		)
		await userEvent.click(screen.getByRole('menuitem', { name: 'Déplacer' }))
		expect(props.onMove).toHaveBeenCalled()
	})

	it('shows no actions menu and a read-only title to a reader', () => {
		documentPage({ canManage: false })
		expect(
			screen.queryByRole('button', { name: 'Actions du document' }),
		).toBeNull()
		expect(
			screen.getByLabelText('Titre du document').hasAttribute('readonly'),
		).toBe(true)
	})

	it('announces the loading of the document', () => {
		render(<DocumentLoading />)
		expect(screen.getByText('Chargement du document…')).toBeTruthy()
	})
})

describe('DocumentProperties', () => {
	it('shows the tags and the links and opens the infos', async () => {
		const onOpenInfo = vi.fn()
		render(
			<DocumentProperties
				canManage
				tags={['paiement', 'cgv']}
				links={[{ key: 'customer:c1', label: 'Client · Jardins du Parc' }]}
				onOpenInfo={onOpenInfo}
			/>,
		)
		expect(screen.getByText('paiement')).toBeTruthy()
		expect(screen.getByText('Client · Jardins du Parc')).toBeTruthy()
		await userEvent.click(screen.getByRole('button', { name: /paiement/ }))
		expect(onOpenInfo).toHaveBeenCalled()
	})

	it('invites an editor to fill empty properties', () => {
		render(
			<DocumentProperties
				canManage
				tags={[]}
				links={[]}
				onOpenInfo={vi.fn()}
			/>,
		)
		expect(screen.getByText('Ajouter des étiquettes')).toBeTruthy()
		expect(
			screen.getByText('Rattacher à un projet, client ou devis'),
		).toBeTruthy()
	})

	it('shows nothing to a reader when there is nothing to show', () => {
		const { container } = render(
			<DocumentProperties
				canManage={false}
				tags={[]}
				links={[]}
				onOpenInfo={vi.fn()}
			/>,
		)
		expect(container.textContent).toBe('')
	})
})
