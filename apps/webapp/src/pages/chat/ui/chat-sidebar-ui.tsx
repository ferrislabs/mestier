import { Link } from '@tanstack/react-router'
import {
	AlertCircle,
	ChevronRight,
	Hash,
	MessageSquarePlus,
	MoreHorizontal,
	Plus,
	Settings,
} from 'lucide-react'
import { Button } from '#/components/ui/button'
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
import type { ChannelGroup } from '#/pages/chat/lib/group-channels'
import { PresenceSummaryUI } from '#/pages/chat/ui/presence-summary-ui'

export interface ChatSidebarUIProps {
	organizationSlug: string
	groups: ChannelGroup[]
	collapsedCategoryIds: ReadonlySet<string>
	onToggleCategory: (categoryId: string, collapsed: boolean) => void
	activeChannelId?: string
	isLoading: boolean
	isError: boolean
	onlineCount: number
	unreadChannelIds: ReadonlySet<string>
	mentionCount: number
	onRequestNewChannel: () => void
	/** Opens the channel admin sheet for one channel — see #372: channel
	 * management now starts here, not only from inside the channel itself. */
	onOpenChannelAdmin: (channelId: string) => void
}

export function ChatSidebarUI({
	organizationSlug,
	groups,
	collapsedCategoryIds,
	onToggleCategory,
	activeChannelId,
	isLoading,
	isError,
	onlineCount,
	unreadChannelIds,
	mentionCount,
	onRequestNewChannel,
	onOpenChannelAdmin,
}: ChatSidebarUIProps) {
	return (
		<nav
			aria-label="Canaux"
			className="flex h-full w-64 shrink-0 flex-col overflow-y-auto border-r bg-card/50"
		>
			<div className="flex items-center justify-between px-4 py-4">
				<h2 className="text-sm font-semibold text-foreground">Discussions</h2>
				<div className="flex items-center gap-1">
					{mentionCount > 0 ? (
						<output
							aria-label={`${mentionCount} mentions non lues`}
							className="flex min-w-5 items-center justify-center rounded-full bg-destructive px-1.5 py-0.5 text-xs font-semibold text-destructive-foreground"
						>
							{mentionCount}
						</output>
					) : null}
					<Button
						type="button"
						variant="ghost"
						size="icon"
						aria-label="Créer un canal ou une catégorie"
						onClick={onRequestNewChannel}
					>
						<Plus className="size-4" />
					</Button>
				</div>
			</div>

			<PresenceSummaryUI onlineCount={onlineCount} />

			{isLoading ? <SidebarSkeleton /> : null}

			{isError ? (
				<div className="flex items-start gap-2 px-4 py-2 text-sm text-destructive">
					<AlertCircle className="mt-0.5 size-4 shrink-0" />
					<span>Impossible de charger les canaux.</span>
				</div>
			) : null}

			{!isLoading && !isError && groups.length === 0 ? (
				<EmptyState onRequestNewChannel={onRequestNewChannel} />
			) : null}

			{!isLoading && !isError && groups.length > 0 ? (
				<div className="flex flex-col gap-1 px-2 pb-4">
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
							unreadChannelIds={unreadChannelIds}
							onOpenChannelAdmin={onOpenChannelAdmin}
						/>
					))}
				</div>
			) : null}
		</nav>
	)
}

function SidebarSkeleton() {
	return (
		<div className="flex flex-col gap-2 px-4 py-2">
			<Skeleton className="h-4 w-24" />
			<Skeleton className="h-6 w-full" />
			<Skeleton className="h-6 w-full" />
			<Skeleton className="mt-3 h-4 w-20" />
			<Skeleton className="h-6 w-full" />
		</div>
	)
}

function EmptyState({
	onRequestNewChannel,
}: {
	onRequestNewChannel: () => void
}) {
	return (
		<div className="flex flex-col items-start gap-2 px-4 py-3 text-sm text-muted-foreground">
			<MessageSquarePlus className="size-5" />
			<p>Aucun canal pour le moment.</p>
			<Button variant="outline" size="sm" onClick={onRequestNewChannel}>
				<Plus className="size-4" />
				Créer un canal
			</Button>
		</div>
	)
}

interface CategoryGroupProps {
	group: ChannelGroup
	organizationSlug: string
	activeChannelId?: string
	collapsed: boolean
	onToggleCategory: (categoryId: string, collapsed: boolean) => void
	unreadChannelIds: ReadonlySet<string>
	onOpenChannelAdmin: (channelId: string) => void
}

function CategoryGroup({
	group,
	organizationSlug,
	activeChannelId,
	collapsed,
	onToggleCategory,
	unreadChannelIds,
	onOpenChannelAdmin,
}: CategoryGroupProps) {
	const channelList = (
		<ul className="flex flex-col gap-0.5">
			{group.channels.map((chan) => {
				const unread = unreadChannelIds.has(chan.id)
				return (
					<li key={chan.id} className="group/channel relative">
						<Link
							to={buildOrgPath(organizationSlug, `/chat/${chan.id}`)}
							className={cn(
								'flex items-center gap-1.5 rounded-md py-1.5 pr-8 pl-2 text-sm text-muted-foreground hover:bg-accent hover:text-accent-foreground',
								chan.id === activeChannelId &&
									'bg-accent font-medium text-accent-foreground',
								unread && 'font-semibold text-foreground',
							)}
						>
							<Hash className="size-3.5 shrink-0" />
							<span className="truncate">{chan.name}</span>
							{unread ? (
								<span
									aria-hidden="true"
									className="ml-auto size-1.5 shrink-0 rounded-full bg-primary"
								/>
							) : null}
							{unread ? <span className="sr-only">Non lu</span> : null}
						</Link>
						<DropdownMenu>
							<DropdownMenuTrigger asChild>
								<Button
									type="button"
									variant="ghost"
									size="icon"
									aria-label={`Gérer le canal ${chan.name}`}
									className="absolute top-1/2 right-1 size-6 -translate-y-1/2 opacity-0 focus-visible:opacity-100 group-hover/channel:opacity-100"
								>
									<MoreHorizontal className="size-3.5" />
								</Button>
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
			<CollapsibleTrigger className="flex w-full items-center gap-1 rounded-md px-2 py-1 text-xs font-semibold uppercase tracking-wide text-muted-foreground hover:text-foreground">
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
