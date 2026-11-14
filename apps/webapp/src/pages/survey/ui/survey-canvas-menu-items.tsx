import { Flag, PanelsTopLeft, Trash2 } from 'lucide-react'
import {
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuShortcut,
} from '#/components/ui/dropdown-menu'
import type { ShortcutLabels } from '#/pages/automation/lib/shortcuts'

export function SurveyNodeMenuItems({
	readOnly,
	canMakeStart,
	shortcuts,
	onOpen,
	onMakeStart,
	onDelete,
}: {
	readOnly: boolean
	canMakeStart: boolean
	shortcuts: ShortcutLabels
	onOpen: () => void
	onMakeStart: () => void
	onDelete: () => void
}) {
	return (
		<>
			<DropdownMenuItem onSelect={onOpen}>
				<PanelsTopLeft />
				Ouvrir
				<DropdownMenuShortcut>{shortcuts.open}</DropdownMenuShortcut>
			</DropdownMenuItem>
			{readOnly ? null : (
				<>
					{canMakeStart ? (
						<DropdownMenuItem onSelect={onMakeStart}>
							<Flag />
							Définir comme départ
						</DropdownMenuItem>
					) : null}
					<DropdownMenuSeparator />
					<DropdownMenuItem variant="destructive" onSelect={onDelete}>
						<Trash2 />
						Supprimer
						<DropdownMenuShortcut>{shortcuts.delete}</DropdownMenuShortcut>
					</DropdownMenuItem>
				</>
			)}
		</>
	)
}
