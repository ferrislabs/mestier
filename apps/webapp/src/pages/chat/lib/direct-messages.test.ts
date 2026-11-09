import { describe, expect, it } from 'vitest'
import { buildDirectMessageItems } from '#/pages/chat/lib/direct-messages'

const directory = [
	{ user_id: 'u-karim', display_name: 'Karim Benali' },
	{ user_id: 'u-alix', display_name: 'Alix Martin' },
]

describe('buildDirectMessageItems', () => {
	it('names each conversation after its peer and carries their presence', () => {
		const items = buildDirectMessageItems(
			[{ channel_id: 'c-1', peer_user_id: 'u-karim' }],
			directory,
			new Map([['u-karim', 'ONLINE']]),
			new Map(),
		)

		expect(items).toEqual([
			expect.objectContaining({
				channelId: 'c-1',
				name: 'Karim Benali',
				presence: 'ONLINE',
				unreadCount: 0,
			}),
		])
	})

	it('puts unread conversations first, then sorts by name', () => {
		const items = buildDirectMessageItems(
			[
				{ channel_id: 'c-1', peer_user_id: 'u-karim' },
				{ channel_id: 'c-2', peer_user_id: 'u-alix' },
			],
			directory,
			new Map(),
			new Map([['c-1', 3]]),
		)

		expect(items.map((item) => item.name)).toEqual([
			'Karim Benali',
			'Alix Martin',
		])
	})

	it('keeps a conversation whose peer left the organization', () => {
		const items = buildDirectMessageItems(
			[{ channel_id: 'c-9', peer_user_id: 'u-gone' }],
			directory,
			new Map(),
			new Map(),
		)

		expect(items[0]?.name).toBe('Ancien membre')
		expect(items[0]?.presence).toBe('OFFLINE')
	})
})
