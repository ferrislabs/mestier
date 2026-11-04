import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type {
	SupportEntryVM,
	SupportThreadUIProps,
} from '#/pages/support/types'
import { SupportThreadUI } from '#/pages/support/ui/support-thread-ui'

const ENTRIES: SupportEntryVM[] = [
	{
		id: 'entry-1',
		kind: 'client_message',
		kindLabel: 'Message du client',
		visibilityLabel: 'Visible par le client',
		authorLabel: 'Camille Durand',
		createdAtLabel: '3 nov. 2026, 09:12',
		body: 'Bonjour,\nl’évier fuit encore.',
		attachments: [
			{ id: 'attachment-1', fileName: 'photo-evier.jpg', sizeLabel: '1,2 Mo' },
			{ id: 'attachment-2', fileName: 'facture.pdf', sizeLabel: '86 Ko' },
		],
	},
	{
		id: 'entry-2',
		kind: 'reply',
		kindLabel: 'Réponse',
		visibilityLabel: 'Visible par le client',
		authorLabel: 'Alix Martin',
		createdAtLabel: '3 nov. 2026, 10:40',
		body: 'Nous passons demain matin.',
		attachments: [],
	},
	{
		id: 'entry-3',
		kind: 'internal_note',
		kindLabel: 'Note interne',
		visibilityLabel: 'Équipe uniquement',
		authorLabel: 'Sam Roux',
		createdAtLabel: '3 nov. 2026, 11:05',
		body: 'Prévoir un joint de rechange.',
		attachments: [],
	},
	{
		id: 'entry-4',
		kind: 'draft',
		kindLabel: 'Brouillon',
		visibilityLabel: 'Non envoyé',
		authorLabel: 'Alix Martin',
		createdAtLabel: '3 nov. 2026, 11:30',
		body: 'Brouillon à relire avant envoi.',
		attachments: [],
	},
]

function renderThread(overrides: Partial<SupportThreadUIProps> = {}) {
	const props: SupportThreadUIProps = {
		entries: ENTRIES,
		canManage: true,
		isLoading: false,
		error: null,
		onRetry: vi.fn(),
		editingEntryId: null,
		pendingEntryId: null,
		onStartEdit: vi.fn(),
		onCancelEdit: vi.fn(),
		onSaveEdit: vi.fn(),
		onSendDraft: vi.fn(),
		onDeleteDraft: vi.fn(),
		onOpenAttachment: vi.fn(),
		...overrides,
	}
	render(<SupportThreadUI {...props} />)
	return props
}

function articleNamed(name: string) {
	return screen.getByRole('article', { name })
}

function buttonNames(container: HTMLElement) {
	return within(container)
		.queryAllByRole('button')
		.map((button) => button.textContent)
}

