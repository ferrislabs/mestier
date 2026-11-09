import { useNavigate } from '@tanstack/react-router'
import { useMemo, useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useActivity,
	useChannels,
	useChatDirectory,
	useDirectChannels,
	useMarkAllNotificationsRead,
	useMarkNotificationRead,
	useRefreshActivity,
} from '#/hooks/use-chat'
import { buildOrgPath } from '#/modules/org-path'
import {
	type ActivityFilter,
	type ActivityItem,
	buildActivityItems,
	countUnreadMentions,
	filterActivity,
} from '#/pages/chat/lib/activity'
import { userHue } from '#/pages/chat/lib/format'
import { ActivityUI } from '#/pages/chat/ui/activity-ui'

/** `/discussions/activite`: what the caller has to read, from the server. */
export function ActivityFeature() {
	const { activeOrganization } = useActiveOrganization()
	const organizationId = activeOrganization.id
	const navigate = useNavigate()
	const activity = useActivity(organizationId)
	const directory = useChatDirectory(organizationId)
	const channels = useChannels(organizationId)
	const directs = useDirectChannels(organizationId)
	const markRead = useMarkNotificationRead()
	const markAllRead = useMarkAllNotificationsRead()
	const refresh = useRefreshActivity(organizationId)
	const [filter, setFilter] = useState<ActivityFilter>('all')

	const items = useMemo(() => {
		const authors = new Map(
			(directory.data ?? []).map((entry) => [
				entry.user_id,
				{ name: entry.display_name, hue: userHue(entry.user_id) },
			]),
		)
		const channelNames = new Map(
			(channels.data ?? []).map((channel) => [channel.id, channel.name]),
		)
		const peerNames = new Map(
			(directory.data ?? []).map((entry) => [
				entry.user_id,
				entry.display_name,
			]),
		)
		const directNames = new Map(
			(directs.data ?? []).map((direct) => [
				direct.channel_id,
				peerNames.get(direct.peer_user_id) ?? 'Ancien membre',
			]),
		)
		return buildActivityItems(
			activity.data ?? [],
			authors,
			channelNames,
			directNames,
			new Date(),
		)
	}, [activity.data, directory.data, channels.data, directs.data])

	function handleOpen(item: ActivityItem) {
		if (item.notificationId && !item.read) {
			markRead.mutate(
				{ path: { notification_id: item.notificationId } } as never,
				{ onSettled: refresh },
			)
		}
		void navigate({
			to: buildOrgPath(
				activeOrganization.slug,
				`/discussions/${item.channelId}`,
			),
		})
	}

	function handleMarkAllRead() {
		markAllRead.mutate({ path: { organization_id: organizationId } } as never, {
			onSettled: refresh,
		})
	}

	return (
		<ActivityUI
			items={filterActivity(items, filter)}
			isLoading={activity.isLoading}
			isError={activity.isError}
			filter={filter}
			onChangeFilter={setFilter}
			canMarkAllRead={countUnreadMentions(items) > 0}
			onMarkAllRead={handleMarkAllRead}
			onOpen={handleOpen}
		/>
	)
}
