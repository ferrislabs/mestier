import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { TooltipProvider } from '#/components/ui/tooltip'
import { MessageComposerUI } from './message-composer-ui'

function renderComposer(ui: React.ReactElement) {
	return render(<TooltipProvider>{ui}</TooltipProvider>)
}

function baseProps() {
	return {
		value: '',
		onChange: vi.fn(),
		onSend: vi.fn(),
		onTyping: vi.fn(),
		pendingAttachments: [],
		onAttachFiles: vi.fn(),
		onRemoveAttachment: vi.fn(),
	}
}

describe('MessageComposerUI', () => {
	it('disables send with empty content and no attachments', () => {
		renderComposer(<MessageComposerUI {...baseProps()} />)
		expect(
			(screen.getByLabelText('Envoyer le message') as HTMLButtonElement)
				.disabled,
		).toBe(true)
	})

	it('enables send once there is text', () => {
		renderComposer(<MessageComposerUI {...baseProps()} value="hello" />)
		expect(
			(screen.getByLabelText('Envoyer le message') as HTMLButtonElement)
				.disabled,
		).toBe(false)
	})

	it('calls onChange and onTyping as the user types', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		renderComposer(<MessageComposerUI {...props} />)

		await user.type(screen.getByPlaceholderText('Écrire un message…'), 'h')

		expect(props.onChange).toHaveBeenCalled()
		expect(props.onTyping).toHaveBeenCalled()
	})

	it('sends on click when there is content', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		renderComposer(<MessageComposerUI {...props} value="hello" />)

		await user.click(screen.getByLabelText('Envoyer le message'))

		expect(props.onSend).toHaveBeenCalledTimes(1)
	})

	it('sends on Enter without Shift, not on Shift+Enter', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		renderComposer(<MessageComposerUI {...props} value="hello" />)
		const textarea = screen.getByPlaceholderText('Écrire un message…')

		await user.type(textarea, '{Shift>}{Enter}{/Shift}')
		expect(props.onSend).not.toHaveBeenCalled()

		await user.type(textarea, '{Enter}')
		expect(props.onSend).toHaveBeenCalledTimes(1)
	})

	it('disables send while an attachment is still uploading', () => {
		renderComposer(
			<MessageComposerUI
				{...baseProps()}
				value="hello"
				pendingAttachments={[
					{ id: 'a1', filename: 'photo.png', status: 'uploading' },
				]}
			/>,
		)
		expect(
			(screen.getByLabelText('Envoyer le message') as HTMLButtonElement)
				.disabled,
		).toBe(true)
	})

	it('appends a mention to the draft from the mention button', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		renderComposer(<MessageComposerUI {...props} value="salut" />)

		await user.click(screen.getByLabelText('Mentionner un collègue'))

		expect(props.onChange).toHaveBeenCalledWith('salut @')
	})

	it('appends the chosen emoji to the draft', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		renderComposer(<MessageComposerUI {...props} value="ok " />)

		await user.click(screen.getByLabelText('Insérer un emoji'))
		await user.click(screen.getByLabelText('Réagir avec 🎉'))

		expect(props.onChange).toHaveBeenCalledWith('ok 🎉')
	})

	it('labels the field and takes a custom placeholder', () => {
		renderComposer(
			<MessageComposerUI
				{...baseProps()}
				label="Message dans #chantier"
				placeholder="Écrire dans #chantier"
			/>,
		)

		expect(screen.getByLabelText('Message dans #chantier')).toBeDefined()
		expect(screen.getByPlaceholderText('Écrire dans #chantier')).toBeDefined()
	})

	it('lets the user remove a pending attachment', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		renderComposer(
			<MessageComposerUI
				{...props}
				pendingAttachments={[
					{ id: 'a1', filename: 'photo.png', status: 'ready' },
				]}
			/>,
		)

		await user.click(screen.getByLabelText('Retirer photo.png'))

		expect(props.onRemoveAttachment).toHaveBeenCalledWith('a1')
	})
})
