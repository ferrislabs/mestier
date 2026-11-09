import { Link } from '@tanstack/react-router'
import type { DragEvent, ReactNode } from 'react'
import {
	AlertCircle,
	ChevronRight,
	Copy,
	Bell,
	Fingerprint,
	Plus,
	Settings,
	Trash2,
} from 'lucide-react'
import { DoubleClickConfirmButton } from '#/components/double-click-confirm-button'
import { RequirePermission } from '#/components/require-permission'
import {
	Collapsible,
	CollapsibleContent,
	CollapsibleTrigger,
} from '#/components/ui/collapsible'
import {
	ContextMenu,
	ContextMenuContent,
	ContextMenuItem,
	ContextMenuRadioGroup,
	ContextMenuRadioItem,
	ContextMenuSeparator,
	ContextMenuSub,
	ContextMenuSubContent,
	ContextMenuSubTrigger,
	ContextMenuTrigger,
} from '#/components/ui/context-menu'
import { Skeleton } from '#/components/ui/skeleton'
import type { NotificationLevel } from '#/hooks/use-chat'
import { cn } from '#/lib/utils'
import { buildOrgPath } from '#/modules/org-path'
import { channelKindIcon } from '#/pages/chat/lib/channel-kinds'
import {
	effectiveNotificationLevel,
	NOTIFICATION_LEVELS,
} from '#/pages/chat/lib/notification-levels'
import type { DirectMessageItem } from '#/pages/chat/lib/direct-messages'
import type { ChannelGroup } from '#/pages/chat/lib/group-channels'
import type { DropHint } from '#/pages/chat/lib/reorder-channels'
import { PersonAvatar } from '#/pages/chat/ui/person-avatar'

const ITEM = 'gap-2 px-2 py-1 text-xs [&_svg]:size-3.5'

/** Dragging a channel to reorder it or to move it to another category. */
export interface ChannelDnd {
	draggingId: string | null
	hint: DropHint | null
	onDragStart: (channelId: string) => void
	onDragEnd: () => void
	onHint: (hint: DropHint) => void
	onDrop: () => void
}

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
	onRequestNewChannelInCategory: (categoryId: string) => void
	/** The "+" of the direct messages section, with the member picker it opens. */
	newDirectMessage: ReactNode
	/** Opens the channel admin sheet for one channel — see #372: channel
	 * management starts here, not only from inside the channel itself. */
	onOpenChannelAdmin: (channelId: string) => void
	onDuplicateChannel: (channelId: string) => void
	/** The levels the person chose per channel; the others have the default. */
	notificationLevels: ReadonlyMap<string, NotificationLevel>
	onChangeNotificationLevel: (
		channelId: string,
		level: NotificationLevel,
	) => void
	onDeleteChannel: (channelId: string) => void
	/** Set only for someone who may manage channels. */
	dnd?: ChannelDnd
}

const headingClassName =
	'font-semibold text-nav-muted text-xs uppercase tracking-wide'

const linkClassName =
	'flex h-8 items-center gap-2 rounded-md px-2 text-nav-foreground/80 text-sm outline-none transition-colors hover:bg-nav-active/60 hover:text-nav-foreground focus-visible:ring-2 focus-visible:ring-nav-primary data-[status=active]:bg-nav-active data-[status=active]:text-nav-active-foreground'

export const addButtonClassName =
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
	onRequestNewChannelInCategory,
	newDirectMessage,
	onOpenChannelAdmin,
	onDuplicateChannel,
	notificationLevels,
	onChangeNotificationLevel,
	onDeleteChannel,
	dnd,
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
								onDuplicateChannel={onDuplicateChannel}
								notificationLevels={notificationLevels}
								onChangeNotificationLevel={onChangeNotificationLevel}
								onDeleteChannel={onDeleteChannel}
								onRequestNewChannelInCategory={onRequestNewChannelInCategory}
								dnd={dnd}
							/>
						))}
						{dnd?.draggingId && !groups.some((group) => !group.category) ? (
							<div
								onDragOver={(event) => {
									event.preventDefault()
									dnd.onHint({ kind: 'category', categoryId: null })
								}}
								onDrop={(event) => {
									event.preventDefault()
									dnd.onDrop()
								}}
								className={cn(
									'rounded-md border border-nav-border border-dashed px-2 py-1.5 text-nav-muted text-xs',
									dnd.hint?.kind === 'category' &&
										dnd.hint.categoryId === null &&
										'bg-nav-active/60 text-nav-foreground',
								)}
							>
								Sans catégorie
							</div>
						) : null}
					</div>
				) : null}
			</section>

			<section aria-labelledby="discussions-direct-heading">
				<div className="flex items-center justify-between px-2 pb-1">
					<h2 id="discussions-direct-heading" className={headingClassName}>
						Messages directs
					</h2>
					{newDirectMessage}
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

