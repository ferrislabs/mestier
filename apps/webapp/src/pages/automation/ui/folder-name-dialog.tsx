import { Loader2 } from 'lucide-react'
import { TextField } from '#/components/reference-table'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'

export interface FolderNameDialogProps {
	open: boolean
	title: string
	submitLabel: string
	name: string
	isPending: boolean
	error: string | null
	onNameChange: (name: string) => void
	onOpenChange: (open: boolean) => void
	onSubmit: () => void
}

export function FolderNameDialog({
	open,
	title,
	submitLabel,
	name,
	isPending,
	error,
	onNameChange,
	onOpenChange,
	onSubmit,
}: FolderNameDialogProps) {
	const canSubmit = name.trim().length > 0 && !isPending

	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent>
				<form
					className="flex flex-col gap-4"
					onSubmit={(event) => {
						event.preventDefault()
						if (canSubmit) onSubmit()
					}}
				>
					<DialogHeader>
						<DialogTitle>{title}</DialogTitle>
						<DialogDescription>
							Un dossier regroupe des workflows. Il s’affiche comme un projet.
						</DialogDescription>
					</DialogHeader>
					<TextField
						label="Nom du dossier"
						value={name}
						onChange={onNameChange}
						maxLength={80}
						autoFocus
					/>
					{error ? <p className="text-sm text-destructive">{error}</p> : null}
					<DialogFooter>
						<Button
							type="button"
							variant="outline"
							onClick={() => onOpenChange(false)}
						>
							Annuler
						</Button>
						<Button type="submit" disabled={!canSubmit}>
							{isPending ? <Loader2 className="animate-spin" /> : null}
							{submitLabel}
						</Button>
					</DialogFooter>
				</form>
			</DialogContent>
		</Dialog>
	)
}
