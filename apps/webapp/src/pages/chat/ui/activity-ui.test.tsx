import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { ActivityItem } from '#/pages/chat/lib/activity'
import { ActivityUI, type ActivityUIProps } from './activity-ui'

function item(overrides: Partial<ActivityItem> = {}): ActivityItem {
	return {
		key: 'n-1',
		channelId: 'ch-1',
		kind: 'MENTION',
		authorName: 'Karim Benali',
		authorHue: 200,
		where: '#general',
		excerpt: 'Tu peux regarder ça ?',
		when: '09:00',
		read: false,
		unreadCount: null,
		notificationId: 'n-1',
		...overrides,
	}
}

function props(overrides: Partial<ActivityUIProps> = {}): ActivityUIProps {
	return {
		items: [item()],
		isLoading: false,
		isError: false,
		filter: 'all',
		onChangeFilter: vi.fn(),
		canMarkAllRead: true,
		onMarkAllRead: vi.fn(),
		onOpen: vi.fn(),
		...overrides,
	}
}

describe('ActivityUI', () => {
	it('says who mentioned the person, where, and what they wrote', () => {
		render(<ActivityUI {...props()} />)

		expect(screen.getByText('Karim Benali')).toBeDefined()
		expect(screen.getByText(/vous a mentionné · #general/)).toBeDefined()
		expect(screen.getByText('Tu peux regarder ça ?')).toBeDefined()
		expect(screen.getByRole('img', { name: 'Non lu' })).toBeDefined()
	})

	it('counts the waiting messages of an unread channel', () => {
		render(
			<ActivityUI
				{...props({
					items: [
						item({
							key: 'unread:ch-1',
							kind: 'UNREAD',
							unreadCount: 3,
							notificationId: null,
						}),
					],
				})}
			/>,
		)

		expect(screen.getByText(/3 nouveaux messages · #general/)).toBeDefined()
	})

	it('opens the item that is clicked', async () => {
		const user = userEvent.setup()
		const onOpen = vi.fn()
		render(<ActivityUI {...props({ onOpen })} />)

		await user.click(screen.getByText('Tu peux regarder ça ?'))

		expect(onOpen).toHaveBeenCalledWith(item())
	})

	it('switches filter and marks everything read', async () => {
		const user = userEvent.setup()
		const onChangeFilter = vi.fn()
		const onMarkAllRead = vi.fn()
		render(<ActivityUI {...props({ onChangeFilter, onMarkAllRead })} />)

		await user.click(screen.getByRole('tab', { name: 'Mentions' }))
		await user.click(
			screen.getByRole('button', { name: 'Tout marquer comme lu' }),
		)

		expect(onChangeFilter).toHaveBeenCalledWith('mentions')
		expect(onMarkAllRead).toHaveBeenCalledTimes(1)
	})

	it('disables marking all read when nothing is unread', () => {
		render(<ActivityUI {...props({ canMarkAllRead: false })} />)

		expect(
			screen
				.getByRole('button', { name: 'Tout marquer comme lu' })
				.hasAttribute('disabled'),
		).toBe(true)
	})

	it('explains an empty activity', () => {
		render(<ActivityUI {...props({ items: [] })} />)

		expect(screen.getByText('Rien à lire pour le moment')).toBeDefined()
	})
})
