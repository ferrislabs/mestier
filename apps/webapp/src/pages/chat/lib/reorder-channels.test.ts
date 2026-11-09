import { describe, expect, it } from 'vitest'
import type { Category, Channel } from '#/hooks/use-chat'
import { groupChannelsByCategory } from './group-channels'
import { planMove } from './reorder-channels'

function category(id: string, position: number): Category {
	return { id, name: id, position } as Category
}

function channel(
	id: string,
	categoryId: string | null,
	position: number,
): Channel {
	return { id, name: id, category_id: categoryId, position } as Channel
}

const groups = groupChannelsByCategory(
	[category('c1', 0), category('c2', 1)],
	[
		channel('a', 'c1', 0),
		channel('b', 'c1', 1),
		channel('c', 'c1', 2),
		channel('d', 'c2', 0),
		channel('loose', null, 0),
	],
)

describe('planMove', () => {
	it('reorders inside a category and renumbers only what moved', () => {
		const placements = planMove(groups, 'c', { kind: 'before', channelId: 'a' })

		expect(placements).toEqual([
			{ channel_id: 'c', category_id: 'c1', position: 0 },
			{ channel_id: 'a', category_id: 'c1', position: 1 },
			{ channel_id: 'b', category_id: 'c1', position: 2 },
		])
	})

	it('moves a channel after another one, down the same list', () => {
		const placements = planMove(groups, 'a', { kind: 'after', channelId: 'b' })

		expect(placements).toEqual([
			{ channel_id: 'b', category_id: 'c1', position: 0 },
			{ channel_id: 'a', category_id: 'c1', position: 1 },
		])
	})

	it('moves a channel to another category and closes the gap it leaves', () => {
		const placements = planMove(groups, 'a', { kind: 'before', channelId: 'd' })

		expect(placements).toEqual([
			{ channel_id: 'b', category_id: 'c1', position: 0 },
			{ channel_id: 'c', category_id: 'c1', position: 1 },
			{ channel_id: 'a', category_id: 'c2', position: 0 },
			{ channel_id: 'd', category_id: 'c2', position: 1 },
		])
	})

	it('drops on a category header to append at its end', () => {
		const placements = planMove(groups, 'loose', {
			kind: 'category',
			categoryId: 'c2',
		})

		expect(placements).toContainEqual({
			channel_id: 'loose',
			category_id: 'c2',
			position: 1,
		})
	})

	it('takes a channel out of its category', () => {
		const placements = planMove(groups, 'a', {
			kind: 'category',
			categoryId: null,
		})

		expect(placements).toContainEqual({
			channel_id: 'a',
			category_id: null,
			position: 1,
		})
	})

	it('plans nothing when the channel would stay where it is', () => {
		expect(planMove(groups, 'b', { kind: 'before', channelId: 'c' })).toEqual(
			[],
		)
		expect(planMove(groups, 'b', { kind: 'after', channelId: 'a' })).toEqual([])
		expect(planMove(groups, 'b', { kind: 'before', channelId: 'b' })).toEqual(
			[],
		)
	})

	it('can fill a category that has no channel yet', () => {
		const withEmpty = groupChannelsByCategory(
			[category('c1', 0), category('empty', 1)],
			[channel('a', 'c1', 0)],
		)

		expect(
			planMove(withEmpty, 'a', { kind: 'category', categoryId: 'empty' }),
		).toEqual([{ channel_id: 'a', category_id: 'empty', position: 0 }])
	})
})
