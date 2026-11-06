import {
	ClipboardPaste,
	Copy,
	CopyPlus,
	PanelsTopLeft,
	PencilLine,
	Plus,
	Power,
	PowerOff,
	Trash2,
} from 'lucide-react'
import {
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuShortcut,
} from '#/components/ui/dropdown-menu'
import type { ShortcutLabels } from '#/pages/automation/lib/shortcuts'

export function PaneMenuItems({
	pasteLabel,
	shortcuts,
	onAddNode,
	onPaste,
}: {
	pasteLabel: string | null
	shortcuts: ShortcutLabels
	onAddNode: () => void
	onPaste: () => void
}) {
	return (
		<>
			<DropdownMenuItem onSelect={onAddNode}>
				<Plus />
				Ajouter un nœud
			</DropdownMenuItem>
			{pasteLabel && (
				<DropdownMenuItem onSelect={onPaste}>
					<ClipboardPaste />
					<span className="truncate">Coller « {pasteLabel} »</span>
					<DropdownMenuShortcut>{shortcuts.paste}</DropdownMenuShortcut>
				</DropdownMenuItem>
			)}
		</>
	)
}

export function NodeMenuItems({
	canToggleDisabled,
	disabled,
	shortcuts,
	onOpen,
	onRename,
	onToggleDisabled,
	onCopy,
	onDuplicate,
	onDelete,
}: {
	canToggleDisabled: boolean
	disabled: boolean
	shortcuts: ShortcutLabels
	onOpen: () => void
	onRename: () => void
	onToggleDisabled: () => void
	onCopy: () => void
	onDuplicate: () => void
	onDelete: () => void
}) {
	return (
		<>
			<DropdownMenuItem onSelect={onOpen}>
				<PanelsTopLeft />
				Ouvrir
				<DropdownMenuShortcut>{shortcuts.open}</DropdownMenuShortcut>
			</DropdownMenuItem>
			<DropdownMenuItem onSelect={onRename}>
				<PencilLine />
				Renommer
				<DropdownMenuShortcut>{shortcuts.rename}</DropdownMenuShortcut>
			</DropdownMenuItem>
			{canToggleDisabled && (
				<DropdownMenuItem onSelect={onToggleDisabled}>
					{disabled ? <Power /> : <PowerOff />}
					{disabled ? 'Réactiver' : 'Désactiver'}
					<DropdownMenuShortcut>
						{shortcuts['toggle-disabled']}
					</DropdownMenuShortcut>
				</DropdownMenuItem>
			)}
			<DropdownMenuSeparator />
			<DropdownMenuItem onSelect={onCopy}>
				<Copy />
				Copier
				<DropdownMenuShortcut>{shortcuts.copy}</DropdownMenuShortcut>
			</DropdownMenuItem>
			<DropdownMenuItem onSelect={onDuplicate}>
				<CopyPlus />
				Dupliquer
				<DropdownMenuShortcut>{shortcuts.duplicate}</DropdownMenuShortcut>
			</DropdownMenuItem>
			<DropdownMenuSeparator />
			<DropdownMenuItem variant="destructive" onSelect={onDelete}>
				<Trash2 />
				Supprimer
				<DropdownMenuShortcut>{shortcuts.delete}</DropdownMenuShortcut>
			</DropdownMenuItem>
		</>
	)
}
