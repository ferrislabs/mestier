import { Outlet, useParams } from '@tanstack/react-router'
import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useCategories,
	useChannels,
	useChatListGatewaySync,
	useReadStateGatewaySync,
	useUnreadChannels,
	useUnreadMentionCount,
} from '#/hooks/use-chat'
import { useOnlineCount } from '#/hooks/use-presence'
import { groupChannelsByCategory } from '#/pages/chat/lib/group-channels'
import { ChatSidebarUI } from '#/pages/chat/ui/chat-sidebar-ui'
import {
	ChannelAdminDialogProvider,
	useChannelAdminDialog,
} from './channel-admin-dialog-context'
import { NewChannelFeature } from './new-channel-feature'

/**
 * The chat layout: sidebar (categories + channels) on the left, the active
 * channel's own route rendered through `Outlet` on the right. Mounted once
 * per organization visit — queries and the gateway subscription live here so
 * switching channels never re-fetches the channel list itself.
 *
 * `ChannelAdminDialogProvider` wraps the whole layout so the admin sheet can
 * be opened for any channel from anywhere under it — the sidebar's
 * per-channel menu, or the active channel's own header (`ChatChannelFeature`)
 * — without threading a callback through the router's `Outlet`.
 */
export function ChatShellFeature() {
	const { activeOrganization } = useActiveOrganization()

	return (
		<ChannelAdminDialogProvider>
			<ChatShellLayout organizationId={activeOrganization.id} />
		</ChannelAdminDialogProvider>
	)
}

function ChatShellLayout({ organizationId }: { organizationId: string }) {
	const { activeOrganization } = useActiveOrganization()
	const { channelId } = useParams({ strict: false })
	const { openChannelAdmin } = useChannelAdminDialog()
	const [collapsedCategoryIds, setCollapsedCategoryIds] = useState<Set<string>>(
		new Set(),
	)

	const categories = useCategories(organizationId)
	const channels = useChannels(organizationId)
	useChatListGatewaySync(organizationId)
	useReadStateGatewaySync(organizationId)
	const onlineCount = useOnlineCount()
	const unreadChannelIds = useUnreadChannels(organizationId)
	const mentionCount = useUnreadMentionCount(organizationId)
	const [newChannelDialogOpen, setNewChannelDialogOpen] = useState(false)

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

	return (
		<div className="flex min-h-0 flex-1">
			<ChatSidebarUI
				organizationSlug={activeOrganization.slug}
				groups={groups}
				collapsedCategoryIds={collapsedCategoryIds}
				onToggleCategory={toggleCategory}
				activeChannelId={channelId}
				isLoading={categories.isLoading || channels.isLoading}
				isError={categories.isError || channels.isError}
				onlineCount={onlineCount}
				unreadChannelIds={unreadChannelIds.data ?? new Set()}
				mentionCount={mentionCount}
				onRequestNewChannel={() => setNewChannelDialogOpen(true)}
				onOpenChannelAdmin={openChannelAdmin}
			/>
			<div className="flex min-w-0 flex-1 flex-col">
				<Outlet />
			</div>
			<NewChannelFeature
				organizationId={organizationId}
				open={newChannelDialogOpen}
				onOpenChange={setNewChannelDialogOpen}
			/>
		</div>
	)
}
