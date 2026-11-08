import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import type { MemberItem } from '#/pages/chat/lib/members'
import { PRESENCE_LABELS, PRESENCE_ORDER } from '#/pages/chat/lib/presence'
import { PersonAvatar } from '#/pages/chat/ui/person-avatar'

export interface ChannelMembersDialogUIProps {
	open: boolean
	onOpenChange: (open: boolean) => void
	channelLabel: string
	members: readonly MemberItem[]
	currentUserId: string | null
}

/** Everyone in a channel, grouped by how reachable they are. */
export function ChannelMembersDialogUI({
	open,
	onOpenChange,
	channelLabel,
	members,
	currentUserId,
}: ChannelMembersDialogUIProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="gap-0 p-0">
				<DialogHeader className="px-4 pt-4 pb-2">
					<DialogTitle>
						Membres{' '}
						<span className="font-normal text-muted-foreground">
							· {members.length}
						</span>
					</DialogTitle>
					<DialogDescription>{channelLabel}</DialogDescription>
				</DialogHeader>
				<div className="max-h-96 overflow-y-auto pb-3">
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
				</div>
			</DialogContent>
		</Dialog>
	)
}
