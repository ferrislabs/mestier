import { useChannels, useUnreadChannels } from '#/hooks/use-chat'
import {
	type DiscussionChannelRow,
	DiscussionsUI,
} from '#/pages/home/ui/discussions-ui'

interface DiscussionsFeatureProps {
	organizationId: string
	organizationSlug: string
}

const MAX_ROWS = 5

/**
 * A homepage glance at the chat: unread channels first, no message preview —
 * the API has no last-message snapshot to show one honestly (see
 * `useUnreadChannels`'s own doc in `hooks/use-chat.ts`).
 */
export function DiscussionsFeature({
	organizationId,
	organizationSlug,
}: DiscussionsFeatureProps) {
	const channels = useChannels(organizationId)
	const unread = useUnreadChannels(organizationId)
	const unreadIds = unread.data ?? new Set<string>()

	const rows: DiscussionChannelRow[] = (channels.data ?? [])
		.filter((channel) => !channel.archived)
		.map((channel) => ({
			id: channel.id,
			name: channel.name,
			topic: channel.topic ?? null,
			unread: unreadIds.has(channel.id),
		}))
		.sort((a, b) => Number(b.unread) - Number(a.unread))
		.slice(0, MAX_ROWS)

	return (
		<DiscussionsUI
			organizationSlug={organizationSlug}
			channels={rows}
			isLoading={channels.isLoading}
			error={channels.error?.message ?? null}
		/>
	)
}
