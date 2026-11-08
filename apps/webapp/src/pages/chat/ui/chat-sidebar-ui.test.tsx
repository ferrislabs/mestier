import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactElement } from 'react'
import { describe, expect, it, vi } from 'vitest'
import { renderWithRouter } from '#/test/render-with-router'
import { wrapWithPermissions } from '#/test/with-permissions'
import { ChatSidebarUI, type ChatSidebarUIProps } from './chat-sidebar-ui'

/**
 * The sidebar's create-channel affordances are gated on MANAGE_CHANNELS
 * (#407), so every render needs the permissions context on top of the
 * router one. Defaults to a fully-privileged caller; the gate itself is
 * exercised by the tests that pass an explicit, narrower list.
 */
async function renderSidebar(ui: ReactElement, permissions?: string[]) {
	return renderWithRouter(
		wrapWithPermissions(ui, permissions ? { permissions } : undefined),
	)
}

function category(id: string, position = 0) {
	return {
		id,
		organization_id: 'org-1',
		name: id,
		position,
		created_at: '',
		updated_at: '',
	}
}

function channel(id: string, categoryId: string | null, position = 0) {
	return {
		id,
		organization_id: 'org-1',
		channel_type: 'TEXT' as const,
		name: id,
		topic: null,
		position,
		category_id: categoryId,
		parent_id: null,
		origin_message_id: null,
		archived: false,
		created_at: '',
		updated_at: '',
	}
}

function baseProps(
	overrides: Partial<ChatSidebarUIProps> = {},
): ChatSidebarUIProps {
	return {
		organizationSlug: 'acme',
		groups: [],
		collapsedCategoryIds: new Set(),
		onToggleCategory: vi.fn(),
		isLoading: false,
		isError: false,
		unreadCounts: new Map(),
		directMessages: [],
		onRequestNewDirectMessage: vi.fn(),
		mentionCount: 0,
		onRequestNewChannel: vi.fn(),
		onRequestNewChannelInCategory: vi.fn(),
		onOpenChannelAdmin: vi.fn(),
		...overrides,
	}
}

describe('ChatSidebarUI', () => {
	it('shows a skeleton while loading', async () => {
		await renderSidebar(<ChatSidebarUI {...baseProps({ isLoading: true })} />)
		expect(screen.getByRole('navigation')).toBeDefined()
	})

	it('shows an error message on failure', async () => {
		await renderSidebar(<ChatSidebarUI {...baseProps({ isError: true })} />)
		expect(screen.getByText('Impossible de charger les canaux.')).toBeDefined()
	})

	it('shows guidance when the organization has no channel', async () => {
		await renderSidebar(<ChatSidebarUI {...baseProps()} />)
		expect(screen.getByText(/Aucun canal pour le moment/)).toBeDefined()
	})

	it('lets a request for a new channel start from the channels heading', async () => {
		const user = userEvent.setup()
		const onRequestNewChannel = vi.fn()
		await renderSidebar(
			<ChatSidebarUI {...baseProps({ onRequestNewChannel })} />,
		)

		await user.click(
			screen.getByRole('button', { name: 'Créer un canal ou une catégorie' }),
		)

		expect(onRequestNewChannel).toHaveBeenCalled()
	})

	it('lists channels grouped by category', async () => {
		const groups = [
			{
				category: category('cat-1'),
				channels: [channel('ch-1', 'cat-1'), channel('ch-2', 'cat-1')],
			},
		]
		await renderSidebar(<ChatSidebarUI {...baseProps({ groups })} />)

		expect(screen.getByText('cat-1')).toBeDefined()
		expect(screen.getByText('ch-1')).toBeDefined()
		expect(screen.getByText('ch-2')).toBeDefined()
	})

	it('highlights the active channel', async () => {
		const groups = [
			{ category: category('cat-1'), channels: [channel('ch-1', 'cat-1')] },
		]
		await renderSidebar(
			<ChatSidebarUI {...baseProps({ groups, activeChannelId: 'ch-1' })} />,
		)

		const link = screen.getByText('ch-1').closest('a')
		expect(link?.getAttribute('data-status')).toBe('active')
	})

	it('renders uncategorized channels without a category header', async () => {
		const groups = [{ category: null, channels: [channel('ch-1', null)] }]
		await renderSidebar(<ChatSidebarUI {...baseProps({ groups })} />)

		expect(screen.getByText('ch-1')).toBeDefined()
	})

	it('collapses a category on click', async () => {
		const user = userEvent.setup()
		const onToggleCategory = vi.fn()
		const groups = [
			{ category: category('cat-1'), channels: [channel('ch-1', 'cat-1')] },
		]
		await renderSidebar(
			<ChatSidebarUI {...baseProps({ groups, onToggleCategory })} />,
		)

		await user.click(screen.getByText('cat-1'))

		expect(onToggleCategory).toHaveBeenCalledWith('cat-1', true)
	})
})

