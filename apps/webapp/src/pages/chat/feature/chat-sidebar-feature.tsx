import { useParams } from '@tanstack/react-router'
import { Plus } from 'lucide-react'
import { useMemo, useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useCategories,
	useChannels,
	useChatDirectory,
	useChatListGatewaySync,
	useDeleteChannel,
	useDirectChannels,
	useNotificationLevels,
	useReadStateGatewaySync,
	useReorderChannels,
	useSetNotificationLevel,
	useUnreadMentionCount,
	useUnreadSummaries,
} from '#/hooks/use-chat'
import { useHasPermission } from '#/hooks/use-permissions'
import { usePresenceStatuses } from '#/hooks/use-presence'
import { buildDirectMessageItems } from '#/pages/chat/lib/direct-messages'
import { groupChannelsByCategory } from '#/pages/chat/lib/group-channels'
import { type DropHint, planMove } from '#/pages/chat/lib/reorder-channels'
import {
	addButtonClassName,
	ChatSidebarUI,
} from '#/pages/chat/ui/chat-sidebar-ui'
import { openChannelAdmin, openNewChannel } from './chat-dialogs-store'
import { NewDirectMessageFeature } from './new-direct-message-feature'

/**
 * The module's side nav for Discussions: categories and channels, then direct
 * messages. Mounted by `ModuleNav` for the whole time the chat module is
 * active, so the gateway subscriptions that keep the lists live live here.
 */
export function ChatSidebarFeature() {
	const { activeOrganization } = useActiveOrganization()
	const organizationId = activeOrganization.id
	const { channelId } = useParams({ strict: false })
	const [collapsedCategoryIds, setCollapsedCategoryIds] = useState<Set<string>>(
		new Set(),
	)

	const categories = useCategories(organizationId)
	const channels = useChannels(organizationId)
	useChatListGatewaySync(organizationId)
	useReadStateGatewaySync(organizationId)
	const unreadSummaries = useUnreadSummaries(organizationId)
	const mentionCount = useUnreadMentionCount(organizationId)
	const directChannels = useDirectChannels(organizationId)
	const directory = useChatDirectory(organizationId)
	const presenceStatuses = usePresenceStatuses()
	const deleteChannel = useDeleteChannel(organizationId)
	const notificationLevels = useNotificationLevels(organizationId)
	const setNotificationLevel = useSetNotificationLevel(organizationId)
	const reorderChannels = useReorderChannels(organizationId)
	const canReorder = useHasPermission('MANAGE_CHANNELS')
	const [draggingId, setDraggingId] = useState<string | null>(null)
	const [dropHint, setDropHint] = useState<DropHint | null>(null)

	const unreadCounts = useMemo(
		() =>
			new Map(
				[...(unreadSummaries.data ?? [])].map(([id, unread]) => [
					id,
					unread.unread_count,
				]),
			),
		[unreadSummaries.data],
	)
	const directMessages = buildDirectMessageItems(
		directChannels.data ?? [],
		directory.data ?? [],
		presenceStatuses,
		unreadCounts,
	)
	const groups = groupChannelsByCategory(
		categories.data ?? [],
		channels.data ?? [],
	)

	function toggleCategory(categoryId: string, collapsed: boolean) {
		setCollapsedCategoryIds((previous) => {
			const next = new Set(previous)
			if (collapsed) next.add(categoryId)
			else next.delete(categoryId)
			return next
		})
	}

	function endDrag() {
		setDraggingId(null)
		setDropHint(null)
	}

	function dropChannel() {
		if (draggingId && dropHint) {
			const placements = planMove(groups, draggingId, dropHint)
			if (placements.length > 0) {
				reorderChannels.mutate({
					path: { organization_id: organizationId },
					body: { placements },
				} as never)
			}
		}
		endDrag()
	}

	function duplicateChannel(id: string) {
		const source = (channels.data ?? []).find((channel) => channel.id === id)
		if (!source) return
		openNewChannel(source.category_id ?? undefined, {
			name: `${source.name} (copie)`,
			kind:
				source.channel_type === 'ANNOUNCEMENT' ||
				source.channel_type === 'VOICE'
					? source.channel_type
					: 'TEXT',
		})
	}

	return (
		<ChatSidebarUI
			organizationSlug={activeOrganization.slug}
			groups={groups}
			collapsedCategoryIds={collapsedCategoryIds}
			onToggleCategory={toggleCategory}
			activeChannelId={channelId}
			isLoading={categories.isLoading || channels.isLoading}
			isError={categories.isError || channels.isError}
			unreadCounts={unreadCounts}
			mentionCount={mentionCount}
			directMessages={directMessages}
			onRequestNewChannel={() => openNewChannel()}
			onRequestNewChannelInCategory={openNewChannel}
			newDirectMessage={
				<NewDirectMessageFeature
					organizationId={organizationId}
					organizationSlug={activeOrganization.slug}
				>
					<button
						type="button"
						aria-label="Nouveau message direct"
						className={addButtonClassName}
					>
						<Plus className="size-3.5" />
					</button>
				</NewDirectMessageFeature>
			}
			onOpenChannelAdmin={openChannelAdmin}
			onDuplicateChannel={duplicateChannel}
			moveError={reorderChannels.isError ? reorderChannels.error.message : null}
			dnd={
				canReorder
					? {
							draggingId,
							hint: dropHint,
							// Re-rendering inside `dragstart` cancels the drag in Chrome.
							onDragStart: (id) => setTimeout(() => setDraggingId(id), 0),
							onDragEnd: endDrag,
							onHint: setDropHint,
							onDrop: dropChannel,
						}
					: undefined
			}
			notificationLevels={notificationLevels.data ?? new Map()}
			onChangeNotificationLevel={(id, level) =>
				setNotificationLevel.mutate({
					path: { channel_id: id },
					body: { level },
				} as never)
			}
			onDeleteChannel={(id) =>
				deleteChannel.mutate({ path: { channel_id: id } } as never)
			}
		/>
	)
}
