import { Check, Settings } from 'lucide-react'
import {
	Command,
	CommandEmpty,
	CommandGroup,
	CommandInput,
	CommandItem,
	CommandList,
} from '#/components/ui/command'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import { cn } from '#/lib/utils'
import type { SupportMemberVM } from '#/pages/support/types'
import { MemberAvatar } from '#/pages/support/ui/member-avatar'

export function AssigneeList({
	assignee,
}: {
	assignee: SupportMemberVM | null
}) {
	if (assignee === null)
		return (
			<p className="flex items-center gap-2 text-muted-foreground text-sm">
				<MemberAvatar member={null} size="sm" />
				Personne ne traite ce ticket
			</p>
		)
	return (
		<ul className="grid gap-1.5">
			<li className="flex items-center gap-2 text-sm">
				<MemberAvatar member={assignee} size="sm" />
				<span className="min-w-0 truncate">{assignee.name}</span>
			</li>
		</ul>
	)
}

export function AssigneeMenu({
	assigneeId,
	members,
	disabled = false,
	onChange,
}: {
	assigneeId: string | null
	members: readonly SupportMemberVM[]
	disabled?: boolean
	onChange: (memberId: string | null) => void
}) {
	return (
		<Popover>
			<PopoverTrigger asChild>
				<button
					type="button"
					disabled={disabled}
					aria-label="Modifier les assignés"
					title="Modifier les assignés"
					className="-my-1 -mr-1.5 rounded-md p-1 text-muted-foreground outline-none transition-colors hover:bg-accent hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50"
				>
					<Settings className="size-4" aria-hidden />
				</button>
			</PopoverTrigger>
			<PopoverContent align="end" className="w-64 p-0">
				<p className="border-border border-b px-3 py-2 font-medium text-sm">
					Assigner un membre
				</p>
				<Command>
					<CommandInput placeholder="Rechercher un membre…" />
					<CommandList>
						<CommandEmpty>Aucun membre trouvé.</CommandEmpty>
						<CommandGroup>
							{members.map((member) => {
								const selected = member.id === assigneeId
								return (
									<CommandItem
										key={member.id}
										value={member.name}
										onSelect={() => onChange(selected ? null : member.id)}
										aria-selected={selected}
									>
										<MemberAvatar member={member} size="sm" />
										<span className="min-w-0 flex-1">
											<span className="block truncate">{member.name}</span>
										</span>
										<Check
											className={cn(
												'size-4',
												selected ? 'opacity-100' : 'opacity-0',
											)}
											aria-hidden
										/>
									</CommandItem>
								)
							})}
						</CommandGroup>
					</CommandList>
				</Command>
			</PopoverContent>
		</Popover>
	)
}
