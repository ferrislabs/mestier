import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { MessageComposerUI } from './message-composer-ui'

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
		render(<MessageComposerUI {...baseProps()} />)
		expect(
			(screen.getByLabelText('Envoyer') as HTMLButtonElement).disabled,
		).toBe(true)
	})

	it('enables send once there is text', () => {
		render(<MessageComposerUI {...baseProps()} value="hello" />)
		expect(
			(screen.getByLabelText('Envoyer') as HTMLButtonElement).disabled,
		).toBe(false)
	})

	it('calls onChange and onTyping as the user types', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		render(<MessageComposerUI {...props} />)

		await user.type(screen.getByPlaceholderText('Écrire un message…'), 'h')

		expect(props.onChange).toHaveBeenCalled()
		expect(props.onTyping).toHaveBeenCalled()
	})

	it('sends on click when there is content', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		render(<MessageComposerUI {...props} value="hello" />)

		await user.click(screen.getByLabelText('Envoyer'))

		expect(props.onSend).toHaveBeenCalledTimes(1)
	})

	it('sends on Enter without Shift, not on Shift+Enter', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		render(<MessageComposerUI {...props} value="hello" />)
		const textarea = screen.getByPlaceholderText('Écrire un message…')

		await user.type(textarea, '{Shift>}{Enter}{/Shift}')
		expect(props.onSend).not.toHaveBeenCalled()

		await user.type(textarea, '{Enter}')
		expect(props.onSend).toHaveBeenCalledTimes(1)
	})

	it('disables send while an attachment is still uploading', () => {
		render(
			<MessageComposerUI
				{...baseProps()}
				value="hello"
				pendingAttachments={[
					{ id: 'a1', filename: 'photo.png', status: 'uploading' },
				]}
			/>,
		)
		expect(
			(screen.getByLabelText('Envoyer') as HTMLButtonElement).disabled,
		).toBe(true)
	})

	it('lets the user remove a pending attachment', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		render(
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
