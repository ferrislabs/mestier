import { Loader2 } from 'lucide-react'
import type { ReactNode } from 'react'
import {
	Command,
	CommandEmpty,
	CommandInput,
	CommandItem,
	CommandList,
} from '#/components/ui/command'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import type { MemberItem } from '#/pages/chat/lib/members'
import { PersonAvatar } from '#/pages/chat/ui/person-avatar'

export interface NewDirectMessagePopoverUIProps {
	open: boolean
	onOpenChange: (open: boolean) => void
	members: readonly MemberItem[]
	isSubmitting: boolean
	onSelect: (userId: string) => void
	/** The button that opens the popover. */
	children: ReactNode
}

/** Pick who to write to: a search field over the members, anchored to the
 * "+" of the direct messages section. */
export function NewDirectMessagePopoverUI({
	open,
	onOpenChange,
	members,
	isSubmitting,
	onSelect,
	children,
}: NewDirectMessagePopoverUIProps) {
	return (
		<Popover open={open} onOpenChange={onOpenChange}>
			<PopoverTrigger asChild>{children}</PopoverTrigger>
			<PopoverContent align="start" className="w-64 p-0">
				<Command>
					<CommandInput placeholder="Rechercher un membre" />
					<CommandList>
						<CommandEmpty>Aucun membre trouvé.</CommandEmpty>
						{members.map((member) => (
							<CommandItem
								key={member.id}
								value={member.name}
								disabled={isSubmitting}
								onSelect={() => onSelect(member.id)}
							>
								<PersonAvatar
									name={member.name}
									hue={member.hue}
									size="sm"
									presence={member.presence}
									surface="card"
								/>
								<span className="min-w-0 flex-1 truncate">{member.name}</span>
							</CommandItem>
						))}
					</CommandList>
				</Command>
				{isSubmitting ? (
					<div className="flex justify-center border-border border-t py-2">
						<Loader2 className="size-4 animate-spin text-muted-foreground" />
					</div>
				) : null}
			</PopoverContent>
		</Popover>
	)
}
