import { Loader2 } from 'lucide-react'
import {
	Command,
	CommandEmpty,
	CommandInput,
	CommandItem,
	CommandList,
} from '#/components/ui/command'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import type { MemberItem } from '#/pages/chat/lib/members'
import { PersonAvatar } from '#/pages/chat/ui/person-avatar'

export interface NewDirectMessageDialogUIProps {
	open: boolean
	onOpenChange: (open: boolean) => void
	members: readonly MemberItem[]
	isSubmitting: boolean
	onSelect: (userId: string) => void
}

export function NewDirectMessageDialogUI({
	open,
	onOpenChange,
	members,
	isSubmitting,
	onSelect,
}: NewDirectMessageDialogUIProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="gap-0 p-0">
				<DialogHeader className="px-4 pt-4 pb-2">
					<DialogTitle>Nouveau message direct</DialogTitle>
					<DialogDescription>
						Choisissez la personne à qui écrire.
					</DialogDescription>
				</DialogHeader>
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
			</DialogContent>
		</Dialog>
	)
}
