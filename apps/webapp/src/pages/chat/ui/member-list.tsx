import type { PresenceStatus } from '#/hooks/use-presence'
import { PRESENCE_LABELS, PRESENCE_ORDER } from '#/pages/chat/lib/presence'
import { PersonAvatar } from '#/pages/chat/ui/person-avatar'

export interface MemberItem {
	id: string
	name: string
	hue: number
	presence: PresenceStatus
}

export function MemberList({
	members,
	currentUserId,
}: {
	members: readonly MemberItem[]
	currentUserId: string | null
}) {
	return (
		<aside
			aria-label="Membres du canal"
			className="hidden w-60 shrink-0 flex-col overflow-y-auto border-border border-l bg-card xl:flex"
		>
			<h2 className="sticky top-0 border-border border-b bg-card px-4 py-3 font-semibold text-sm">
				Membres{' '}
				<span className="font-normal text-muted-foreground">
					· {members.length}
				</span>
			</h2>
			{PRESENCE_ORDER.map((status) => {
				const group = members.filter((member) => member.presence === status)
				if (group.length === 0) return null
				return (
					<section
						key={status}
						aria-label={`${PRESENCE_LABELS[status]}, ${group.length}`}
						className="px-2 pt-3"
					>
						<h3 className="px-2 pb-1 font-semibold text-muted-foreground text-xs uppercase tracking-wide">
							{PRESENCE_LABELS[status]} · {group.length}
						</h3>
						<ul>
							{group.map((member) => (
								<li
									key={member.id}
									className="flex items-center gap-2.5 rounded-md px-2 py-1.5 hover:bg-accent"
								>
									<PersonAvatar
										name={member.name}
										hue={member.hue}
										size="sm"
										presence={member.presence}
										surface="card"
									/>
									<p className="min-w-0 truncate font-medium text-sm">
										{member.name}
										{member.id === currentUserId ? (
											<span className="font-normal text-muted-foreground">
												{' '}
												(vous)
											</span>
										) : null}
									</p>
								</li>
							))}
						</ul>
					</section>
				)
			})}
		</aside>
	)
}
