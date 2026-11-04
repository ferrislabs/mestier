import { fireEvent, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type {
	SupportComposerTabVM,
	SupportComposerUIProps,
} from '#/pages/support/types'
import { SupportComposerUI } from '#/pages/support/ui/support-composer-ui'

const TABS: SupportComposerTabVM[] = [
	{
		id: 'reply',
		label: 'Réponse',
		actionLabel: 'Envoyer la réponse',
		hint: 'Le client recevra cette réponse par e-mail.',
	},
	{
		id: 'draft',
		label: 'Brouillon',
		actionLabel: 'Enregistrer le brouillon',
		hint: 'Le brouillon reste visible de l’équipe uniquement.',
	},
	{
		id: 'note',
		label: 'Note interne',
		actionLabel: 'Ajouter la note',
		hint: 'La note n’est jamais visible par le client.',
	},
]

function renderComposer(overrides: Partial<SupportComposerUIProps> = {}) {
	const props: SupportComposerUIProps = {
		tabs: TABS,
		activeTab: 'reply',
		body: 'Nous passons demain.',
		isSubmitting: false,
		error: null,
		onTabChange: vi.fn(),
		onBodyChange: vi.fn(),
		onSubmit: vi.fn(),
		...overrides,
	}
	render(<SupportComposerUI {...props} />)
	return props
}

function submitButton(name: string) {
	return screen.getByRole('button', { name }) as HTMLButtonElement
}

describe('SupportComposerUI', () => {
	it('renders one tab per entry of tabs, in a tablist', () => {
		renderComposer()

		expect(screen.getByRole('tablist')).toBeTruthy()
		expect(screen.getAllByRole('tab').map((tab) => tab.textContent)).toEqual([
			'Réponse',
			'Brouillon',
			'Note interne',
		])
	})

	it('marks only the active tab as selected', () => {
		renderComposer({ activeTab: 'draft' })

		expect(
			screen
				.getAllByRole('tab')
				.map((tab) => tab.getAttribute('aria-selected')),
		).toEqual(['false', 'true', 'false'])
	})

	it('follows activeTab when it changes', () => {
		renderComposer({ activeTab: 'note' })

		expect(
			screen
				.getByRole('tab', { name: 'Note interne' })
				.getAttribute('aria-selected'),
		).toBe('true')
		expect(
			screen
				.getByRole('tab', { name: 'Réponse' })
				.getAttribute('aria-selected'),
		).toBe('false')
	})

	it('calls onTabChange with the id of the clicked tab', async () => {
		const props = renderComposer()

		await userEvent.click(screen.getByRole('tab', { name: 'Note interne' }))
		await userEvent.click(screen.getByRole('tab', { name: 'Brouillon' }))

		expect(props.onTabChange).toHaveBeenNthCalledWith(1, 'note')
		expect(props.onTabChange).toHaveBeenNthCalledWith(2, 'draft')
		expect(props.onTabChange).toHaveBeenCalledTimes(2)
	})

	it('shows the hint of the active tab only', () => {
		renderComposer({ activeTab: 'draft' })

		expect(
			screen.getByText('Le brouillon reste visible de l’équipe uniquement.'),
		).toBeTruthy()
		expect(
			screen.queryByText('Le client recevra cette réponse par e-mail.'),
		).toBeNull()
		expect(
			screen.queryByText('La note n’est jamais visible par le client.'),
		).toBeNull()
	})

	it.each([
		['reply', 'Envoyer la réponse'],
		['draft', 'Enregistrer le brouillon'],
		['note', 'Ajouter la note'],
	] as const)('labels the submit button after the %s tab', (activeTab, label) => {
		renderComposer({ activeTab })

		expect(submitButton(label)).toBeTruthy()
		for (const other of TABS.filter((tab) => tab.id !== activeTab)) {
			expect(
				screen.queryByRole('button', { name: other.actionLabel }),
			).toBeNull()
		}
	})

	it('shows the body in the Message textarea and reports edits', async () => {
		const props = renderComposer({ body: '' })

		const textarea = screen.getByRole('textbox', { name: 'Message' })
		expect((textarea as HTMLTextAreaElement).value).toBe('')
		await userEvent.type(textarea, 'a')

		expect(props.onBodyChange).toHaveBeenCalledWith('a')
	})

	it('reflects the body it is given', () => {
		renderComposer({ body: 'Texte du brouillon' })

		expect(
			(screen.getByRole('textbox', { name: 'Message' }) as HTMLTextAreaElement)
				.value,
		).toBe('Texte du brouillon')
	})

	it('calls onSubmit once from a non blank body', async () => {
		const props = renderComposer()

		await userEvent.click(submitButton('Envoyer la réponse'))

		expect(props.onSubmit).toHaveBeenCalledTimes(1)
	})

	it('disables the submit button for an empty body and never submits', async () => {
		const props = renderComposer({ body: '' })

		const button = submitButton('Envoyer la réponse')
		expect(button.disabled).toBe(true)
		await userEvent.click(button)
		fireEvent.click(button)

		expect(props.onSubmit).not.toHaveBeenCalled()
	})

	it('treats a whitespace only body as blank', async () => {
		const props = renderComposer({ body: '  \n\t ' })

		const button = submitButton('Envoyer la réponse')
		expect(button.disabled).toBe(true)
		button.disabled = false
		fireEvent.click(button)

		expect(props.onSubmit).not.toHaveBeenCalled()
	})

	it('disables the submit button while submitting', async () => {
		const props = renderComposer({ isSubmitting: true })

		const button = submitButton('Envoyer la réponse')
		expect(button.disabled).toBe(true)
		button.disabled = false
		fireEvent.click(button)

		expect(props.onSubmit).not.toHaveBeenCalled()
	})

	it('shows the error text when there is an error', () => {
		renderComposer({ error: 'Envoi impossible pour le moment.' })

		expect(screen.getByText('Envoi impossible pour le moment.')).toBeTruthy()
	})

	it('shows no error text without an error', () => {
		renderComposer()

		expect(screen.queryByText('Envoi impossible pour le moment.')).toBeNull()
	})
})
