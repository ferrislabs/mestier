import { BellOff, CheckCheck, Loader2 } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { cn } from '#/lib/utils'
import type { ActivityFilter, ActivityItem } from '#/pages/chat/lib/activity'
import { PersonAvatar } from '#/pages/chat/ui/person-avatar'

const FILTERS: readonly { value: ActivityFilter; label: string }[] = [
	{ value: 'all', label: 'Tout' },
	{ value: 'mentions', label: 'Mentions' },
	{ value: 'unread', label: 'Non lus' },
]

export interface ActivityUIProps {
	items: readonly ActivityItem[]
	isLoading: boolean
	isError: boolean
	filter: ActivityFilter
	onChangeFilter: (filter: ActivityFilter) => void
	canMarkAllRead: boolean
	onMarkAllRead: () => void
	onOpen: (item: ActivityItem) => void
}

function reason(item: ActivityItem): string {
	if (item.kind === 'UNREAD') {
		const count = item.unreadCount ?? 1
		return count > 1 ? `${count} nouveaux messages` : 'Nouveau message'
	}
	return item.kind === 'REPLY' ? 'a répondu' : 'vous a mentionné'
}

export function ActivityUI({
	items,
	isLoading,
	isError,
	filter,
	onChangeFilter,
	canMarkAllRead,
	onMarkAllRead,
	onOpen,
}: ActivityUIProps) {
	return (
		<section className="mx-auto flex h-full w-full max-w-3xl min-h-0 flex-col gap-4 px-6 py-6">
			<header className="flex items-center justify-between gap-4">
				<h1 className="font-semibold text-xl">Activité</h1>
				<Button
					type="button"
					variant="outline"
					size="sm"
					disabled={!canMarkAllRead}
					onClick={onMarkAllRead}
				>
					<CheckCheck className="size-4" />
					Tout marquer comme lu
				</Button>
			</header>

			<div
				role="tablist"
				aria-label="Filtrer l’activité"
				className="flex gap-1"
			>
				{FILTERS.map((option) => (
					<button
						key={option.value}
						type="button"
						role="tab"
						aria-selected={filter === option.value}
						onClick={() => onChangeFilter(option.value)}
						className={cn(
							'rounded-md px-3 py-1.5 text-sm text-muted-foreground hover:bg-accent',
							filter === option.value &&
								'bg-accent font-medium text-foreground',
						)}
					>
						{option.label}
					</button>
				))}
			</div>

			<div className="min-h-0 flex-1 overflow-y-auto">
				{isLoading ? (
					<div className="flex justify-center py-10">
						<Loader2 className="size-5 animate-spin text-muted-foreground" />
					</div>
				) : null}
				{isError ? (
					<p
						role="alert"
						className="py-10 text-center text-destructive text-sm"
					>
						Impossible de charger l’activité.
					</p>
				) : null}
				{!isLoading && !isError && items.length === 0 ? (
					<div className="flex flex-col items-center gap-2 py-16 text-center">
						<BellOff className="size-6 text-muted-foreground" />
						<p className="font-medium text-sm">Rien à lire pour le moment</p>
						<p className="max-w-sm text-muted-foreground text-xs">
							Les mentions et les messages des canaux que vous suivez
							apparaissent ici. Réglez un canal par clic droit, puis
							Notifications.
						</p>
					</div>
				) : null}
				<ul className="flex flex-col divide-y rounded-lg border bg-card empty:hidden">
					{items.map((item) => (
						<li key={item.key}>
							<button
								type="button"
								onClick={() => onOpen(item)}
								className="flex w-full items-start gap-3 px-4 py-3 text-left outline-none hover:bg-accent/50 focus-visible:bg-accent/50"
							>
								<PersonAvatar
									name={item.authorName}
									hue={item.authorHue}
									size="md"
									surface="card"
								/>
								<span className="min-w-0 flex-1">
									<span className="flex items-baseline gap-2">
										<span
											className={cn(
												'truncate text-sm',
												!item.read && 'font-semibold',
											)}
										>
											{item.authorName}
										</span>
										<span className="truncate text-muted-foreground text-xs">
											{reason(item)} · {item.where}
										</span>
										<span className="ml-auto shrink-0 text-muted-foreground text-xs">
											{item.when}
										</span>
									</span>
									<span className="mt-0.5 line-clamp-2 break-words text-muted-foreground text-sm">
										{item.excerpt}
									</span>
								</span>
								{!item.read ? (
									<span
										role="img"
										aria-label="Non lu"
										className="mt-1.5 size-2 shrink-0 rounded-full bg-primary"
									/>
								) : null}
							</button>
						</li>
					))}
				</ul>
			</div>
		</section>
	)
}
