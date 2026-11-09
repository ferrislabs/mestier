import {
	Bookmark,
	Ellipsis,
	Link2,
	MessageSquareReply,
	Pencil,
	Pin,
	SmilePlus,
	Trash2,
} from 'lucide-react'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import { EmojiPopover } from '#/pages/chat/ui/emoji-popover'
import { IconButton } from '#/pages/chat/ui/icon-button'

export function MessageToolbar({
	threadsEnabled = true,
	hasThread,
	isOwn,
	pinned,
	saved,
	onReact,
	onOpenThread,
	onCopyLink,
	onTogglePin,
	onToggleSaved,
	onEdit,
	onDelete,
}: {
	threadsEnabled?: boolean
	hasThread: boolean
	isOwn: boolean
	pinned: boolean
	saved: boolean
	onReact: (emoji: string) => void
	onOpenThread: () => void
	onCopyLink: () => void
	onTogglePin: () => void
	onToggleSaved: () => void
	onEdit: () => void
	onDelete: () => void
}) {
	return (
		<div
			role="toolbar"
			aria-label="Actions du message"
			className="absolute -top-3 right-4 flex items-center gap-0.5 rounded-md border border-border bg-card p-0.5 opacity-0 shadow-sm transition-opacity focus-within:opacity-100 group-hover:opacity-100 has-[[data-state=open]]:opacity-100"
		>
			<EmojiPopover onSelect={onReact} align="end">
				<IconButton label="Ajouter une réaction" icon={SmilePlus} />
			</EmojiPopover>
			{threadsEnabled ? (
				<IconButton
					label={hasThread ? 'Ouvrir le fil' : 'Répondre dans un fil'}
					icon={MessageSquareReply}
					onClick={onOpenThread}
				/>
			) : null}
			<DropdownMenu>
				<DropdownMenuTrigger asChild>
					<IconButton label="Plus d’actions" icon={Ellipsis} />
				</DropdownMenuTrigger>
				<DropdownMenuContent align="end">
					<DropdownMenuItem onSelect={onCopyLink}>
						<Link2 aria-hidden />
						Copier le lien
					</DropdownMenuItem>
					<DropdownMenuItem onSelect={onTogglePin}>
						<Pin aria-hidden />
						{pinned ? 'Désépingler le message' : 'Épingler le message'}
					</DropdownMenuItem>
					<DropdownMenuItem onSelect={onToggleSaved}>
						<Bookmark aria-hidden />
						{saved
							? 'Retirer des messages enregistrés'
							: 'Enregistrer pour plus tard'}
					</DropdownMenuItem>
					{isOwn ? (
						<>
							<DropdownMenuSeparator />
							<DropdownMenuItem onSelect={onEdit}>
								<Pencil aria-hidden />
								Modifier le message
							</DropdownMenuItem>
							<DropdownMenuItem variant="destructive" onSelect={onDelete}>
								<Trash2 aria-hidden />
								Supprimer le message
							</DropdownMenuItem>
						</>
					) : null}
				</DropdownMenuContent>
			</DropdownMenu>
		</div>
	)
}
