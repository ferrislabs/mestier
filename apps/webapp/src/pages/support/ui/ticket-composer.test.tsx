import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeAll, describe, expect, it, vi } from 'vitest'
import { installRadixPolyfills } from '#/pages/support/feature/support-test-kit'
import {
	TicketComposer,
	type TicketComposerProps,
} from '#/pages/support/ui/ticket-composer'

beforeAll(installRadixPolyfills)

function renderComposer(overrides: Partial<TicketComposerProps> = {}) {
	const props: TicketComposerProps = {
		mode: 'reply',
		onModeChange: vi.fn(),
		text: '',
		onTextChange: vi.fn(),
		nextStatus: 'waiting_customer',
		onNextStatusChange: vi.fn(),
		editingDraft: false,
		onCancelDraft: vi.fn(),
		onSend: vi.fn(),
		onSaveDraft: vi.fn(),
		onAddNote: vi.fn(),
		isSubmitting: false,
		composerError: null,
		...overrides,
	}
	render(<TicketComposer {...props} />)
	return props
}

describe('TicketComposer — réponse', () => {
	it('labels the field for a reply and invites to write to the customer', () => {
		renderComposer()

		expect(
			screen
				.getByRole('textbox', { name: 'Réponse au client' })
				.getAttribute('placeholder'),
		).toBe('Votre réponse au client…')
	})

	it('reports the typed text', async () => {
		const props = renderComposer()

		await userEvent.type(screen.getByRole('textbox'), 'a')

		expect(props.onTextChange).toHaveBeenCalledWith('a')
	})

	it('disables saving and sending while the text is blank', () => {
		renderComposer({ text: '  ' })

		expect(
			(
				screen.getByRole('button', {
					name: 'Enregistrer en brouillon',
				}) as HTMLButtonElement
			).disabled,
		).toBe(true)
		expect(
			(
				screen.getByRole('button', {
					name: 'Envoyer au client',
				}) as HTMLButtonElement
			).disabled,
		).toBe(true)
	})

	it('disables saving and sending while a submission is running', () => {
		renderComposer({ text: 'Bonjour', isSubmitting: true })

		expect(
			(
				screen.getByRole('button', {
					name: 'Envoyer au client',
				}) as HTMLButtonElement
			).disabled,
		).toBe(true)
	})

	it('needs two presses to send', async () => {
		const props = renderComposer({ text: 'Bonjour' })

		await userEvent.click(
			screen.getByRole('button', { name: 'Envoyer au client' }),
		)
		expect(props.onSend).not.toHaveBeenCalled()

		await userEvent.click(
			screen.getByRole('button', { name: 'Confirmer l’envoi' }),
		)
		expect(props.onSend).toHaveBeenCalledTimes(1)
	})

	it('announces the pending confirmation', async () => {
		renderComposer({ text: 'Bonjour' })

		await userEvent.click(
			screen.getByRole('button', { name: 'Envoyer au client' }),
		)

		expect(
			screen.getByText('Cliquez à nouveau pour envoyer le message au client.'),
		).toBeTruthy()
	})

	it('disarms the confirmation when the focus leaves the button', async () => {
		renderComposer({ text: 'Bonjour' })

		await userEvent.click(
			screen.getByRole('button', { name: 'Envoyer au client' }),
		)
		await userEvent.click(screen.getByRole('textbox'))

		expect(
			screen.getByRole('button', { name: 'Envoyer au client' }),
		).toBeTruthy()
	})

	it('arms the confirmation with Ctrl+Enter', async () => {
		renderComposer({ text: 'Bonjour' })

		await userEvent.type(
			screen.getByRole('textbox'),
			'{Control>}{Enter}{/Control}',
		)

		expect(
			screen.getByRole('button', { name: 'Confirmer l’envoi' }),
		).toBeTruthy()
	})

	it('saves a draft', async () => {
		const props = renderComposer({ text: 'Bonjour' })

		await userEvent.click(
			screen.getByRole('button', { name: 'Enregistrer en brouillon' }),
		)

		expect(props.onSaveDraft).toHaveBeenCalledTimes(1)
	})

	it('changes the status applied after the send', async () => {
		const props = renderComposer()

		await userEvent.click(screen.getByLabelText('Statut après l’envoi'))
		await userEvent.click(await screen.findByRole('option', { name: 'Fermé' }))

		expect(props.onNextStatusChange).toHaveBeenCalledWith('resolved')
	})

	it('shows the error', () => {
		renderComposer({ composerError: 'Le serveur a refusé le message.' })

		expect(screen.getByText('Le serveur a refusé le message.')).toBeTruthy()
	})
})

describe('TicketComposer — note interne', () => {
	it('switches between the two modes', async () => {
		const props = renderComposer()

		await userEvent.click(screen.getByRole('radio', { name: 'Note interne' }))

		expect(props.onModeChange).toHaveBeenCalledWith('note')
	})

	it('labels the field as an internal note and hides the reply controls', () => {
		renderComposer({ mode: 'note', text: 'Rappeler lundi' })

		expect(screen.getByRole('textbox', { name: 'Note interne' })).toBeTruthy()
		expect(
			screen.getByText('Visible uniquement par l’équipe support.'),
		).toBeTruthy()
		expect(screen.queryByLabelText('Statut après l’envoi')).toBeNull()
		expect(
			screen.queryByRole('button', { name: 'Envoyer au client' }),
		).toBeNull()
	})

	it('adds the note at once', async () => {
		const props = renderComposer({ mode: 'note', text: 'Rappeler lundi' })

		await userEvent.click(
			screen.getByRole('button', { name: 'Ajouter la note' }),
		)

		expect(props.onAddNote).toHaveBeenCalledTimes(1)
	})

	it('adds the note with Ctrl+Enter', async () => {
		const props = renderComposer({ mode: 'note', text: 'Rappeler lundi' })

		await userEvent.type(
			screen.getByRole('textbox'),
			'{Control>}{Enter}{/Control}',
		)

		expect(props.onAddNote).toHaveBeenCalledTimes(1)
	})
})

describe('TicketComposer — brouillon en cours de modification', () => {
	it('explains the edit and offers to cancel it', async () => {
		const props = renderComposer({ editingDraft: true, text: 'Brouillon' })

		expect(screen.getByText(/Modification d’un brouillon/)).toBeTruthy()
		expect(screen.queryByRole('radio')).toBeNull()
		await userEvent.click(screen.getByRole('button', { name: 'Annuler' }))

		expect(props.onCancelDraft).toHaveBeenCalledTimes(1)
	})

	it('renames the save button', () => {
		renderComposer({ editingDraft: true, text: 'Brouillon' })

		expect(
			screen.getByRole('button', { name: 'Mettre à jour le brouillon' }),
		).toBeTruthy()
	})
})
