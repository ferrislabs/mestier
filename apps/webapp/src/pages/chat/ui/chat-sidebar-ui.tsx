import { Link } from '@tanstack/react-router'
import {
	AlertCircle,
	ChevronRight,
	Hash,
	MoreHorizontal,
	Plus,
	Settings,
} from 'lucide-react'
import { RequirePermission } from '#/components/require-permission'
import {
	Collapsible,
	CollapsibleContent,
	CollapsibleTrigger,
} from '#/components/ui/collapsible'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import { Skeleton } from '#/components/ui/skeleton'
import { cn } from '#/lib/utils'
import { buildOrgPath } from '#/modules/org-path'
import type { DirectMessageItem } from '#/pages/chat/lib/direct-messages'
import type { ChannelGroup } from '#/pages/chat/lib/group-channels'
import { PersonAvatar } from '#/pages/chat/ui/person-avatar'

export interface ChatSidebarUIProps {
	organizationSlug: string
	groups: ChannelGroup[]
	collapsedCategoryIds: ReadonlySet<string>
	onToggleCategory: (categoryId: string, collapsed: boolean) => void
	activeChannelId?: string
	isLoading: boolean
	isError: boolean
	unreadCounts: ReadonlyMap<string, number>
	mentionCount: number
	directMessages: readonly DirectMessageItem[]
	onRequestNewChannel: () => void
	onRequestNewDirectMessage: () => void
	/** Opens the channel admin sheet for one channel — see #372: channel
	 * management starts here, not only from inside the channel itself. */
	onOpenChannelAdmin: (channelId: string) => void
}

const headingClassName =
	'font-semibold text-nav-muted text-xs uppercase tracking-wide'

const linkClassName =
	'flex h-8 items-center gap-2 rounded-md px-2 text-nav-foreground/80 text-sm outline-none transition-colors hover:bg-nav-active/60 hover:text-nav-foreground focus-visible:ring-2 focus-visible:ring-nav-primary data-[status=active]:bg-nav-active data-[status=active]:text-nav-active-foreground'

const addButtonClassName =
	'flex size-6 items-center justify-center rounded-md text-nav-muted outline-none transition-colors hover:bg-nav-active/60 hover:text-nav-foreground focus-visible:ring-2 focus-visible:ring-nav-primary'

/**
 * Channels and direct messages, in the module's side nav. Coloured with the
 * nav tokens, not the card ones: this sits on the dark rail.
 */
export function ChatSidebarUI({
	organizationSlug,
	groups,
	collapsedCategoryIds,
	onToggleCategory,
	activeChannelId,
	isLoading,
	isError,
	unreadCounts,
	mentionCount,
	directMessages,
	onRequestNewChannel,
	onRequestNewDirectMessage,
	onOpenChannelAdmin,
}: ChatSidebarUIProps) {
	return (
		<nav
			aria-label="Canaux et messages directs"
			className="flex flex-col gap-4 py-2"
		>
			<section aria-labelledby="discussions-channels-heading">
				<div className="flex items-center justify-between px-2 pb-1">
					<h2 id="discussions-channels-heading" className={headingClassName}>
						Canaux
					</h2>
					<div className="flex items-center gap-1">
						{mentionCount > 0 ? (
							<output
								aria-label={`${mentionCount} mentions non lues`}
								className="tabular flex h-5 min-w-5 items-center justify-center rounded-full bg-destructive px-1.5 font-semibold text-destructive-foreground text-xs"
							>
								{mentionCount}
							</output>
						) : null}
						<RequirePermission permission="MANAGE_CHANNELS">
							<button
								type="button"
								aria-label="Créer un canal ou une catégorie"
								className={addButtonClassName}
								onClick={onRequestNewChannel}
							>
								<Plus className="size-3.5" />
							</button>
						</RequirePermission>
					</div>
				</div>

				{isLoading ? <SidebarSkeleton /> : null}

				{isError ? (
					<div className="flex items-start gap-2 px-2 py-1 text-nav-foreground/80 text-sm">
						<AlertCircle className="mt-0.5 size-4 shrink-0" />
						<span>Impossible de charger les canaux.</span>
					</div>
				) : null}

				{!isLoading && !isError && groups.length === 0 ? (
					<p className="px-2 py-1 text-nav-muted text-sm">
						Aucun canal pour le moment.
					</p>
				) : null}

				{!isLoading && !isError && groups.length > 0 ? (
					<div className="flex flex-col gap-2">
						{groups.map((group) => (
							<CategoryGroup
								key={group.category?.id ?? 'uncategorized'}
								group={group}
								organizationSlug={organizationSlug}
								activeChannelId={activeChannelId}
								collapsed={
									group.category
										? collapsedCategoryIds.has(group.category.id)
										: false
								}
								onToggleCategory={onToggleCategory}
								unreadCounts={unreadCounts}
								onOpenChannelAdmin={onOpenChannelAdmin}
							/>
						))}
					</div>
				) : null}
			</section>

			<section aria-labelledby="discussions-direct-heading">
				<div className="flex items-center justify-between px-2 pb-1">
					<h2 id="discussions-direct-heading" className={headingClassName}>
						Messages directs
					</h2>
					<button
						type="button"
						aria-label="Nouveau message direct"
						className={addButtonClassName}
						onClick={onRequestNewDirectMessage}
					>
						<Plus className="size-3.5" />
					</button>
				</div>
				<ul className="flex flex-col gap-0.5">
					{directMessages.map((item) => (
						<li key={item.channelId}>
							<Link
								to={buildOrgPath(
									organizationSlug,
									`/discussions/${item.channelId}`,
								)}
								className={cn(
									linkClassName,
									item.unreadCount > 0 && 'font-semibold text-nav-foreground',
								)}
							>
								<PersonAvatar
									name={item.name}
									hue={item.hue}
									size="sm"
									presence={item.presence}
									surface="nav"
								/>
								<span className="min-w-0 flex-1 truncate">{item.name}</span>
								<UnreadBadge count={item.unreadCount} />
							</Link>
						</li>
					))}
				</ul>
			</section>
		</nav>
	)
}

