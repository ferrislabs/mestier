import { describe, expect, it } from 'vitest'
import type { ActivityEntry } from '#/hooks/use-chat'
import {
	buildActivityItems,
	countUnreadMentions,
	filterActivity,
	humanizeMentions,
} from './activity'

const ID = '018f0000-0000-7000-8000-000000000001'
const authors = new Map([[ID, { name: 'Karim', hue: 200 }]])
const NOW = new Date('2026-11-09T12:00:00Z')

function entry(overrides: Partial<ActivityEntry> = {}): ActivityEntry {
	return {
		kind: 'MENTION',
		channel_id: 'ch-1',
		message_id: 'm-1',
		author_user_id: ID,
		author_type: 'USER',
		excerpt: 'salut',
		created_at: '2026-11-09T09:00:00Z',
		read: false,
		notification_id: 'n-1',
		unread_count: null,
		...overrides,
	} as ActivityEntry
}

describe('humanizeMentions', () => {
	it('names a mentioned member and channel', () => {
		const text = humanizeMentions(
			`coucou <@${ID}> dans <#ch-id-that-is-not-a-uuid> et <#${ID}>`,
			authors,
			new Map([[ID, 'general']]),
		)

		expect(text).toBe(
			`coucou @Karim dans <#ch-id-that-is-not-a-uuid> et #general`,
		)
	})

	it('hides a role id behind a plain word', () => {
		expect(humanizeMentions(`<@&${ID}> merci`, authors, new Map())).toBe(
			'@rôle merci',
		)
	})
})

describe('buildActivityItems', () => {
	it('names where a message is: a channel, or the other person for a direct message', () => {
		const items = buildActivityItems(
			[
				entry(),
				entry({ channel_id: 'dm-1', notification_id: null, kind: 'UNREAD' }),
			],
			authors,
			new Map([['ch-1', 'general']]),
			new Map([['dm-1', 'Alix Martin']]),
			NOW,
		)

		expect(items.map((item) => item.where)).toEqual(['#general', 'Alix Martin'])
		expect(items[0]?.authorName).toBe('Karim')
	})

	it('keys a mention by its notification and an unread entry by its channel', () => {
		const items = buildActivityItems(
			[
				entry(),
				entry({ kind: 'UNREAD', notification_id: null, unread_count: 3 }),
			],
			authors,
			new Map(),
			new Map(),
			NOW,
		)

		expect(items.map((item) => item.key)).toEqual(['n-1', 'unread:ch-1'])
		expect(items[1]?.unreadCount).toBe(3)
	})
})

describe('filterActivity and countUnreadMentions', () => {
	const items = buildActivityItems(
		[
			entry(),
			entry({ notification_id: 'n-2', read: true }),
			entry({ kind: 'UNREAD', notification_id: null, unread_count: 2 }),
		],
		authors,
		new Map(),
		new Map(),
		NOW,
	)

	it('splits mentions from unread channels', () => {
		expect(filterActivity(items, 'mentions')).toHaveLength(2)
		expect(filterActivity(items, 'unread')).toHaveLength(1)
		expect(filterActivity(items, 'all')).toHaveLength(3)
	})

	it('counts only the mentions not read yet', () => {
		expect(countUnreadMentions(items)).toBe(1)
	})
})
