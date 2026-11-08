import { Loader2 } from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Label } from '#/components/ui/label'
import type { FolderTarget } from '#/pages/automation/lib/folders'

export const ROOT_TARGET = ''

export interface FolderMoveDialogProps {
	open: boolean
	title: string
	targets: readonly FolderTarget[]
	value: string
	isPending: boolean
	error: string | null
	onValueChange: (value: string) => void
	onOpenChange: (open: boolean) => void
	onSubmit: () => void
}

export function FolderMoveDialog({
	open,
	title,
	targets,
	value,
	isPending,
	error,
	onValueChange,
	onOpenChange,
	onSubmit,
}: FolderMoveDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent>
				<DialogHeader>
					<DialogTitle>{title}</DialogTitle>
					<DialogDescription>
						Choisissez le dossier de destination.
					</DialogDescription>
				</DialogHeader>
				<div className="flex flex-col gap-2">
					<Label htmlFor="folder-target">Destination</Label>
					<select
						id="folder-target"
						value={value}
						onChange={(event) => onValueChange(event.target.value)}
						className="h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring"
					>
						<option value={ROOT_TARGET}>Aucun dossier (racine)</option>
						{targets.map((target) => (
							<option key={target.id} value={target.id}>
								{target.label}
							</option>
						))}
					</select>
				</div>
				{error ? <p className="text-sm text-destructive">{error}</p> : null}
				<DialogFooter>
					<Button variant="outline" onClick={() => onOpenChange(false)}>
						Annuler
					</Button>
					<Button disabled={isPending} onClick={onSubmit}>
						{isPending ? <Loader2 className="animate-spin" /> : null}
						Déplacer
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	)
}
