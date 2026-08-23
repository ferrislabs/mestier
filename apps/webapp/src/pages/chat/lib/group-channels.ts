import type { Category, Channel } from '#/hooks/use-chat'

export interface ChannelGroup {
	category: Category | null
	channels: Channel[]
}

/**
 * Groups channels under their category, sorted by position within each
 * group, categories sorted by position, uncategorized channels last.
 * A category with no channels still appears — an empty category is a real
 * state an admin can be looking at, not a value to filter away.
 */
export function groupChannelsByCategory(
	categories: Category[],
	channels: Channel[],
): ChannelGroup[] {
	const byCategory = new Map<string, Channel[]>()
	const uncategorized: Channel[] = []

	for (const channel of channels) {
		if (channel.category_id) {
			const group = byCategory.get(channel.category_id) ?? []
			group.push(channel)
			byCategory.set(channel.category_id, group)
		} else {
			uncategorized.push(channel)
		}
	}

	const sortedCategories = [...categories].sort(
		(a, b) => a.position - b.position,
	)
	const byPosition = (a: Channel, b: Channel) => a.position - b.position

	const groups: ChannelGroup[] = sortedCategories.map((category) => ({
		category,
		channels: (byCategory.get(category.id) ?? []).sort(byPosition),
	}))

	if (uncategorized.length > 0) {
		groups.push({ category: null, channels: uncategorized.sort(byPosition) })
	}

	return groups
}