function UnreadBadge({ count }: { count: number }) {
	if (count <= 0) return null
	return (
		<span className="tabular flex h-5 min-w-5 shrink-0 items-center justify-center rounded-full bg-nav-primary px-1.5 font-semibold text-nav-primary-foreground text-xs">
			{count}
			<span className="sr-only">
				{' '}
				{count > 1 ? 'messages non lus' : 'message non lu'}
			</span>
		</span>
	)
}

function SidebarSkeleton() {
	return (
		<div className="flex flex-col gap-2 px-2 py-1">
			<Skeleton className="h-6 w-full bg-nav-active/60" />
			<Skeleton className="h-6 w-full bg-nav-active/60" />
			<Skeleton className="h-6 w-3/4 bg-nav-active/60" />
		</div>
	)
}

interface CategoryGroupProps {
	group: ChannelGroup
	organizationSlug: string
	activeChannelId?: string
	collapsed: boolean
	onToggleCategory: (categoryId: string, collapsed: boolean) => void
	unreadCounts: ReadonlyMap<string, number>
	onOpenChannelAdmin: (channelId: string) => void
}

function CategoryGroup({
	group,
	organizationSlug,
	activeChannelId,
	collapsed,
	onToggleCategory,
	unreadCounts,
	onOpenChannelAdmin,
}: CategoryGroupProps) {
	const channelList = (
		<ul className="flex flex-col gap-0.5">
			{group.channels.map((chan) => {
				const unreadCount = unreadCounts.get(chan.id) ?? 0
				return (
					<li key={chan.id} className="group/channel relative">
						<Link
							to={buildOrgPath(organizationSlug, `/discussions/${chan.id}`)}
							data-status={chan.id === activeChannelId ? 'active' : undefined}
							className={cn(
								linkClassName,
								'pr-8',
								unreadCount > 0 && 'font-semibold text-nav-foreground',
							)}
						>
							<Hash aria-hidden className="size-4 shrink-0 text-nav-muted" />
							<span className="min-w-0 flex-1 truncate">{chan.name}</span>
							<UnreadBadge count={unreadCount} />
						</Link>
						<DropdownMenu>
							<DropdownMenuTrigger asChild>
								<button
									type="button"
									aria-label={`Gérer le canal ${chan.name}`}
									className="absolute top-1/2 right-1 flex size-6 -translate-y-1/2 items-center justify-center rounded-md text-nav-muted opacity-0 outline-none hover:bg-nav-active hover:text-nav-foreground focus-visible:opacity-100 group-hover/channel:opacity-100 data-[state=open]:opacity-100"
								>
									<MoreHorizontal className="size-3.5" />
								</button>
							</DropdownMenuTrigger>
							<DropdownMenuContent align="start">
								<DropdownMenuItem onSelect={() => onOpenChannelAdmin(chan.id)}>
									<Settings className="size-4" />
									Paramètres du canal
								</DropdownMenuItem>
							</DropdownMenuContent>
						</DropdownMenu>
					</li>
				)
			})}
		</ul>
	)

	if (!group.category) {
		return channelList
	}

	return (
		<Collapsible
			open={!collapsed}
			onOpenChange={(open) => {
				if (group.category) onToggleCategory(group.category.id, !open)
			}}
		>
			<CollapsibleTrigger className="flex w-full items-center gap-1 rounded-md px-2 pb-1 font-semibold text-nav-muted text-xs uppercase tracking-wide outline-none hover:text-nav-foreground focus-visible:ring-2 focus-visible:ring-nav-primary">
				<ChevronRight
					className={cn(
						'size-3.5 shrink-0 transition-transform',
						!collapsed && 'rotate-90',
					)}
				/>
				<span className="truncate">{group.category.name}</span>
			</CollapsibleTrigger>
			<CollapsibleContent>{channelList}</CollapsibleContent>
		</Collapsible>
	)
}
