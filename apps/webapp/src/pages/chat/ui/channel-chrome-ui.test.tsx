import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { TooltipProvider } from '#/components/ui/tooltip'
import { ChannelMembersDialogUI } from './channel-members-dialog-ui'
import { ChannelNotFoundUI } from './channel-not-found-ui'
import { ChannelSearchBar } from './channel-search-bar'
import { ChatChannelHeaderUI } from './chat-channel-header-ui'
import { ChatIndexUI } from './chat-index-ui'
import { PinnedMessagesPopover } from './pinned-messages-popover'
import { ThreadPanelHeaderUI } from './thread-panel-header-ui'

function withTooltips(ui: React.ReactElement) {
	return render(<TooltipProvider>{ui}</TooltipProvider>)
}

describe('ChannelNotFoundUI', () => {
	it('explains the channel is gone and offers the way back', async () => {
		const user = userEvent.setup()
		const onBack = vi.fn()
		render(<ChannelNotFoundUI onBack={onBack} />)

		expect(screen.getByText('Ce canal est introuvable')).toBeDefined()
		await user.click(screen.getByRole('button', { name: /Retour aux canaux/ }))

		expect(onBack).toHaveBeenCalledTimes(1)
	})
})

describe('ChannelSearchBar', () => {
	it('focuses the field, reports typing and the result count', async () => {
		const user = userEvent.setup()
		const onQueryChange = vi.fn()
		withTooltips(
			<ChannelSearchBar
				query=""
				label="Rechercher dans #x"
				resultLabel="2 résultats"
				onQueryChange={onQueryChange}
				onClose={vi.fn()}
			/>,
		)

		const field = screen.getByLabelText('Rechercher dans #x')
		expect(document.activeElement).toBe(field)
		await user.type(field, 'd')

		expect(onQueryChange).toHaveBeenCalledWith('d')
		expect(screen.getByText('2 résultats')).toBeDefined()
	})

	it('closes on Escape and from its button', async () => {
		const user = userEvent.setup()
		const onClose = vi.fn()
		withTooltips(
			<ChannelSearchBar
				query=""
				label="Rechercher"
				resultLabel=""
				onQueryChange={vi.fn()}
				onClose={onClose}
			/>,
		)

		await user.keyboard('{Escape}')
		await user.click(screen.getByLabelText('Fermer la recherche'))

		expect(onClose).toHaveBeenCalledTimes(2)
	})
})

describe('PinnedMessagesPopover', () => {
	it('counts the pinned messages in its label and lists them', async () => {
		const user = userEvent.setup()
		withTooltips(
			<PinnedMessagesPopover
				items={[
					{
						id: 'm-1',
						authorName: 'Membre',
						whenLabel: 'Hier à 09:00',
						text: 'Code du portail : 4821',
					},
				]}
			/>,
		)

		await user.click(screen.getByLabelText('Messages épinglés (1)'))

		expect(await screen.findByText('Code du portail : 4821')).toBeDefined()
		expect(screen.getByText('Hier à 09:00')).toBeDefined()
	})

	it('says nothing is pinned when the list is empty', async () => {
		const user = userEvent.setup()
		withTooltips(<PinnedMessagesPopover items={[]} />)

		await user.click(screen.getByLabelText('Messages épinglés (0)'))

		expect(
			await screen.findByText('Aucun message épinglé dans ce canal.'),
		).toBeDefined()
	})
})

describe('ThreadPanelHeaderUI', () => {
	it.each([
		[0, 'Aucune réponse'],
		[1, '1 réponse'],
		[3, '3 réponses'],
	])('writes %i replies as "%s"', (count, label) => {
		withTooltips(<ThreadPanelHeaderUI replyCount={count} onClose={vi.fn()} />)

		expect(screen.getByText(label)).toBeDefined()
	})

	it('closes the thread', async () => {
		const user = userEvent.setup()
		const onClose = vi.fn()
		withTooltips(<ThreadPanelHeaderUI replyCount={2} onClose={onClose} />)

		await user.click(screen.getByLabelText('Fermer le fil'))

		expect(onClose).toHaveBeenCalledTimes(1)
	})
})

describe('ChatIndexUI', () => {
	it('invites to create the first channel when there is none', () => {
		render(<ChatIndexUI state="empty" />)

		expect(screen.getByText('Aucun canal pour le moment')).toBeDefined()
	})

	it('renders nothing while loading or redirecting', () => {
		const { container, rerender } = render(<ChatIndexUI state="loading" />)
		expect(container.innerHTML).toBe('')

		rerender(<ChatIndexUI state="redirecting" />)
		expect(container.innerHTML).toBe('')
	})
})

