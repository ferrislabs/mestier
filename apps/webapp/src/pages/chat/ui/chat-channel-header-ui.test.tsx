import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { TooltipProvider } from '#/components/ui/tooltip'
import {
	ChatChannelHeaderUI,
	type ChatChannelHeaderUIProps,
} from './chat-channel-header-ui'

function renderHeader(overrides: Partial<ChatChannelHeaderUIProps> = {}) {
	const props: ChatChannelHeaderUIProps = {
		name: 'chantier-dupont',
		topic: 'Suivi du chantier Dupont',
		isLoading: false,
		searchOpen: false,
		onSearchOpenChange: vi.fn(),
		membersOpen: true,
		onMembersOpenChange: vi.fn(),
		pinned: <span>épinglés</span>,
		onOpenAdmin: vi.fn(),
		...overrides,
	}
	render(
		<TooltipProvider>
			<ChatChannelHeaderUI {...props} />
		</TooltipProvider>,
	)
	return props
}

describe('ChatChannelHeaderUI', () => {
	it('shows the channel name and topic', () => {
		renderHeader()

		expect(
			screen.getByRole('heading', { name: 'chantier-dupont' }),
		).toBeDefined()
		expect(screen.getByText('Suivi du chantier Dupont')).toBeDefined()
	})

	it('shows no topic when the channel has none', () => {
		renderHeader({ topic: null })

		expect(screen.queryByText('Suivi du chantier Dupont')).toBeNull()
	})

	it('shows a skeleton instead of the title while loading', () => {
		renderHeader({ isLoading: true })

		expect(screen.queryByRole('heading')).toBeNull()
	})

	it('toggles the search', async () => {
		const user = userEvent.setup()
		const props = renderHeader()

		await user.click(screen.getByLabelText('Rechercher dans le canal'))

		expect(props.onSearchOpenChange).toHaveBeenCalledWith(true)
	})

	it('reports the search as pressed when open', () => {
		renderHeader({ searchOpen: true })

		expect(
			screen
				.getByLabelText('Rechercher dans le canal')
				.getAttribute('aria-pressed'),
		).toBe('true')
	})

	it('toggles the member list and names the action after its state', async () => {
		const user = userEvent.setup()
		const props = renderHeader({ membersOpen: true })

		await user.click(screen.getByLabelText('Masquer la liste des membres'))

		expect(props.onMembersOpenChange).toHaveBeenCalledWith(false)
	})

	it('opens the channel settings', async () => {
		const user = userEvent.setup()
		const props = renderHeader()

		await user.click(screen.getByLabelText('Paramètres du canal'))

		expect(props.onOpenAdmin).toHaveBeenCalledTimes(1)
	})

	it('renders the pinned slot', () => {
		renderHeader()

		expect(screen.getByText('épinglés')).toBeDefined()
	})
})
