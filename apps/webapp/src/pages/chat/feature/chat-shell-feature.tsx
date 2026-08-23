import { Outlet, useParams } from '@tanstack/react-router'
import { useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useCategories,
	useChannels,
	useChatListGatewaySync,
} from '#/hooks/use-chat'
import { useOnlineCount } from '#/hooks/use-presence'
import { groupChannelsByCategory } from '#/pages/chat/lib/group-channels'
import { ChatSidebarUI } from '#/pages/chat/ui/chat-sidebar-ui'

/**
 * The chat layout: sidebar (categories + channels) on the left, the active
 * channel's own route rendered through `Outlet` on the right. Mounted once
 * per organization visit — queries and the gateway subscription live here so
 * switching channels never re-fetches the channel list itself.
 */
export function ChatShellFeature() {
	const { activeOrganization } = useActiveOrganization()
	const { channelId } = useParams({ strict: false })
	const [collapsedCategoryIds, setCollapsedCategoryIds] = useState<Set<string>>(
		new Set(),
	)

	const categories = useCategories(activeOrganization.id)
	const channels = useChannels(activeOrganization.id)
	useChatListGatewaySync(activeOrganization.id)
	const onlineCount = useOnlineCount()

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
			/>
			<div className="flex min-w-0 flex-1 flex-col">
				<Outlet />
			</div>
		</div>
	)
}
