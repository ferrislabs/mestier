import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { createRef } from 'react'
import { describe, expect, it, vi } from 'vitest'
import { TooltipProvider } from '#/components/ui/tooltip'
import type { DisplayMessage } from '#/pages/chat/lib/messages-reducer'
import { MessageListUI, type MessageListUIProps } from './message-list-ui'

function renderList(ui: React.ReactElement) {
	return render(<TooltipProvider>{ui}</TooltipProvider>)
}

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
		authors: new Map([
			['me', { name: 'Baptiste Parmantier', hue: 10 }],
			['other', { name: 'Karim Benali', hue: 200 }],
		]),
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
		renderList(<MessageListUI {...baseProps()} />)
		expect(screen.getByText(/Aucun message/)).toBeDefined()
	})

	it('shows an error state', () => {
		renderList(
			<MessageListUI {...baseProps({ initialError: 'load-failed' })} />,
		)
		expect(screen.getByText(/Impossible de charger les messages/)).toBeDefined()
	})

	it('shows each author by their directory name', () => {
		const messages = [
			displayMessage({ message: { id: 'm-1', author_user_id: 'me' } }),
			displayMessage({ message: { id: 'm-2', author_user_id: 'other' } }),
		]
		renderList(<MessageListUI {...baseProps({ messages })} />)

		expect(screen.getByText('Baptiste Parmantier')).toBeDefined()
		expect(screen.getByText('Karim Benali')).toBeDefined()
	})

	it('names an author missing from the directory instead of hiding them', () => {
		const messages = [
			displayMessage({ message: { id: 'm-1', author_user_id: 'gone' } }),
		]
		renderList(<MessageListUI {...baseProps({ messages })} />)

		expect(screen.getByText('Ancien membre')).toBeDefined()
	})

	it('highlights a message that mentions the current user', () => {
		const messages = [
			displayMessage({
				message: {
					id: 'm-1',
					author_user_id: 'other',
					content: 'salut @Baptiste Parmantier',
					mention_user_ids: ['me'],
				},
			}),
			displayMessage({
				message: { id: 'm-2', author_user_id: 'other', content: 'rien' },
			}),
		]
		renderList(<MessageListUI {...baseProps({ messages })} />)

		const [mentioning, other] = screen.getAllByRole('article')
		expect(mentioning?.className).toContain('bg-highlight/10')
		expect(other?.className).not.toContain('bg-highlight/10')
	})

	it('shows a retry action on a failed send', async () => {
		const user = userEvent.setup()
		const onRetry = vi.fn()
		const messages = [displayMessage({ status: 'failed', tempId: 'temp-1' })]
		renderList(<MessageListUI {...baseProps({ messages, onRetry })} />)

		await user.click(screen.getByText(/réessayer/))

		expect(onRetry).toHaveBeenCalledWith('temp-1')
	})

	it('offers edit and delete in the menu of own sent messages only', async () => {
		const user = userEvent.setup()
		const onStartEdit = vi.fn()
		const onDelete = vi.fn()
		const messages = [
			displayMessage({
				message: { id: 'm-1', author_user_id: 'me', content: 'hi' },
			}),
			displayMessage({ message: { id: 'm-2', author_user_id: 'other' } }),
			displayMessage({
				message: { id: 'm-3', author_user_id: 'me' },
				status: 'sending',
			}),
		]
		renderList(
			<MessageListUI {...baseProps({ messages, onStartEdit, onDelete })} />,
		)

		const menus = screen.getAllByLabelText('Plus d’actions')
		expect(menus).toHaveLength(2)

		await user.click(menus[1])
		expect(screen.queryByText('Modifier le message')).toBeNull()
		await user.keyboard('{Escape}')

		await user.click(menus[0])
		await user.click(await screen.findByText('Modifier le message'))
		expect(onStartEdit).toHaveBeenCalledWith('m-1', 'hi')

		await user.click(screen.getAllByLabelText('Plus d’actions')[0])
		await user.click(await screen.findByText('Supprimer le message'))
		expect(onDelete).toHaveBeenCalledWith('m-1')
	})

	it('groups consecutive messages of one author under one header', () => {
		const messages = [
			displayMessage({
				message: { id: 'm-1', created_at: '2026-01-01T09:00:00Z' },
			}),
			displayMessage({
				message: { id: 'm-2', created_at: '2026-01-01T09:02:00Z' },
			}),
		]
		renderList(<MessageListUI {...baseProps({ messages })} />)

		expect(screen.getAllByText('Baptiste Parmantier')).toHaveLength(1)
	})

	it('opens each day with a separator', () => {
		const messages = [
			displayMessage({
				message: { id: 'm-1', created_at: '2020-01-01T09:00:00Z' },
			}),
			displayMessage({
				message: { id: 'm-2', created_at: '2020-01-02T09:00:00Z' },
			}),
		]
		renderList(<MessageListUI {...baseProps({ messages })} />)

		expect(screen.getAllByRole('heading', { level: 2 })).toHaveLength(2)
	})

	it('marks the log with the given label', () => {
		renderList(
			<MessageListUI
				{...baseProps({
					messages: [displayMessage()],
					label: 'Messages de #x',
				})}
			/>,
		)

		expect(screen.getByRole('log', { name: 'Messages de #x' })).toBeDefined()
	})

	it('shows a custom empty label', () => {
		renderList(<MessageListUI {...baseProps({ emptyLabel: 'Rien ici' })} />)

		expect(screen.getByText('Rien ici')).toBeDefined()
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
		renderList(
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
		renderList(
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
		renderList(<MessageListUI {...baseProps({ messages })} />)

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
		renderList(<MessageListUI {...baseProps({ messages, onToggleReaction })} />)

		await user.click(screen.getByLabelText('Retirer la réaction 👍'))

		expect(onToggleReaction).toHaveBeenCalledWith('m-1', '👍', true)
	})

	it('adds a new reaction from the emoji picker of the message toolbar', async () => {
		const user = userEvent.setup()
		const onToggleReaction = vi.fn()
		const messages = [displayMessage({ message: { id: 'm-1' } })]
		renderList(<MessageListUI {...baseProps({ messages, onToggleReaction })} />)

		await user.click(screen.getByLabelText('Ajouter une réaction'))
		await user.click(await screen.findByLabelText('Réagir avec 🎉'))

		expect(onToggleReaction).toHaveBeenCalledWith('m-1', '🎉', false)
	})

	it('removes the reaction when picking an emoji already used by me', async () => {
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
		renderList(<MessageListUI {...baseProps({ messages, onToggleReaction })} />)

		await user.click(screen.getByLabelText('Ajouter une réaction'))
		await user.click(await screen.findByLabelText('Réagir avec 👍'))

		expect(onToggleReaction).toHaveBeenCalledWith('m-1', '👍', true)
	})
})

