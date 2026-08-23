import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { renderWithRouter } from '#/test/render-with-router'
import { ChatSidebarUI, type ChatSidebarUIProps } from './chat-sidebar-ui'

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
		...overrides,
	}
}

describe('ChatSidebarUI', () => {
	it('shows a skeleton while loading', async () => {
		await renderWithRouter(
			<ChatSidebarUI {...baseProps({ isLoading: true })} />,
		)
		expect(screen.getByRole('navigation')).toBeDefined()
	})

	it('shows an error message on failure', async () => {
		await renderWithRouter(<ChatSidebarUI {...baseProps({ isError: true })} />)
		expect(screen.getByText('Impossible de charger les canaux.')).toBeDefined()
	})

	it('shows guidance when the organization has no channel', async () => {
		await renderWithRouter(<ChatSidebarUI {...baseProps()} />)
		expect(screen.getByText(/Aucun canal pour le moment/)).toBeDefined()
	})

	it('lists channels grouped by category', async () => {
		const groups = [
			{
				category: category('cat-1'),
				channels: [channel('ch-1', 'cat-1'), channel('ch-2', 'cat-1')],
			},
		]
		await renderWithRouter(<ChatSidebarUI {...baseProps({ groups })} />)

		expect(screen.getByText('cat-1')).toBeDefined()
		expect(screen.getByText('ch-1')).toBeDefined()
		expect(screen.getByText('ch-2')).toBeDefined()
	})

	it('highlights the active channel', async () => {
		const groups = [
			{ category: category('cat-1'), channels: [channel('ch-1', 'cat-1')] },
		]
		await renderWithRouter(
			<ChatSidebarUI {...baseProps({ groups, activeChannelId: 'ch-1' })} />,
		)

		const link = screen.getByText('ch-1').closest('a')
		expect(link?.className).toContain('bg-accent')
	})

	it('renders uncategorized channels without a category header', async () => {
		const groups = [{ category: null, channels: [channel('ch-1', null)] }]
		await renderWithRouter(<ChatSidebarUI {...baseProps({ groups })} />)

		expect(screen.getByText('ch-1')).toBeDefined()
	})

	it('collapses a category on click', async () => {
		const user = userEvent.setup()
		const onToggleCategory = vi.fn()
		const groups = [
			{ category: category('cat-1'), channels: [channel('ch-1', 'cat-1')] },
		]
		await renderWithRouter(
			<ChatSidebarUI {...baseProps({ groups, onToggleCategory })} />,
		)

		await user.click(screen.getByText('cat-1'))

		expect(onToggleCategory).toHaveBeenCalledWith('cat-1', true)
	})
})
