import { Link } from '@tanstack/react-router'
import { Skeleton } from '#/components/ui/skeleton'
import { buildOrgPath } from '#/modules/org-path'

export interface DiscussionChannelRow {
	id: string
	name: string
	/** The channel's own topic, when one is set — never a message preview:
	 * the API has no per-channel "last message" snapshot, only a presence
	 * list of channels with something unread (see `useUnreadChannels`'s own
	 * doc). Showing a fabricated snippet here would be worse than showing
	 * none. */
	topic: string | null
	unread: boolean
}

export interface DiscussionsUIProps {
	organizationSlug: string
	channels: DiscussionChannelRow[]
	isLoading: boolean
	error: string | null
}

/**
 * A handful of channels, unread ones first — an entry point into the chat,
 * not an inbox preview (see `DiscussionChannelRow.topic`'s own doc for why
 * there is no message snippet here).
 */
export function DiscussionsUI({
	organizationSlug,
	channels,
	isLoading,
	error,
}: DiscussionsUIProps) {
	return (
		<div className="flex flex-col rounded-xl bg-card p-5 shadow-sm">
			<div className="mb-1 flex items-baseline justify-between gap-4">
				<span className="island-kicker">Discussions</span>
				<Link
					to={buildOrgPath(organizationSlug, '/chat')}
					className="text-xs font-semibold text-brand-muted"
				>
					Ouvrir le chat →
				</Link>
			</div>

			{isLoading ? (
				<div className="flex flex-col gap-3 py-3" aria-busy="true">
					<Skeleton className="h-4 w-full" />
					<Skeleton className="h-4 w-5/6" />
					<Skeleton className="h-4 w-2/3" />
				</div>
			) : error ? (
				<p className="py-3 text-sm text-destructive">{error}</p>
			) : channels.length === 0 ? (
				<p className="py-3 text-sm text-muted-foreground">
					Aucun canal de discussion pour le moment.
				</p>
			) : (
				<ul>
					{channels.map((channel) => (
						<li
							key={channel.id}
							className="flex items-start gap-2.5 border-t py-3 first:border-t-0"
						>
							<span
								className={
									channel.unread
										? 'mt-1.5 size-1.5 shrink-0 rounded-full bg-brand-500'
										: 'mt-1.5 size-1.5 shrink-0'
								}
								aria-hidden="true"
							/>
							<div className="min-w-0">
								<p
									className={
										channel.unread
											? 'truncate text-sm font-semibold text-foreground'
											: 'truncate text-sm font-medium text-foreground'
									}
								>
									#{channel.name}
								</p>
								{channel.topic ? (
									<p className="truncate text-xs text-muted-foreground">
										{channel.topic}
									</p>
								) : null}
							</div>
						</li>
					))}
				</ul>
			)}
		</div>
	)
}
