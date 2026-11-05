import { Link } from '@tanstack/react-router'
import { Skeleton } from '#/components/ui/skeleton'
import { cn } from '#/lib/utils'
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
		<section
			aria-label="Discussions"
			className="overflow-hidden rounded-xl border bg-card"
		>
			<header className="flex items-center justify-between border-b px-4 py-2.5">
				<h3 className="font-medium text-[13px]">Discussions</h3>
				<Link
					to={buildOrgPath(organizationSlug, '/chat')}
					className="text-[12px] text-primary outline-none hover:underline focus-visible:ring-2 focus-visible:ring-ring"
				>
					Ouvrir le chat →
				</Link>
			</header>

			{isLoading ? (
				<div className="flex flex-col gap-3 px-4 py-3" aria-busy="true">
					<Skeleton className="h-4 w-full" />
					<Skeleton className="h-4 w-5/6" />
					<Skeleton className="h-4 w-2/3" />
				</div>
			) : error ? (
				<p className="px-4 py-6 text-center text-[13px] text-destructive">
					{error}
				</p>
			) : channels.length === 0 ? (
				<p className="px-4 py-6 text-center text-[13px] text-muted-foreground">
					Aucun canal de discussion pour le moment.
				</p>
			) : (
				<ul className="divide-y">
					{channels.map((channel) => (
						<li key={channel.id} className="flex items-start gap-2.5 px-4 py-3">
							<span
								className={cn(
									'mt-1.5 size-1.5 shrink-0 rounded-full',
									channel.unread && 'bg-primary',
								)}
								aria-hidden="true"
							/>
							<div className="min-w-0">
								<p
									className={cn(
										'truncate text-[13.5px]',
										channel.unread ? 'font-semibold' : 'font-medium',
									)}
								>
									#{channel.name}
								</p>
								{channel.topic ? (
									<p className="truncate text-[12.5px] text-muted-foreground">
										{channel.topic}
									</p>
								) : null}
							</div>
						</li>
					))}
				</ul>
			)}
		</section>
	)
}