function ChannelIcon({ type, className }: { type: string; className: string }) {
	const Icon = channelKindIcon(type)
	return <Icon aria-hidden className={className} />
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
	onDuplicateChannel: (channelId: string) => void
	/** The levels the person chose per channel; the others have the default. */
	notificationLevels: ReadonlyMap<string, NotificationLevel>
	onChangeNotificationLevel: (
		channelId: string,
		level: NotificationLevel,
	) => void
	onDeleteChannel: (channelId: string) => void
	onRequestNewChannelInCategory: (categoryId: string) => void
	dnd?: ChannelDnd
}

function CategoryGroup({
	group,
	organizationSlug,
	activeChannelId,
	collapsed,
	onToggleCategory,
	unreadCounts,
	onOpenChannelAdmin,
	onDuplicateChannel,
	notificationLevels,
	onChangeNotificationLevel,
	onDeleteChannel,
	onRequestNewChannelInCategory,
	dnd,
}: CategoryGroupProps) {
	const channelList = (
		<ul className="flex flex-col gap-0.5">
			{group.channels.map((chan) => {
				const unreadCount = unreadCounts.get(chan.id) ?? 0
				return (
					<ContextMenu key={chan.id}>
						<ContextMenuTrigger asChild>
							<li
								className={cn(
									'group/channel relative',
									dnd?.draggingId === chan.id && 'opacity-40',
									dnd?.hint?.kind === 'before' &&
										dnd.hint.channelId === chan.id &&
										'shadow-[inset_0_2px_0_0_var(--nav-primary)]',
									dnd?.hint?.kind === 'after' &&
										dnd.hint.channelId === chan.id &&
										'shadow-[inset_0_-2px_0_0_var(--nav-primary)]',
								)}
								draggable={dnd ? true : undefined}
								onDragStart={(event) => {
									if (!dnd) return
									event.dataTransfer.effectAllowed = 'move'
									event.dataTransfer.setData('text/plain', chan.id)
									dnd.onDragStart(chan.id)
								}}
								onDragEnd={dnd?.onDragEnd}
								onDragOver={(event) => {
									if (!dnd?.draggingId) return
									event.preventDefault()
									event.stopPropagation()
									const box = event.currentTarget.getBoundingClientRect()
									dnd.onHint({
										kind:
											event.clientY < box.top + box.height / 2
												? 'before'
												: 'after',
										channelId: chan.id,
									})
								}}
								onDrop={(event) => {
									if (!dnd?.draggingId) return
									event.preventDefault()
									event.stopPropagation()
									dnd.onDrop()
								}}
							>
								<Link
									to={buildOrgPath(organizationSlug, `/discussions/${chan.id}`)}
									draggable={dnd ? false : undefined}
									data-status={
										chan.id === activeChannelId ? 'active' : undefined
									}
									className={cn(
										linkClassName,
										'pr-8',
										unreadCount > 0 && 'font-semibold text-nav-foreground',
									)}
								>
									<ChannelIcon
										type={chan.channel_type}
										className="size-4 shrink-0 text-nav-muted"
									/>
									<span className="min-w-0 flex-1 truncate">{chan.name}</span>
									<UnreadBadge count={unreadCount} />
								</Link>
								<button
									type="button"
									aria-label={`Paramètres du canal ${chan.name}`}
									onClick={() => onOpenChannelAdmin(chan.id)}
									className="absolute top-1/2 right-1 flex size-6 -translate-y-1/2 items-center justify-center rounded-md text-nav-muted opacity-0 outline-none hover:bg-nav-active hover:text-nav-foreground focus-visible:opacity-100 group-hover/channel:opacity-100"
								>
									<Settings className="size-3.5" />
								</button>
							</li>
						</ContextMenuTrigger>
						<ContextMenuContent className="w-56 min-w-0 p-0.5">
							<RequirePermission permission="MANAGE_CHANNELS">
								<ContextMenuItem
									className={ITEM}
									onSelect={() => onOpenChannelAdmin(chan.id)}
								>
									<Settings className="size-4" />
									Modifier le canal
								</ContextMenuItem>
								<ContextMenuItem
									className={ITEM}
									onSelect={() => onDuplicateChannel(chan.id)}
								>
									<Copy className="size-3.5" />
									Dupliquer le canal
								</ContextMenuItem>
							</RequirePermission>
							<ContextMenuItem
								className={ITEM}
								onSelect={() => navigator.clipboard?.writeText(chan.id)}
							>
								<Fingerprint className="size-3.5" />
								Copier l’identifiant du canal
							</ContextMenuItem>
							<ContextMenuSub>
								<ContextMenuSubTrigger className={ITEM}>
									<Bell className="size-3.5" />
									Notifications
								</ContextMenuSubTrigger>
								<ContextMenuSubContent className="w-52 min-w-0 p-0.5">
									<ContextMenuRadioGroup
										value={effectiveNotificationLevel(
											notificationLevels,
											chan.id,
										)}
										onValueChange={(level) =>
											onChangeNotificationLevel(
												chan.id,
												level as NotificationLevel,
											)
										}
									>
										{NOTIFICATION_LEVELS.map((option) => (
											<ContextMenuRadioItem
												key={option.value}
												value={option.value}
												className={ITEM}
											>
												{option.label}
											</ContextMenuRadioItem>
										))}
									</ContextMenuRadioGroup>
								</ContextMenuSubContent>
							</ContextMenuSub>
							<RequirePermission permission="MANAGE_CHANNELS">
								<ContextMenuSeparator className="my-0.5" />
								<div>
									<DoubleClickConfirmButton
										variant="ghost"
										size="sm"
										className="h-7 w-full justify-start gap-2 overflow-hidden px-2 text-destructive text-xs hover:text-destructive"
										icon={<Trash2 className="size-3.5" />}
										label="Supprimer le canal"
										confirmLabel="Confirmer la suppression"
										onConfirm={() => onDeleteChannel(chan.id)}
									/>
								</div>
							</RequirePermission>
						</ContextMenuContent>
					</ContextMenu>
				)
			})}
		</ul>
	)

	// The whole group takes a drop: the gaps between rows and the space below
	// the last one put the channel at the end of the category.
	const categoryId = group.category?.id ?? null
	const groupDrop = dnd
		? {
				onDragOver: (event: DragEvent<HTMLElement>) => {
					if (!dnd.draggingId) return
					event.preventDefault()
					dnd.onHint({ kind: 'category', categoryId })
				},
				onDrop: (event: DragEvent<HTMLElement>) => {
					if (!dnd.draggingId) return
					event.preventDefault()
					dnd.onDrop()
				},
			}
		: {}

	if (!group.category) {
		return <div {...groupDrop}>{channelList}</div>
	}

	return (
		<Collapsible
			{...groupDrop}
			open={!collapsed}
			onOpenChange={(open) => {
				if (group.category) onToggleCategory(group.category.id, !open)
			}}
		>
			<div
				className={cn(
					'group/category flex items-center rounded-md pr-1 pb-1',
					dnd?.hint?.kind === 'category' &&
						dnd.hint.categoryId === group.category.id &&
						'bg-nav-active/60',
				)}
			>
				<CollapsibleTrigger className="flex min-w-0 flex-1 items-center gap-1 rounded-md px-2 font-semibold text-nav-muted text-xs uppercase tracking-wide outline-none hover:text-nav-foreground focus-visible:ring-2 focus-visible:ring-nav-primary">
					<ChevronRight
						className={cn(
							'size-3.5 shrink-0 transition-transform',
							!collapsed && 'rotate-90',
						)}
					/>
					<span className="truncate">{group.category.name}</span>
				</CollapsibleTrigger>
				<RequirePermission permission="MANAGE_CHANNELS">
					<button
						type="button"
						aria-label={`Créer un canal dans ${group.category.name}`}
						className={cn(
							addButtonClassName,
							'opacity-0 focus-visible:opacity-100 group-hover/category:opacity-100',
						)}
						onClick={() => {
							if (group.category)
								onRequestNewChannelInCategory(group.category.id)
						}}
					>
						<Plus className="size-3.5" />
					</button>
				</RequirePermission>
			</div>
			<CollapsibleContent>{channelList}</CollapsibleContent>
		</Collapsible>
	)
}