describe('ChatSidebarUI — channel creation in a category', () => {
	it('creates a channel from the plus button of its category', async () => {
		const user = userEvent.setup()
		const onRequestNewChannelInCategory = vi.fn()
		const groups = [
			{ category: category('cat-1'), channels: [channel('ch-1', 'cat-1')] },
		]
		await renderSidebar(
			<ChatSidebarUI
				{...baseProps({ groups, onRequestNewChannelInCategory })}
			/>,
		)

		await user.click(
			screen.getByRole('button', { name: 'Créer un canal dans cat-1' }),
		)

		expect(onRequestNewChannelInCategory).toHaveBeenCalledWith('cat-1')
	})
})

describe('ChatSidebarUI — per-channel management (#372)', () => {
	it('opens the channel admin dialog for that channel from the sidebar itself', async () => {
		const user = userEvent.setup()
		const onOpenChannelAdmin = vi.fn()
		const groups = [
			{ category: category('cat-1'), channels: [channel('ch-1', 'cat-1')] },
		]
		await renderSidebar(
			<ChatSidebarUI {...baseProps({ groups, onOpenChannelAdmin })} />,
		)

		await user.click(
			screen.getByRole('button', { name: 'Paramètres du canal ch-1' }),
		)

		expect(onOpenChannelAdmin).toHaveBeenCalledWith('ch-1')
	})
})

describe('ChatSidebarUI — unread and mentions', () => {
	it('shows the unread count only for channels with unread content', async () => {
		const groups = [
			{
				category: category('cat-1'),
				channels: [channel('ch-1', 'cat-1'), channel('ch-2', 'cat-1')],
			},
		]
		await renderSidebar(
			<ChatSidebarUI
				{...baseProps({ groups, unreadCounts: new Map([['ch-1', 4]]) })}
			/>,
		)

		expect(screen.getAllByText(/messages non lus/)).toHaveLength(1)
		expect(screen.getByText('4')).toBeDefined()
	})

	it('shows no mention badge when there are no unread mentions', async () => {
		await renderSidebar(<ChatSidebarUI {...baseProps()} />)
		expect(screen.queryByText(/mentions non lues/)).toBeNull()
	})

	it('shows the mention count badge when there are unread mentions', async () => {
		await renderSidebar(<ChatSidebarUI {...baseProps({ mentionCount: 3 })} />)
		expect(screen.getByLabelText('3 mentions non lues')).toBeDefined()
	})
})

describe('ChatSidebarUI permission gating (#407)', () => {
	const CREATE_LABEL = 'Créer un canal ou une catégorie'

	it('offers channel creation to a caller holding MANAGE_CHANNELS', async () => {
		await renderSidebar(<ChatSidebarUI {...baseProps()} />, ['MANAGE_CHANNELS'])
		expect(screen.getByLabelText(CREATE_LABEL)).toBeDefined()
	})

	it('hides the create button from a caller without MANAGE_CHANNELS', async () => {
		await renderSidebar(<ChatSidebarUI {...baseProps()} />, ['VIEW_CHANNEL'])
		expect(screen.queryByLabelText(CREATE_LABEL)).toBeNull()
	})

	it('hides the empty-state create button without MANAGE_CHANNELS', async () => {
		await renderSidebar(<ChatSidebarUI {...baseProps({ groups: [] })} />, [
			'VIEW_CHANNEL',
		])
		expect(
			screen.queryByRole('button', { name: /créer un canal$/i }),
		).toBeNull()
	})
})

describe('ChatSidebarUI — direct messages', () => {
	const karim = {
		channelId: 'dm-1',
		name: 'Karim Benali',
		hue: 200,
		presence: 'ONLINE' as const,
		unreadCount: 2,
	}

	it('lists each conversation by its peer, with the unread count', async () => {
		await renderSidebar(
			<ChatSidebarUI {...baseProps({ directMessages: [karim] })} />,
		)

		const link = screen.getByRole('link', { name: /Karim Benali/ })
		expect(link.getAttribute('href')).toBe('/o/acme/discussions/dm-1')
		expect(screen.getByText(/messages non lus/)).toBeDefined()
		expect(screen.getByText('En ligne')).toBeDefined()
	})

	it('asks for a new direct message from the plus button', async () => {
		const user = userEvent.setup()
		const onRequestNewDirectMessage = vi.fn()
		await renderSidebar(
			<ChatSidebarUI {...baseProps({ onRequestNewDirectMessage })} />,
		)

		await user.click(
			screen.getByRole('button', { name: 'Nouveau message direct' }),
		)

		expect(onRequestNewDirectMessage).toHaveBeenCalledTimes(1)
	})
})
