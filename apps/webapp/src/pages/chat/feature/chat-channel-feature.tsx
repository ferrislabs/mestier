import { useNavigate } from '@tanstack/react-router'
import { useEffect, useMemo, useState } from 'react'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import {
	useChannel,
	useChatDirectory,
	useCreateThread,
	useDirectChannels,
	useThreads,
	useThreadsGatewaySync,
	useUnreadSummaries,
} from '#/hooks/use-chat'
import { usePresenceStatuses } from '#/hooks/use-presence'
import { buildOrgPath } from '#/modules/org-path'
import {
	toggleMessagePin,
	toggleMessageSaved,
	useMessageFlags,
} from '#/pages/chat/feature/mocks/message-flags'
import { resolveAuthor } from '#/pages/chat/lib/authors'
import { formatTime, userHue } from '#/pages/chat/lib/format'
import {
	filterMessagesByQuery,
	formatDayLabel,
	type UnreadMarker,
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
	const {
		authors,
		currentUserId,
		messageListProps,
		composerProps,
		typingCount,
	} = useMessageThreadUi(channelId)
	const { openChannelAdmin } = useChannelAdminDialog()
	const directory = useChatDirectory(activeOrganization.id)
	const presenceStatuses = usePresenceStatuses()
	const unreadSummaries = useUnreadSummaries(activeOrganization.id)
	const directChannels = useDirectChannels(activeOrganization.id)
	const [unreadMarker, setUnreadMarker] = useState<UnreadMarker | undefined>()
	const [unreadCaptured, setUnreadCaptured] = useState(false)

	// The line is drawn from what was unread when the channel opened: reading
	// advances the server's marker, and the line must not vanish under the reader.
	useEffect(() => {
		if (unreadCaptured || !unreadSummaries.data) return
		const summary = unreadSummaries.data.get(channelId)
		if (summary) {
			setUnreadMarker({
				messageId: summary.first_unread_message_id,
				count: summary.unread_count,
			})
		}
		setUnreadCaptured(true)
	}, [channelId, unreadCaptured, unreadSummaries.data])
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
					void navigate({
						to: buildOrgPath(activeOrganization.slug, '/discussions'),
					})
				}
			/>
		)
	}

	const isDirect = channel.data?.channel_type === 'DIRECT'
	const peerUserId = directChannels.data?.find(
		(direct) => direct.channel_id === channelId,
	)?.peer_user_id
	const peerEntry = directory.data?.find(
		(entry) => entry.user_id === peerUserId,
	)
	const peer = isDirect
		? {
				name: peerEntry?.display_name ?? 'Ancien membre',
				hue: userHue(peerUserId),
				presence:
					(peerUserId ? presenceStatuses.get(peerUserId) : undefined) ??
					peerEntry?.presence ??
					'OFFLINE',
			}
		: undefined
	const channelLabel = peer
		? peer.name
		: channel.data?.name
			? `#${channel.data.name}`
			: 'ce canal'
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
				authorName: resolveAuthor(authors, message.author_user_id).name,
				whenLabel: `${formatDayLabel(sentAt, now)} à ${formatTime(sentAt)}`,
				text: message.content,
			}
		})
	const members = (directory.data ?? [])
		.filter(
			(entry) =>
				!isDirect ||
				entry.user_id === currentUserId ||
				entry.user_id === peerUserId,
		)
		.map((entry) => ({
			id: entry.user_id,
			name: entry.display_name,
			hue: userHue(entry.user_id),
			presence: presenceStatuses.get(entry.user_id) ?? entry.presence,
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
					peer={peer}
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
					unread={isFiltering ? undefined : unreadMarker}
					label={`Messages de ${channelLabel}`}
					emptyLabel={
						isFiltering
							? `Aucun message ne correspond à « ${searchQuery.trim()} ».`
							: undefined
					}
					pinnedMessageIds={flags.data.pinned}
					savedMessageIds={flags.data.saved}
					threadsEnabled={!isDirect}
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
					placeholder={
						isDirect
							? `Écrire à ${channelLabel}`
							: `Écrire dans ${channelLabel}`
					}
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
				<MemberList members={members} currentUserId={currentUserId} />
			) : null}
		</div>
	)
}