describe('SupportThreadUI', () => {
	it('renders the thread heading', () => {
		renderThread()

		expect(
			screen.getByRole('heading', { name: 'Fil de discussion' }),
		).toBeTruthy()
	})

	it('renders the entries in the given order', () => {
		renderThread()

		expect(
			screen
				.getAllByRole('article')
				.map((article) => article.getAttribute('aria-label')),
		).toEqual([
			'Message du client, Camille Durand',
			'Réponse, Alix Martin',
			'Note interne, Sam Roux',
			'Brouillon, Alix Martin',
		])
	})

	it('does not reorder the entries it is given', () => {
		renderThread({ entries: [...ENTRIES].reverse() })

		expect(
			screen
				.getAllByRole('article')
				.map((article) => article.getAttribute('aria-label')),
		).toEqual([
			'Brouillon, Alix Martin',
			'Note interne, Sam Roux',
			'Réponse, Alix Martin',
			'Message du client, Camille Durand',
		])
	})

	it('shows kind, author, date, visibility and body on an entry', () => {
		renderThread()

		const article = articleNamed('Message du client, Camille Durand')
		expect(within(article).getByText('Message du client')).toBeTruthy()
		expect(within(article).getByText('Camille Durand')).toBeTruthy()
		expect(within(article).getByText('3 nov. 2026, 09:12')).toBeTruthy()
		expect(within(article).getByText('Visible par le client')).toBeTruthy()
		expect(within(article).getByText(/l’évier fuit encore/).textContent).toBe(
			'Bonjour,\nl’évier fuit encore.',
		)
	})

	it('tells internal notes and drafts apart by kind and visibility', () => {
		renderThread()

		const note = articleNamed('Note interne, Sam Roux')
		expect(within(note).getByText('Note interne')).toBeTruthy()
		expect(within(note).getByText('Équipe uniquement')).toBeTruthy()

		const draft = articleNamed('Brouillon, Alix Martin')
		expect(within(draft).getByText('Brouillon')).toBeTruthy()
		expect(within(draft).getByText('Non envoyé')).toBeTruthy()

		const reply = articleNamed('Réponse, Alix Martin')
		expect(within(reply).queryByText('Note interne')).toBeNull()
		expect(within(reply).queryByText('Brouillon')).toBeNull()
		expect(within(reply).queryByText('Équipe uniquement')).toBeNull()
	})

	it('renders one button per attachment, labelled with name and size', () => {
		renderThread()

		const article = articleNamed('Message du client, Camille Durand')
		expect(
			within(article)
				.getAllByRole('button')
				.map((button) => button.textContent),
		).toEqual(['photo-evier.jpg (1,2 Mo)', 'facture.pdf (86 Ko)'])
	})

	it('opens an attachment by its id, even without canManage', async () => {
		const props = renderThread({ canManage: false })

		await userEvent.click(
			screen.getByRole('button', { name: 'facture.pdf (86 Ko)' }),
		)

		expect(props.onOpenAttachment).toHaveBeenCalledTimes(1)
		expect(props.onOpenAttachment).toHaveBeenCalledWith('attachment-2')
	})

	it('offers Modifier, Envoyer and Supprimer on drafts only, with canManage', () => {
		renderThread()

		expect(buttonNames(articleNamed('Brouillon, Alix Martin'))).toEqual([
			'Modifier',
			'Envoyer',
			'Supprimer',
		])
		expect(buttonNames(articleNamed('Réponse, Alix Martin'))).toEqual([])
		expect(buttonNames(articleNamed('Note interne, Sam Roux'))).toEqual([])
		expect(
			buttonNames(articleNamed('Message du client, Camille Durand')),
		).toEqual(['photo-evier.jpg (1,2 Mo)', 'facture.pdf (86 Ko)'])
	})

	it('renders no control of any kind without canManage', () => {
		renderThread({ canManage: false })

		expect(screen.queryByRole('button', { name: 'Modifier' })).toBeNull()
		expect(screen.queryByRole('button', { name: 'Envoyer' })).toBeNull()
		expect(screen.queryByRole('button', { name: 'Supprimer' })).toBeNull()
		expect(buttonNames(articleNamed('Brouillon, Alix Martin'))).toEqual([])
		expect(screen.queryByRole('textbox')).toBeNull()
	})

	it('calls the draft callbacks with the id of the draft', async () => {
		const props = renderThread()

		await userEvent.click(screen.getByRole('button', { name: 'Modifier' }))
		await userEvent.click(screen.getByRole('button', { name: 'Envoyer' }))
		await userEvent.click(screen.getByRole('button', { name: 'Supprimer' }))

		expect(props.onStartEdit).toHaveBeenCalledTimes(1)
		expect(props.onStartEdit).toHaveBeenCalledWith('entry-4')
		expect(props.onSendDraft).toHaveBeenCalledTimes(1)
		expect(props.onSendDraft).toHaveBeenCalledWith('entry-4')
		expect(props.onDeleteDraft).toHaveBeenCalledTimes(1)
		expect(props.onDeleteDraft).toHaveBeenCalledWith('entry-4')
	})

	it('disables the draft controls while the draft is pending', () => {
		renderThread({ pendingEntryId: 'entry-4' })

		for (const name of ['Modifier', 'Envoyer', 'Supprimer']) {
			expect(
				(screen.getByRole('button', { name }) as HTMLButtonElement).disabled,
			).toBe(true)
		}
	})

	it('keeps the draft controls enabled while another entry is pending', () => {
		renderThread({ pendingEntryId: 'entry-1' })

		for (const name of ['Modifier', 'Envoyer', 'Supprimer']) {
			expect(
				(screen.getByRole('button', { name }) as HTMLButtonElement).disabled,
			).toBe(false)
		}
	})

	it('swaps the body of the edited draft for a prefilled textarea', () => {
		renderThread({ editingEntryId: 'entry-4' })

		const textarea = screen.getByRole('textbox', {
			name: 'Modifier le brouillon',
		}) as HTMLTextAreaElement
		expect(textarea.value).toBe('Brouillon à relire avant envoi.')
		expect(screen.getAllByRole('textbox')).toHaveLength(1)
		expect(screen.getByRole('button', { name: 'Enregistrer' })).toBeTruthy()
		expect(screen.getByRole('button', { name: 'Annuler' })).toBeTruthy()
		expect(screen.queryByRole('button', { name: 'Modifier' })).toBeNull()
	})

	it('saves the edited text and nothing else', async () => {
		const props = renderThread({ editingEntryId: 'entry-4' })
		const textarea = screen.getByRole('textbox', {
			name: 'Modifier le brouillon',
		})

		await userEvent.clear(textarea)
		await userEvent.type(textarea, 'Texte corrigé')
		await userEvent.click(screen.getByRole('button', { name: 'Enregistrer' }))

		expect(props.onSaveEdit).toHaveBeenCalledTimes(1)
		expect(props.onSaveEdit).toHaveBeenCalledWith('entry-4', 'Texte corrigé')
	})

	it('disables Enregistrer while the text is blank after trimming', async () => {
		const props = renderThread({ editingEntryId: 'entry-4' })
		const textarea = screen.getByRole('textbox', {
			name: 'Modifier le brouillon',
		})
		const save = screen.getByRole('button', {
			name: 'Enregistrer',
		}) as HTMLButtonElement
		expect(save.disabled).toBe(false)

		await userEvent.clear(textarea)
		expect(save.disabled).toBe(true)

		await userEvent.type(textarea, '   ')
		expect(save.disabled).toBe(true)

		await userEvent.click(save)
		expect(props.onSaveEdit).not.toHaveBeenCalled()

		await userEvent.type(textarea, 'x')
		expect(save.disabled).toBe(false)
	})

	it('cancels the edit', async () => {
		const props = renderThread({ editingEntryId: 'entry-4' })

		await userEvent.click(screen.getByRole('button', { name: 'Annuler' }))

		expect(props.onCancelEdit).toHaveBeenCalledTimes(1)
		expect(props.onSaveEdit).not.toHaveBeenCalled()
	})

	it('disables the edit buttons while the edited draft is pending', () => {
		renderThread({ editingEntryId: 'entry-4', pendingEntryId: 'entry-4' })

		expect(
			(screen.getByRole('button', { name: 'Enregistrer' }) as HTMLButtonElement)
				.disabled,
		).toBe(true)
		expect(
			(screen.getByRole('button', { name: 'Annuler' }) as HTMLButtonElement)
				.disabled,
		).toBe(true)
	})

	it('never edits an entry that is not a draft', () => {
		renderThread({ editingEntryId: 'entry-3' })

		expect(screen.queryByRole('textbox')).toBeNull()
		expect(screen.getByText('Prévoir un joint de rechange.')).toBeTruthy()
	})

	it('never edits without canManage', () => {
		renderThread({ canManage: false, editingEntryId: 'entry-4' })

		expect(screen.queryByRole('textbox')).toBeNull()
		expect(screen.getByText('Brouillon à relire avant envoi.')).toBeTruthy()
	})

	it('shows the loading wording instead of the entries while loading', () => {
		renderThread({ isLoading: true })

		expect(screen.getByText('Chargement…')).toBeTruthy()
		expect(screen.queryByRole('article')).toBeNull()
		expect(screen.queryByText('Aucun message pour le moment.')).toBeNull()
	})

	it('shows the empty wording without entries', () => {
		renderThread({ entries: [] })

		expect(screen.getByText('Aucun message pour le moment.')).toBeTruthy()
		expect(screen.queryByRole('article')).toBeNull()
	})

	it('shows the error banner and calls onRetry from its button', async () => {
		const props = renderThread({
			entries: [],
			error: 'Impossible de charger le fil.',
		})

		expect(screen.getByText('Impossible de charger le fil.')).toBeTruthy()
		expect(screen.queryByText('Aucun message pour le moment.')).toBeNull()
		await userEvent.click(screen.getByRole('button', { name: 'Réessayer' }))

		expect(props.onRetry).toHaveBeenCalledTimes(1)
	})

	it('shows no error banner without an error', () => {
		renderThread()

		expect(screen.queryByRole('button', { name: 'Réessayer' })).toBeNull()
	})
})
