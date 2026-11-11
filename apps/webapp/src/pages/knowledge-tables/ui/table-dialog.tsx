import { useId } from 'react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import { Textarea } from '#/components/ui/textarea'

export interface TableDraft {
	name: string
	description: string
}

export const EMPTY_TABLE_DRAFT: TableDraft = { name: '', description: '' }

interface TableDialogProps {
	open: boolean
	mode: 'create' | 'rename'
	draft: TableDraft
	error: string | null
	isSaving: boolean
	onDraftChange: (change: Partial<TableDraft>) => void
	onSubmit: () => void
	onOpenChange: (open: boolean) => void
}

export function TableDialog({
	open,
	mode,
	draft,
	error,
	isSaving,
	onDraftChange,
	onSubmit,
	onOpenChange,
}: TableDialogProps) {
	const baseId = useId()
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent>
				<form
					className="grid gap-4"
					onSubmit={(event) => {
						event.preventDefault()
						onSubmit()
					}}
				>
					<DialogHeader>
						<DialogTitle>
							{mode === 'create' ? 'Nouvelle table' : 'Modifier la table'}
						</DialogTitle>
						<DialogDescription>
							{mode === 'create'
								? 'La table commence avec une colonne « Nom » et une vue Grille.'
								: 'Les colonnes, les vues et les lignes ne changent pas.'}
						</DialogDescription>
					</DialogHeader>
					<div className="grid gap-1.5">
						<Label htmlFor={`${baseId}-name`}>Nom</Label>
						<Input
							id={`${baseId}-name`}
							value={draft.name}
							maxLength={100}
							autoComplete="off"
							autoFocus
							placeholder="Suivi des chantiers"
							onChange={(event) => onDraftChange({ name: event.target.value })}
						/>
					</div>
					<div className="grid gap-1.5">
						<Label htmlFor={`${baseId}-description`}>Description</Label>
						<Textarea
							id={`${baseId}-description`}
							value={draft.description}
							maxLength={2000}
							rows={3}
							placeholder="À quoi sert cette table"
							onChange={(event) =>
								onDraftChange({ description: event.target.value })
							}
						/>
					</div>
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
							type="submit"
							disabled={isSaving || draft.name.trim() === ''}
						>
							{mode === 'create' ? 'Créer' : 'Enregistrer'}
						</Button>
					</DialogFooter>
				</form>
			</DialogContent>
		</Dialog>
	)
}
