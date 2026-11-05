import { PersonAvatar } from '#/pages/chat/ui/person-avatar'

export interface MemberItem {
	id: string
	name: string
	hue: number
}

export function MemberList({
	members,
	onlineCount,
}: {
	members: readonly MemberItem[]
	onlineCount: number
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
			{onlineCount > 0 ? (
				<p className="px-4 pt-3 font-semibold text-muted-foreground text-xs uppercase tracking-wide">
					{onlineCount} en ligne
				</p>
			) : null}
			<ul className="px-2 py-2">
				{members.map((member) => (
					<li
						key={member.id}
						className="flex items-center gap-2.5 rounded-md px-2 py-1.5 hover:bg-accent"
					>
						<PersonAvatar name={member.name} hue={member.hue} size="sm" />
						<p className="min-w-0 truncate font-medium text-sm">
							{member.name}
						</p>
					</li>
				))}
			</ul>
		</aside>
	)
}
