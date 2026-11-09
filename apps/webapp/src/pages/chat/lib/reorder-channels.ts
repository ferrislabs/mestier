import type { ChannelGroup } from '#/pages/chat/lib/group-channels'

/** Where a dragged channel lands, as the side nav shows it. */
export type DropHint =
	| { kind: 'before'; channelId: string }
	| { kind: 'after'; channelId: string }
	/** The end of a category, or of the channels without one (`null`). */
	| { kind: 'category'; categoryId: string | null }

export interface ChannelPlacement {
	channel_id: string
	category_id: string | null
	position: number
}

interface Slot {
	categoryId: string | null
	index: number
}

function lists(groups: readonly ChannelGroup[]): Map<string | null, string[]> {
	const result = new Map<string | null, string[]>()
	for (const group of groups) {
		result.set(
			group.category?.id ?? null,
			group.channels.map((channel) => channel.id),
		)
	}
	return result
}

function slotOf(
	hint: DropHint,
	ordered: Map<string | null, string[]>,
): Slot | null {
	if (hint.kind === 'category') {
		const list = ordered.get(hint.categoryId) ?? []
		return { categoryId: hint.categoryId, index: list.length }
	}
	for (const [categoryId, ids] of ordered) {
		const at = ids.indexOf(hint.channelId)
		if (at !== -1) {
			return { categoryId, index: hint.kind === 'before' ? at : at + 1 }
		}
	}
	return null
}

/**
 * The placements to send so that `channelId` ends up where `hint` says.
 *
 * Every channel of a list that changed is renumbered `0..n`, so ranks stay
 * dense and the server never has to break a tie. Nothing is returned when the
 * channel would not move.
 */
export function planMove(
	groups: readonly ChannelGroup[],
	channelId: string,
	hint: DropHint,
): ChannelPlacement[] {
	if (
		(hint.kind === 'before' || hint.kind === 'after') &&
		hint.channelId === channelId
	) {
		return []
	}
	const before = lists(groups)
	const target = slotOf(hint, before)
	if (!target) return []

	let sourceKey: string | null | undefined
	let sourceIndex = -1
	for (const [categoryId, ids] of before) {
		const at = ids.indexOf(channelId)
		if (at !== -1) {
			sourceKey = categoryId
			sourceIndex = at
		}
	}
	if (sourceKey === undefined) return []

	const after = new Map([...before].map(([key, ids]) => [key, [...ids]]))
	after.get(sourceKey)?.splice(sourceIndex, 1)
	const destination = after.get(target.categoryId) ?? []
	// Removing the channel above its own target shifts the slot up by one.
	const index =
		sourceKey === target.categoryId && sourceIndex < target.index
			? target.index - 1
			: target.index
	destination.splice(index, 0, channelId)
	after.set(target.categoryId, destination)

	const changed = new Set([sourceKey, target.categoryId])
	const placements: ChannelPlacement[] = []
	for (const key of changed) {
		const ids = after.get(key) ?? []
		const initial = before.get(key) ?? []
		ids.forEach((id, position) => {
			if (initial[position] === id && key === lookupKey(before, id)) return
			placements.push({ channel_id: id, category_id: key, position })
		})
	}
	return placements
}

function lookupKey(
	lists: Map<string | null, string[]>,
	channelId: string,
): string | null | undefined {
	for (const [key, ids] of lists) if (ids.includes(channelId)) return key
	return undefined
}
