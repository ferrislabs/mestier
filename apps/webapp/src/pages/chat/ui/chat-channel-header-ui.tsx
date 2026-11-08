import { Search, Settings } from 'lucide-react'
import type { ReactNode } from 'react'
import { Skeleton } from '#/components/ui/skeleton'
import type { PresenceStatus } from '#/hooks/use-presence'
import { channelKindIcon } from '#/pages/chat/lib/channel-kinds'
import { PRESENCE_LABELS } from '#/pages/chat/lib/presence'
import { AvatarGroup } from '#/pages/chat/ui/avatar-group'
import { IconButton, IconToggle } from '#/pages/chat/ui/icon-button'
import { PersonAvatar } from '#/pages/chat/ui/person-avatar'

export interface DirectPeer {
	name: string
	hue: number
	presence: PresenceStatus
}

export interface ChatChannelHeaderUIProps {
	name?: string
	channelType?: string
	topic?: string | null
	isLoading: boolean
	searchOpen: boolean
	onSearchOpenChange: (open: boolean) => void
	pinned: ReactNode
	/** Set on a direct message: the header names the person, not a channel. */
	peer?: DirectPeer
	/** The first of them are shown as an avatar group after the name. */
	members?: readonly { id: string; name: string; hue: number }[]
	onOpenMembers?: () => void
	onOpenAdmin: () => void
}

export function ChatChannelHeaderUI({
	name,
	channelType,
	topic,
	isLoading,
	searchOpen,
	onSearchOpenChange,
	pinned,
	peer,
	members,
	onOpenMembers,
	onOpenAdmin,
}: ChatChannelHeaderUIProps) {
	const KindIcon = channelKindIcon(channelType)

	if (isLoading) {
		return (
			<header className="flex min-h-12 shrink-0 items-center gap-3 border-border border-b bg-background px-4 py-2">
				<Skeleton className="size-4 rounded-full" />
				<Skeleton className="h-4 w-32" />
			</header>
		)
	}

	return (
		<header className="flex min-h-12 shrink-0 items-center gap-3 border-border border-b bg-background px-4 py-2">
			<div className="flex min-w-0 flex-1 items-center gap-3">
				{peer ? (
					<>
						<PersonAvatar
							name={peer.name}
							hue={peer.hue}
							size="sm"
							presence={peer.presence}
						/>
						<h1 className="truncate font-semibold text-base">{peer.name}</h1>
						<p className="hidden truncate text-muted-foreground text-sm md:block">
							{PRESENCE_LABELS[peer.presence]}
						</p>
					</>
				) : (
					<h1 className="flex shrink-0 items-center gap-1 font-semibold text-base">
						<KindIcon aria-hidden className="size-4 text-muted-foreground" />
						{name}
					</h1>
				)}
				{!peer && members && onOpenMembers ? (
					<AvatarGroup members={members} onOpen={onOpenMembers} />
				) : null}
				{!peer && topic ? (
					<p className="hidden min-w-0 truncate text-muted-foreground text-sm md:block">
						{topic}
					</p>
				) : null}
			</div>
			<div className="flex shrink-0 items-center gap-0.5">
				<IconToggle
					label="Rechercher dans le canal"
					icon={Search}
					pressed={searchOpen}
					onPressedChange={onSearchOpenChange}
				/>
				{pinned}
				{peer ? null : (
					<IconButton
						label="Paramètres du canal"
						icon={Settings}
						onClick={onOpenAdmin}
					/>
				)}
			</div>
		</header>
	)
}
