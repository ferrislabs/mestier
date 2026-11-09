import { Hash, Search, Settings, Users } from 'lucide-react'
import type { ReactNode } from 'react'
import { Skeleton } from '#/components/ui/skeleton'
import type { PresenceStatus } from '#/hooks/use-presence'
import { PRESENCE_LABELS } from '#/pages/chat/lib/presence'
import { IconButton, IconToggle } from '#/pages/chat/ui/icon-button'
import { PersonAvatar } from '#/pages/chat/ui/person-avatar'

export interface DirectPeer {
	name: string
	hue: number
	presence: PresenceStatus
}

export interface ChatChannelHeaderUIProps {
	name?: string
	topic?: string | null
	isLoading: boolean
	searchOpen: boolean
	onSearchOpenChange: (open: boolean) => void
	membersOpen: boolean
	onMembersOpenChange: (open: boolean) => void
	pinned: ReactNode
	/** Set on a direct message: the header names the person, not a channel. */
	peer?: DirectPeer
	/** Shown next to the actions on a channel, as in the rework. */
	memberCount?: number
	onOpenAdmin: () => void
}

export function ChatChannelHeaderUI({
	name,
	topic,
	isLoading,
	searchOpen,
	onSearchOpenChange,
	membersOpen,
	onMembersOpenChange,
	pinned,
	peer,
	memberCount,
	onOpenAdmin,
}: ChatChannelHeaderUIProps) {
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
						<Hash aria-hidden className="size-4 text-muted-foreground" />
						{name}
					</h1>
				)}
				{!peer && topic ? (
					<p className="hidden min-w-0 truncate text-muted-foreground text-sm md:block">
						{topic}
					</p>
				) : null}
			</div>
			{!peer && memberCount !== undefined ? (
				<p className="flex shrink-0 items-center gap-1 text-muted-foreground text-xs">
					<Users aria-hidden className="size-3.5" />
					<span aria-hidden>{memberCount}</span>
					<span className="sr-only">
						{memberCount > 1
							? `${memberCount} membres`
							: `${memberCount} membre`}
					</span>
				</p>
			) : null}
			<div className="flex shrink-0 items-center gap-0.5">
				<IconToggle
					label="Rechercher dans le canal"
					icon={Search}
					pressed={searchOpen}
					onPressedChange={onSearchOpenChange}
				/>
				{pinned}
				<IconToggle
					label={
						membersOpen
							? 'Masquer la liste des membres'
							: 'Afficher la liste des membres'
					}
					icon={Users}
					pressed={membersOpen}
					onPressedChange={onMembersOpenChange}
					className="max-xl:hidden"
				/>
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
