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
import type { TableColumn, ViewKind } from '#/pages/knowledge-tables/lib/types'
import { NativeSelect } from '#/pages/knowledge-tables/ui/native-select'

export interface ViewDraft {
	name: string
	kind: ViewKind
	groupBy: string
}

interface ViewDialogProps {
	open: boolean
	mode: 'create' | 'rename'
	draft: ViewDraft
	groupable: readonly TableColumn[]
	error: string | null
	isSaving: boolean
	onDraftChange: (change: Partial<ViewDraft>) => void
	onSubmit: () => void
	onOpenChange: (open: boolean) => void
}

export function ViewDialog({
	open,
	mode,
	draft,
	groupable,
	error,
	isSaving,
	onDraftChange,
	onSubmit,
	onOpenChange,
}: ViewDialogProps) {
	const baseId = useId()
	const needsGroup = mode === 'create' && draft.kind === 'kanban'
	const cannotGroup = needsGroup && groupable.length === 0
	const isIncomplete =
		draft.name.trim() === '' || (needsGroup && draft.groupBy === '')

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
							{mode === 'create' ? 'Nouvelle vue' : 'Renommer la vue'}
						</DialogTitle>
						<DialogDescription>
							{mode === 'create'
								? 'Une vue garde ses propres filtres, tris et colonnes masquées.'
								: 'La vue garde ses filtres, ses tris et ses colonnes.'}
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
							onChange={(event) => onDraftChange({ name: event.target.value })}
						/>
					</div>
					{mode === 'create' ? (
						<fieldset className="grid gap-1.5">
							<legend className="mb-1.5 font-medium text-sm">Type</legend>
							<div className="flex gap-4 text-sm">
								{(
									[
										['grid', 'Grille'],
										['kanban', 'Kanban'],
									] as const
								).map(([kind, label]) => (
									<label key={kind} className="flex items-center gap-2">
										<input
											type="radio"
											name={`${baseId}-kind`}
											value={kind}
											checked={draft.kind === kind}
											onChange={() =>
												onDraftChange({
													kind,
													groupBy:
														kind === 'kanban'
															? ((draft.groupBy || groupable[0]?.id) ?? '')
															: draft.groupBy,
												})
											}
										/>
										{label}
									</label>
								))}
							</div>
						</fieldset>
					) : null}
					{needsGroup ? (
						<div className="grid gap-1.5">
							<Label htmlFor={`${baseId}-group`}>Grouper par</Label>
							{cannotGroup ? (
								<p className="text-muted-foreground text-sm">
									Un Kanban se groupe par une colonne de choix unique : ajoute
									d’abord une telle colonne à la table.
								</p>
							) : (
								<NativeSelect
									id={`${baseId}-group`}
									value={draft.groupBy}
									onChange={(event) =>
										onDraftChange({ groupBy: event.target.value })
									}
								>
									{groupable.map((column) => (
										<option key={column.id} value={column.id}>
											{column.name}
										</option>
									))}
								</NativeSelect>
							)}
						</div>
					) : null}
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
							disabled={isSaving || isIncomplete || cannotGroup}
						>
							{mode === 'create' ? 'Créer' : 'Renommer'}
						</Button>
					</DialogFooter>
				</form>
			</DialogContent>
		</Dialog>
	)
}
