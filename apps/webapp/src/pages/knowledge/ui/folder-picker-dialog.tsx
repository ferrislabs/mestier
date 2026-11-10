import { Folder, HardDrive } from 'lucide-react'
import type { ReactNode } from 'react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import type { KnowledgeFolder } from '#/hooks/use-knowledge'
import { cn } from '#/lib/utils'
import { childrenByParent } from '#/pages/knowledge/lib/folders'

interface FolderPickerDialogProps {
	open: boolean
	title: string
	description: string
	folders: readonly KnowledgeFolder[]
	/** `null` is the root. */
	selected: string | null
	/** Destinations that would break a rule, shown but not selectable. */
	isAllowed: (folderId: string | null) => boolean
	error: string | null
	isSaving: boolean
	onSelect: (folderId: string | null) => void
	onConfirm: () => void
	onOpenChange: (open: boolean) => void
}

export function FolderPickerDialog({
	open,
	title,
	description,
	folders,
	selected,
	isAllowed,
	error,
	isSaving,
	onSelect,
	onConfirm,
	onOpenChange,
}: FolderPickerDialogProps) {
	const byParent = childrenByParent(folders)

	const option = (
		folderId: string | null,
		label: string,
		depth: number,
		icon: typeof Folder,
	) => {
		const allowed = isAllowed(folderId)
		const Icon = icon
		return (
			<button
				type="button"
				aria-pressed={selected === folderId}
				disabled={!allowed}
				className={cn(
					'flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-sm hover:bg-muted disabled:cursor-not-allowed disabled:opacity-50',
					selected === folderId &&
						'bg-primary/10 font-medium ring-1 ring-primary',
				)}
				style={{ paddingLeft: 8 + depth * 16 }}
				onClick={() => onSelect(folderId)}
			>
				<Icon aria-hidden className="size-4 shrink-0 text-muted-foreground" />
				<span className="truncate">{label}</span>
			</button>
		)
	}

	const branch = (parentId: string | null, depth: number): ReactNode =>
		(byParent.get(parentId) ?? []).map((folder) => (
			<div key={folder.id}>
				{option(folder.id, folder.name, depth, Folder)}
				{branch(folder.id, depth + 1)}
			</div>
		))

	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent>
				<DialogHeader>
					<DialogTitle>{title}</DialogTitle>
					<DialogDescription>{description}</DialogDescription>
				</DialogHeader>
				<fieldset className="grid max-h-80 gap-0.5 overflow-y-auto rounded-md border p-1">
					<legend className="sr-only">Destination</legend>
					{option(null, 'Drive (racine)', 0, HardDrive)}
					{branch(null, 1)}
				</fieldset>
				{error ? (
					<p role="alert" className="text-[13px] text-state-alert">
						{error}
					</p>
				) : null}
				<DialogFooter>
					<Button
						type="button"
						variant="outline"
						onClick={() => onOpenChange(false)}
					>
						Annuler
					</Button>
					<Button
						type="button"
						disabled={isSaving || !isAllowed(selected)}
						onClick={onConfirm}
					>
						Déplacer ici
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	)
}
