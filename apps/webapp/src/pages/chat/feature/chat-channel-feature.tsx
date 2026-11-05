import { useNavigate } from '@tanstack/react-router'
import { useMemo, useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useChannel,
	useCreateThread,
	useThreads,
	useThreadsGatewaySync,
} from '#/hooks/use-chat'
import { useOnlineCount } from '#/hooks/use-presence'
import { useReferenceCatalog } from '#/hooks/use-reference-catalog'
import { buildOrgPath } from '#/modules/org-path'
import {
	toggleMessagePin,
	toggleMessageSaved,
	useMessageFlags,
} from '#/pages/chat/feature/mocks/message-flags'
import { formatTime, userHue } from '#/pages/chat/lib/format'
import {
	filterMessagesByQuery,
	formatDayLabel,
} from '#/pages/chat/lib/timeline'
import { ChannelNotFoundUI } from '#/pages/chat/ui/channel-not-found-ui'
import { ChannelSearchBar } from '#/pages/chat/ui/channel-search-bar'
import { ChatChannelHeaderUI } from '#/pages/chat/ui/chat-channel-header-ui'
import { MemberList } from '#/pages/chat/ui/member-list'
import { MessageComposerUI } from '#/pages/chat/ui/message-composer-ui'
import { MessageListUI } from '#/pages/chat/ui/message-list-ui'
import {
	type PinnedMessageItem,
	PinnedMessagesPopover,
} from '#/pages/chat/ui/pinned-messages-popover'
import { TypingIndicatorUI } from '#/pages/chat/ui/typing-indicator-ui'
import { useChannelAdminDialog } from './channel-admin-dialog-context'
import { ChatThreadPanelFeature } from './chat-thread-panel-feature'
import { useMessageThreadUi } from './use-message-thread-ui'

export interface ChatChannelFeatureProps {
	channelId: string
}

function resultLabel(count: number): string {
	if (count === 0) return 'Aucun résultat'
	return count > 1 ? `${count} résultats` : '1 résultat'
}

export function ChatChannelFeature({ channelId }: ChatChannelFeatureProps) {
	const { activeOrganization } = useActiveOrganization()
	const navigate = useNavigate()
	const channel = useChannel(channelId)
	const { currentUserId, messageListProps, composerProps, typingCount } =
		useMessageThreadUi(channelId)
	const { openChannelAdmin } = useChannelAdminDialog()
	const onlineCount = useOnlineCount()
	const catalog = useReferenceCatalog(activeOrganization.id, {
		employeeProfiles: false,
		equipment: false,
		serviceRates: false,
		products: false,
		customUnits: false,
	})
	const flags = useMessageFlags()

	const threads = useThreads(channelId)
	useThreadsGatewaySync(channelId)
	const createThread = useCreateThread()
	const [openThreadChannelId, setOpenThreadChannelId] = useState<string | null>(
		null,
	)
	const [searchOpen, setSearchOpen] = useState(false)
	const [searchQuery, setSearchQuery] = useState('')
	const [membersOpen, setMembersOpen] = useState(true)

	const threadByOriginMessageId = useMemo(() => {
		const map = new Map<string, string>()
		for (const thread of threads.data ?? []) {
			if (thread.origin_message_id) {
				map.set(thread.origin_message_id, thread.id)
			}
		}
		return map
	}, [threads.data])

	function handleOpenThread(messageId: string) {
		const existing = threadByOriginMessageId.get(messageId)
		if (existing) {
			setOpenThreadChannelId(existing)
			return
		}

		createThread.mutate(
			{
				path: { channel_id: channelId },
				body: { name: 'Fil de discussion', origin_message_id: messageId },
			} as never,
			{
				onSuccess: (response) => {
					setOpenThreadChannelId((response as { data: { id: string } }).data.id)
				},
			},
		)
	}

	function closeSearch() {
		setSearchOpen(false)
		setSearchQuery('')
	}

	function handleCopyLink(messageId: string) {
		const url = `${window.location.origin}${window.location.pathname}#${messageId}`
		navigator.clipboard?.writeText(url).catch(() => undefined)
	}

	if (channel.isError) {
		return (
			<ChannelNotFoundUI
				onBack={() =>
					void navigate({ to: buildOrgPath(activeOrganization.slug, '/chat') })
				}
			/>
		)
	}

	const channelLabel = channel.data?.name ? `#${channel.data.name}` : 'ce canal'
	const isFiltering = searchOpen && searchQuery.trim() !== ''
	const visibleMessages = isFiltering
		? filterMessagesByQuery(messageListProps.messages, searchQuery)
		: messageListProps.messages
	const now = new Date()
	const pinnedItems: PinnedMessageItem[] = messageListProps.messages
		.filter(({ message }) => flags.data.pinned.has(message.id))
		.map(({ message }) => {
			const sentAt = new Date(message.created_at)
			return {
				id: message.id,
				authorName:
					currentUserId !== null && message.author_user_id === currentUserId
						? 'Vous'
						: 'Membre',
				whenLabel: `${formatDayLabel(sentAt, now)} à ${formatTime(sentAt)}`,
				text: message.content,
			}
		})
	const members = (catalog.members.data?.data ?? []).map((member) => ({
		id: member.id,
		name: member.display_name,
		hue: userHue(member.id),
	}))

	return (
		<div className="flex min-h-0 flex-1">
			<section
				aria-label={`Conversation ${channelLabel}`}
				className="flex min-h-0 min-w-0 flex-1 flex-col bg-background"
			>
				<ChatChannelHeaderUI
					name={channel.data?.name}
					topic={channel.data?.topic}
					isLoading={channel.isLoading}
					searchOpen={searchOpen}
					onSearchOpenChange={(open) =>
						open ? setSearchOpen(true) : closeSearch()
					}
					membersOpen={membersOpen}
					onMembersOpenChange={setMembersOpen}
					pinned={<PinnedMessagesPopover items={pinnedItems} />}
					onOpenAdmin={() => openChannelAdmin(channelId)}
				/>
				{searchOpen ? (
					<ChannelSearchBar
						query={searchQuery}
						label={`Rechercher dans ${channelLabel}`}
						resultLabel={isFiltering ? resultLabel(visibleMessages.length) : ''}
						onQueryChange={setSearchQuery}
						onClose={closeSearch}
					/>
				) : null}
				<MessageListUI
					{...messageListProps}
					messages={visibleMessages}
					label={`Messages de ${channelLabel}`}
					emptyLabel={
						isFiltering
							? `Aucun message ne correspond à « ${searchQuery.trim()} ».`
							: undefined
					}
					pinnedMessageIds={flags.data.pinned}
					savedMessageIds={flags.data.saved}
					threadedMessageIds={new Set(threadByOriginMessageId.keys())}
					onOpenThread={handleOpenThread}
					onCopyLink={handleCopyLink}
					onTogglePin={(messageId) => void toggleMessagePin(messageId)}
					onToggleSaved={(messageId) => void toggleMessageSaved(messageId)}
				/>
				<TypingIndicatorUI typingCount={typingCount} />
				<MessageComposerUI
					{...composerProps}
					label={`Message dans ${channelLabel}`}
					placeholder={`Écrire dans ${channelLabel}`}
				/>
			</section>
			{openThreadChannelId ? (
				<ChatThreadPanelFeature
					key={openThreadChannelId}
					threadChannelId={openThreadChannelId}
					onClose={() => setOpenThreadChannelId(null)}
				/>
			) : null}
			{membersOpen ? (
				<MemberList members={members} onlineCount={onlineCount} />
			) : null}
		</div>
	)
}
