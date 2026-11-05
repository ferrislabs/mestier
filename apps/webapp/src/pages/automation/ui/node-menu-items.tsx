import { PanelsTopLeft, Trash2 } from 'lucide-react'
import {
	DropdownMenuItem,
	DropdownMenuSeparator,
} from '#/components/ui/dropdown-menu'

export function NodeMenuItems({
	onOpen,
	onDelete,
}: {
	onOpen: () => void
	onDelete: () => void
}) {
	return (
		<>
			<DropdownMenuItem onSelect={onOpen}>
				<PanelsTopLeft />
				Ouvrir
			</DropdownMenuItem>
			<DropdownMenuSeparator />
			<DropdownMenuItem variant="destructive" onSelect={onDelete}>
				<Trash2 />
				Supprimer
			</DropdownMenuItem>
		</>
	)
}
