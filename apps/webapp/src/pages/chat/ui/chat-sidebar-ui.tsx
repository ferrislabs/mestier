import { Link } from '@tanstack/react-router'
import {
	AlertCircle,
	ChevronRight,
	Hash,
	MessageSquarePlus,
} from 'lucide-react'
import {
	Collapsible,
	CollapsibleContent,
	CollapsibleTrigger,
} from '#/components/ui/collapsible'
import { Skeleton } from '#/components/ui/skeleton'
import { cn } from '#/lib/utils'
import { buildOrgPath } from '#/modules/org-path'
import type { ChannelGroup } from '#/pages/chat/lib/group-channels'

export interface ChatSidebarUIProps {
	organizationSlug: string
	groups: ChannelGroup[]
	collapsedCategoryIds: ReadonlySet<string>
	onToggleCategory: (categoryId: string, collapsed: boolean) => void
	activeChannelId?: string
	isLoading: boolean
	isError: boolean
}

export function ChatSidebarUI({
	organizationSlug,
	groups,
	collapsedCategoryIds,
	onToggleCategory,
	activeChannelId,
	isLoading,
	isError,
}: ChatSidebarUIProps) {
	return (
		<nav
			aria-label="Canaux"
			className="flex h-full w-64 shrink-0 flex-col overflow-y-auto border-r bg-card/50"
		>
			<div className="px-4 py-4">
				<h2 className="text-sm font-semibold text-foreground">Discussions</h2>
			</div>

			{isLoading ? <SidebarSkeleton /> : null}

			{isError ? (
				<div className="flex items-start gap-2 px-4 py-2 text-sm text-destructive">
					<AlertCircle className="mt-0.5 size-4 shrink-0" />
					<span>Impossible de charger les canaux.</span>
				</div>
			) : null}

			{!isLoading && !isError && groups.length === 0 ? <EmptyState /> : null}

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

function EmptyState() {
	return (
		<div className="flex flex-col items-start gap-2 px-4 py-3 text-sm text-muted-foreground">
			<MessageSquarePlus className="size-5" />
			<p>
				Aucun canal pour le moment. Un administrateur peut en créer un depuis
				l’administration du canal.
			</p>
		</div>
	)
}

interface CategoryGroupProps {
	group: ChannelGroup
	organizationSlug: string
	activeChannelId?: string
	collapsed: boolean
	onToggleCategory: (categoryId: string, collapsed: boolean) => void
}

function CategoryGroup({
	group,
	organizationSlug,
	activeChannelId,
	collapsed,
	onToggleCategory,
}: CategoryGroupProps) {
	const channelList = (
		<ul className="flex flex-col gap-0.5">
			{group.channels.map((chan) => (
				<li key={chan.id}>
					<Link
						to={buildOrgPath(organizationSlug, `/chat/${chan.id}`)}
						className={cn(
							'flex items-center gap-1.5 rounded-md px-2 py-1.5 text-sm text-muted-foreground hover:bg-accent hover:text-accent-foreground',
							chan.id === activeChannelId &&
								'bg-accent font-medium text-accent-foreground',
						)}
					>
						<Hash className="size-3.5 shrink-0" />
						<span className="truncate">{chan.name}</span>
					</Link>
				</li>
			))}
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
