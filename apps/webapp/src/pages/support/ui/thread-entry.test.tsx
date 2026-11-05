import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { entryVM } from '#/pages/support/ui/support-fixtures'
import { ThreadEntry } from '#/pages/support/ui/thread-entry'

function renderEntry(
	props: Partial<React.ComponentProps<typeof ThreadEntry>> = {},
) {
	const onOpenAttachment = vi.fn()
	render(
		<ol>
			<ThreadEntry
				entry={entryVM()}
				onOpenAttachment={onOpenAttachment}
				{...props}
			/>
		</ol>,
	)
	return { onOpenAttachment }
}

describe('ThreadEntry', () => {
	it('labels a reply as sent to the customer', () => {
		renderEntry()

		const article = screen.getByRole('article', {
			name: 'Envoyé au client de Alix Martin',
		})
		expect(within(article).getByText('Nous passons demain.')).toBeTruthy()
		expect(within(article).getByText('Envoyé au client')).toBeTruthy()
	})

	it('keeps the customer on the other side of the thread', () => {
		renderEntry({
			entry: entryVM({
				kind: 'client_message',
				authorLabel: 'Claire Fontaine',
				authorHue: null,
			}),
		})

		const row = screen.getByRole('listitem')
		expect(row.className).not.toContain('flex-row-reverse')
		expect(
			screen.getByRole('article', { name: 'Client de Claire Fontaine' }),
		).toBeTruthy()
	})

	it('puts the team replies on the right', () => {
		renderEntry()

		expect(screen.getByRole('listitem').className).toContain('flex-row-reverse')
	})

	it('marks an internal note as visible by the team only', () => {
		renderEntry({ entry: entryVM({ kind: 'internal_note' }) })

		expect(screen.getByText(/Note interne · visible par l’équipe/)).toBeTruthy()
	})

	it('marks a draft as not sent yet', () => {
		renderEntry({ entry: entryVM({ kind: 'draft' }) })

		expect(screen.getByText(/Brouillon · pas encore envoyé/)).toBeTruthy()
	})

	it('shows the date as a time element with the full date as title', () => {
		renderEntry()

		const time = screen.getByRole('article').querySelector('time')
		expect(time?.getAttribute('dateTime')).toBe('2026-11-01T09:00:00Z')
		expect(time?.getAttribute('title')).toMatch(/2026/)
	})

	it('opens an attachment', async () => {
		const { onOpenAttachment } = renderEntry({
			entry: entryVM({
				attachments: [
					{ id: 'attachment-1', fileName: 'photo.jpg', sizeLabel: '2 Ko' },
				],
			}),
		})

		await userEvent.click(
			screen.getByRole('button', { name: 'photo.jpg (2 Ko)' }),
		)

		expect(onOpenAttachment).toHaveBeenCalledWith('attachment-1')
	})

	it('highlights an entry being edited', () => {
		renderEntry({ editing: true })

		expect(screen.getByRole('article').className).toContain('ring-2')
	})

	it('offers no draft action unless they are given', () => {
		renderEntry({ entry: entryVM({ kind: 'draft' }) })

		expect(screen.queryByRole('button', { name: 'Modifier' })).toBeNull()
	})
})

describe('ThreadEntry — brouillon', () => {
	function renderDraft(
		props: Partial<React.ComponentProps<typeof ThreadEntry>> = {},
	) {
		const actions = {
			onEdit: vi.fn(),
			onSend: vi.fn(),
			onDelete: vi.fn(),
			onToggleApproval: vi.fn(),
			onComment: vi.fn(),
		}
		renderEntry({
			entry: entryVM({ kind: 'draft' }),
			draftActions: actions,
			...props,
		})
		return actions
	}

	it('edits and deletes the draft', async () => {
		const actions = renderDraft()

		await userEvent.click(screen.getByRole('button', { name: 'Modifier' }))
		await userEvent.click(
			screen.getByRole('button', { name: 'Supprimer le brouillon' }),
		)

		expect(actions.onEdit).toHaveBeenCalledTimes(1)
		expect(actions.onDelete).toHaveBeenCalledTimes(1)
	})

	it('sends the draft after a confirmation', async () => {
		const actions = renderDraft()

		await userEvent.click(
			screen.getByRole('button', { name: 'Envoyer au client' }),
		)
		expect(actions.onSend).not.toHaveBeenCalled()
		await userEvent.click(
			screen.getByRole('button', { name: 'Confirmer l’envoi' }),
		)

		expect(actions.onSend).toHaveBeenCalledTimes(1)
	})

	it('locks the draft while it is pending', () => {
		renderDraft({ pending: true })

		for (const name of [
			'Modifier',
			'Supprimer le brouillon',
			'Envoyer au client',
		]) {
			expect(
				(screen.getByRole('button', { name }) as HTMLButtonElement).disabled,
			).toBe(true)
		}
	})

	it('says nobody approved yet', () => {
		renderDraft()

		expect(screen.getByText('Pas encore d’approbation')).toBeTruthy()
		expect(
			screen
				.getByRole('button', { name: 'Approuver' })
				.getAttribute('aria-pressed'),
		).toBe('false')
	})

	it('names the approvers and shows the approval as given', () => {
		renderDraft({
			reviews: {
				approvedBy: ['Alix Martin', 'Jo Durand'],
				approvedByMe: true,
				comments: [],
			},
		})

		expect(screen.getByText('Approuvé par Alix Martin, Jo Durand')).toBeTruthy()
		expect(
			screen
				.getByRole('button', { name: 'Approuvé' })
				.getAttribute('aria-pressed'),
		).toBe('true')
	})

	it('toggles the approval', async () => {
		const actions = renderDraft()

		await userEvent.click(screen.getByRole('button', { name: 'Approuver' }))

		expect(actions.onToggleApproval).toHaveBeenCalledTimes(1)
	})

	it('lists the opinions given on the draft', () => {
		renderDraft({
			reviews: {
				approvedBy: [],
				approvedByMe: false,
				comments: [
					{
						id: 'review-1',
						memberName: 'Jo Durand',
						memberHue: 40,
						kind: 'comment',
						body: 'Plus court ?',
					},
				],
			},
		})

		expect(screen.getByText('Jo Durand')).toBeTruthy()
		expect(screen.getByText(': Plus court ?')).toBeTruthy()
	})

	it('publishes an opinion and closes the form', async () => {
		const actions = renderDraft()

		await userEvent.click(
			screen.getByRole('button', { name: 'Donner un avis' }),
		)
		const publish = screen.getByRole('button', { name: 'Publier' })
		expect((publish as HTMLButtonElement).disabled).toBe(true)
		await userEvent.type(
			screen.getByLabelText('Avis sur le brouillon'),
			'Ok pour moi',
		)
		await userEvent.click(publish)

		expect(actions.onComment).toHaveBeenCalledWith('Ok pour moi')
		expect(screen.queryByLabelText('Avis sur le brouillon')).toBeNull()
	})
})