describe('ChatChannelHeaderUI — direct message', () => {
	const baseProps = {
		isLoading: false,
		searchOpen: false,
		onSearchOpenChange: vi.fn(),
		pinned: null,
		onOpenAdmin: vi.fn(),
	}

	it('names the person with their presence and offers no channel settings', () => {
		withTooltips(
			<ChatChannelHeaderUI
				{...baseProps}
				name="direct"
				topic="ignored"
				peer={{ name: 'Karim Benali', hue: 200, presence: 'DND' }}
			/>,
		)

		expect(screen.getByRole('heading', { name: 'Karim Benali' })).toBeDefined()
		expect(screen.getAllByText('Absent').length).toBeGreaterThan(0)
		expect(screen.queryByText('ignored')).toBeNull()
		expect(screen.queryByLabelText('Paramètres du canal')).toBeNull()
	})

	it('shows the first three members as an avatar group that opens the list', async () => {
		const user = userEvent.setup()
		const onOpenMembers = vi.fn()
		withTooltips(
			<ChatChannelHeaderUI
				{...baseProps}
				name="general"
				onOpenMembers={onOpenMembers}
				members={[
					{ id: 'u-1', name: 'Alix Martin', hue: 10 },
					{ id: 'u-2', name: 'Jo Durand', hue: 80 },
					{ id: 'u-3', name: 'Sam Roux', hue: 120 },
					{ id: 'u-4', name: 'Lou Petit', hue: 200 },
					{ id: 'u-5', name: 'Eve Blanc', hue: 300 },
				]}
			/>,
		)

		expect(screen.getByText('+2')).toBeDefined()
		expect(screen.queryByText('LP')).toBeNull()
		await user.click(
			screen.getByRole('button', { name: 'Voir les 5 membres du canal' }),
		)

		expect(onOpenMembers).toHaveBeenCalledTimes(1)
	})

	it('shows no member list button on a direct message', () => {
		withTooltips(
			<ChatChannelHeaderUI
				{...baseProps}
				name="direct"
				peer={{ name: 'Karim Benali', hue: 200, presence: 'ONLINE' }}
				onOpenMembers={vi.fn()}
				members={[{ id: 'u-1', name: 'Alix Martin', hue: 10 }]}
			/>,
		)

		expect(screen.queryByRole('button', { name: /membre/ })).toBeNull()
	})

	it('keeps a channel s icon for its type', () => {
		withTooltips(
			<ChatChannelHeaderUI
				{...baseProps}
				name="annonces"
				channelType="ANNOUNCEMENT"
			/>,
		)

		expect(screen.getByRole('heading', { name: 'annonces' })).toBeDefined()
	})

	it('keeps the settings button on a regular channel', () => {
		withTooltips(<ChatChannelHeaderUI {...baseProps} name="general" />)

		expect(screen.getByLabelText('Paramètres du canal')).toBeDefined()
	})
})

describe('ChannelMembersDialogUI', () => {
	const members = [
		{ id: 'm-1', name: 'Alix Martin', hue: 10, presence: 'ONLINE' as const },
		{ id: 'm-2', name: 'Jo Durand', hue: 80, presence: 'OFFLINE' as const },
		{ id: 'm-3', name: 'Sam Roux', hue: 120, presence: 'DND' as const },
	]

	it('lists every member grouped by presence and marks the current user', () => {
		render(
			<ChannelMembersDialogUI
				open
				onOpenChange={vi.fn()}
				channelLabel="#chantier-dupont"
				members={members}
				currentUserId="m-1"
			/>,
		)

		expect(screen.getByText('· 3')).toBeDefined()
		expect(screen.getByText('#chantier-dupont')).toBeDefined()
		expect(screen.getByLabelText('En ligne, 1')).toBeDefined()
		expect(screen.getByLabelText('Absent, 1')).toBeDefined()
		expect(screen.getByLabelText('Hors ligne, 1')).toBeDefined()
		expect(screen.getByText('Sam Roux')).toBeDefined()
		expect(screen.getByText('(vous)')).toBeDefined()
	})

	it('omits a presence group nobody belongs to', () => {
		render(
			<ChannelMembersDialogUI
				open
				onOpenChange={vi.fn()}
				channelLabel="#general"
				members={[members[0] as (typeof members)[number]]}
				currentUserId={null}
			/>,
		)

		expect(screen.queryByLabelText(/Hors ligne/)).toBeNull()
	})

	it('renders nothing when closed', () => {
		render(
			<ChannelMembersDialogUI
				open={false}
				onOpenChange={vi.fn()}
				channelLabel="#general"
				members={members}
				currentUserId={null}
			/>,
		)

		expect(screen.queryByText('Alix Martin')).toBeNull()
	})
})
