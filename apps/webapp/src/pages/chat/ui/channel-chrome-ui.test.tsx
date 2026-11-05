import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { TooltipProvider } from '#/components/ui/tooltip'
import { ChannelNotFoundUI } from './channel-not-found-ui'
import { ChannelSearchBar } from './channel-search-bar'
import { ChatIndexUI } from './chat-index-ui'
import { MemberList } from './member-list'
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

describe('MemberList', () => {
	it('counts the members and the people online', () => {
		render(
			<MemberList
				members={[
					{ id: 'm-1', name: 'Alix Martin', hue: 10 },
					{ id: 'm-2', name: 'Jo Durand', hue: 80 },
				]}
				onlineCount={1}
			/>,
		)

		expect(screen.getByText('Alix Martin')).toBeDefined()
		expect(screen.getByText('Jo Durand')).toBeDefined()
		expect(screen.getByText('· 2')).toBeDefined()
		expect(screen.getByText('1 en ligne')).toBeDefined()
	})

	it('hides the online line when nobody is online', () => {
		render(<MemberList members={[]} onlineCount={0} />)

		expect(screen.queryByText(/en ligne/)).toBeNull()
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