describe('MessageListUI — threads', () => {
	it('offers to reply in a thread when none exists yet', async () => {
		const user = userEvent.setup()
		const onOpenThread = vi.fn()
		const messages = [displayMessage({ message: { id: 'm-1' } })]
		renderList(<MessageListUI {...baseProps({ messages, onOpenThread })} />)

		await user.click(screen.getByLabelText('Répondre dans un fil'))

		expect(onOpenThread).toHaveBeenCalledWith('m-1')
	})

	it('shows the thread is already open once one exists', () => {
		const messages = [displayMessage({ message: { id: 'm-1' } })]
		renderList(
			<MessageListUI
				{...baseProps({ messages, threadedMessageIds: new Set(['m-1']) })}
			/>,
		)

		expect(screen.getByText('Fil de discussion')).toBeDefined()
		expect(screen.getByLabelText('Ouvrir le fil')).toBeDefined()
	})

	it('opens the thread from its indicator', async () => {
		const user = userEvent.setup()
		const onOpenThread = vi.fn()
		const messages = [displayMessage({ message: { id: 'm-1' } })]
		renderList(
			<MessageListUI
				{...baseProps({
					messages,
					onOpenThread,
					threadedMessageIds: new Set(['m-1']),
				})}
			/>,
		)

		await user.click(screen.getByText('Fil de discussion'))

		expect(onOpenThread).toHaveBeenCalledWith('m-1')
	})
})

describe('MessageListUI — pins and saves', () => {
	it('toggles the pin and the saved flag from the message menu', async () => {
		const user = userEvent.setup()
		const onTogglePin = vi.fn()
		const onToggleSaved = vi.fn()
		const messages = [displayMessage({ message: { id: 'm-1' } })]
		renderList(
			<MessageListUI
				{...baseProps({ messages, onTogglePin, onToggleSaved })}
			/>,
		)

		await user.click(screen.getByLabelText('Plus d’actions'))
		await user.click(await screen.findByText('Épingler le message'))
		expect(onTogglePin).toHaveBeenCalledWith('m-1')

		await user.click(screen.getByLabelText('Plus d’actions'))
		await user.click(await screen.findByText('Enregistrer pour plus tard'))
		expect(onToggleSaved).toHaveBeenCalledWith('m-1')
	})

	it('offers to unpin a pinned message', async () => {
		const user = userEvent.setup()
		const messages = [displayMessage({ message: { id: 'm-1' } })]
		renderList(
			<MessageListUI
				{...baseProps({ messages, pinnedMessageIds: new Set(['m-1']) })}
			/>,
		)

		await user.click(screen.getByLabelText('Plus d’actions'))

		expect(await screen.findByText('Désépingler le message')).toBeDefined()
	})
})
