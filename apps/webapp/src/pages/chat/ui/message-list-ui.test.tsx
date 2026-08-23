import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { createRef } from 'react'
import { describe, expect, it, vi } from 'vitest'
import type { DisplayMessage } from '#/pages/chat/lib/messages-reducer'
import { MessageListUI, type MessageListUIProps } from './message-list-ui'

function displayMessage(
	overrides: {
		message?: Partial<Record<string, unknown>>
		status?: string
		tempId?: string
	} = {},
): DisplayMessage {
	const { message: messageOverrides, ...rest } = overrides
	return {
		status: 'sent',
		message: {
			id: 'm-1',
			organization_id: 'org-1',
			channel_id: 'ch-1',
			author_type: 'USER',
			author_user_id: 'me',
			author_webhook_id: null,
			content: 'hello',
			components: null,
			mention_user_ids: [],
			mention_role_ids: [],
			mention_channel_ids: [],
			mention_everyone: false,
			reactions: [],
			attachments: [],
			edited_at: null,
			created_at: '2026-01-01T00:00:00Z',
			...messageOverrides,
		},
		...rest,
	} as DisplayMessage
}

function baseProps(
	overrides: Partial<MessageListUIProps> = {},
): MessageListUIProps {
	return {
		messages: [],
		currentUserId: 'me',
		isLoadingInitial: false,
		initialError: null,
		isLoadingOlder: false,
		hasMoreOlder: false,
		editingMessageId: null,
		editDraft: '',
		threadedMessageIds: new Set(),
		scrollContainerRef: createRef(),
		onScroll: vi.fn(),
		onWheel: vi.fn(),
		onTouchMove: vi.fn(),
		onRetry: vi.fn(),
		onStartEdit: vi.fn(),
		onChangeEditDraft: vi.fn(),
		onConfirmEdit: vi.fn(),
		onCancelEdit: vi.fn(),
		onDelete: vi.fn(),
		onToggleReaction: vi.fn(),
		onOpenThread: vi.fn(),
		resolveAttachmentUrl: () => undefined,
		...overrides,
	}
}

describe('MessageListUI', () => {
	it('shows an empty state with no messages', () => {
		render(<MessageListUI {...baseProps()} />)
		expect(screen.getByText(/Aucun message/)).toBeDefined()
	})

	it('shows an error state', () => {
		render(<MessageListUI {...baseProps({ initialError: 'load-failed' })} />)
		expect(screen.getByText(/Impossible de charger les messages/)).toBeDefined()
	})

	it('renders own messages differently from others', () => {
		const messages = [
			displayMessage({ message: { id: 'm-1', author_user_id: 'me' } }),
			displayMessage({ message: { id: 'm-2', author_user_id: 'other' } }),
		]
		render(<MessageListUI {...baseProps({ messages })} />)

		expect(screen.getByText('Vous')).toBeDefined()
		expect(screen.getByText('Membre')).toBeDefined()
	})

	it('shows a retry action on a failed send', async () => {
		const user = userEvent.setup()
		const onRetry = vi.fn()
		const messages = [displayMessage({ status: 'failed', tempId: 'temp-1' })]
		render(<MessageListUI {...baseProps({ messages, onRetry })} />)

		await user.click(screen.getByText(/réessayer/))

		expect(onRetry).toHaveBeenCalledWith('temp-1')
	})

	it('shows edit and delete actions only for own sent messages', () => {
		const messages = [
			displayMessage({ message: { id: 'm-1', author_user_id: 'me' } }),
			displayMessage({ message: { id: 'm-2', author_user_id: 'other' } }),
			displayMessage({
				message: { id: 'm-3', author_user_id: 'me' },
				status: 'sending',
			}),
		]
		render(<MessageListUI {...baseProps({ messages })} />)

		expect(screen.getAllByText('Modifier')).toHaveLength(1)
		expect(screen.getAllByText('Supprimer')).toHaveLength(1)
	})

	it('enters edit mode and calls back on save', async () => {
		const user = userEvent.setup()
		const onConfirmEdit = vi.fn()
		const onChangeEditDraft = vi.fn()
		const messages = [
			displayMessage({
				message: { id: 'm-1', author_user_id: 'me', content: 'hi' },
			}),
		]
		render(
			<MessageListUI
				{...baseProps({
					messages,
					editingMessageId: 'm-1',
					editDraft: 'hi edited',
					onChangeEditDraft,
					onConfirmEdit,
				})}
			/>,
		)

		expect(screen.getByDisplayValue('hi edited')).toBeDefined()
		await user.click(screen.getByText('Enregistrer'))
		expect(onConfirmEdit).toHaveBeenCalledTimes(1)
	})

	it('shows the loading indicator while an older page is loading', () => {
		const messages = [displayMessage()]
		render(
			<MessageListUI
				{...baseProps({ messages, hasMoreOlder: true, isLoadingOlder: true })}
			/>,
		)
		// no crash, and the "scroll up" hint is replaced by the spinner (no text)
		expect(screen.queryByText(/Faites défiler/)).toBeNull()
	})
})

describe('MessageListUI — reactions', () => {
	it('shows existing reaction groups with counts', () => {
		const messages = [
			displayMessage({
				message: {
					id: 'm-1',
					reactions: [{ emoji: '👍', count: 2, user_ids: ['me', 'alice'] }],
				},
			}),
		]
		render(<MessageListUI {...baseProps({ messages })} />)

		expect(screen.getByLabelText('Retirer la réaction 👍')).toBeDefined()
		expect(screen.getByText('2')).toBeDefined()
	})

	it('toggles a reaction when clicking an existing group', async () => {
		const user = userEvent.setup()
		const onToggleReaction = vi.fn()
		const messages = [
			displayMessage({
				message: {
					id: 'm-1',
					reactions: [{ emoji: '👍', count: 1, user_ids: ['me'] }],
				},
			}),
		]
		render(<MessageListUI {...baseProps({ messages, onToggleReaction })} />)

		await user.click(screen.getByLabelText('Retirer la réaction 👍'))

		expect(onToggleReaction).toHaveBeenCalledWith('m-1', '👍', true)
	})

	it('adds a new reaction from the quick-react row', async () => {
		const user = userEvent.setup()
		const onToggleReaction = vi.fn()
		const messages = [displayMessage({ message: { id: 'm-1' } })]
		render(<MessageListUI {...baseProps({ messages, onToggleReaction })} />)

		await user.click(screen.getByLabelText('Réagir avec ❤️'))

		expect(onToggleReaction).toHaveBeenCalledWith('m-1', '❤️', false)
	})
})

describe('MessageListUI — threads', () => {
	it('offers to reply in a thread when none exists yet', async () => {
		const user = userEvent.setup()
		const onOpenThread = vi.fn()
		const messages = [displayMessage({ message: { id: 'm-1' } })]
		render(<MessageListUI {...baseProps({ messages, onOpenThread })} />)

		await user.click(screen.getByText('Répondre dans un fil'))

		expect(onOpenThread).toHaveBeenCalledWith('m-1')
	})

	it('shows the thread is already open once one exists', () => {
		const messages = [displayMessage({ message: { id: 'm-1' } })]
		render(
			<MessageListUI
				{...baseProps({ messages, threadedMessageIds: new Set(['m-1']) })}
			/>,
		)

		expect(screen.getByText('Fil de discussion')).toBeDefined()
	})
})
