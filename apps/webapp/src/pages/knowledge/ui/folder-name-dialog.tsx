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

interface FolderNameDialogProps {
	open: boolean
	title: string
	description: string
	submitLabel: string
	name: string
	error: string | null
	isSaving: boolean
	onNameChange: (name: string) => void
	onSubmit: () => void
	onOpenChange: (open: boolean) => void
}

export function FolderNameDialog({
	open,
	title,
	description,
	submitLabel,
	name,
	error,
	isSaving,
	onNameChange,
	onSubmit,
	onOpenChange,
}: FolderNameDialogProps) {
	const inputId = useId()
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
						<DialogTitle>{title}</DialogTitle>
						<DialogDescription>{description}</DialogDescription>
					</DialogHeader>
					<div className="grid gap-1.5">
						<Label htmlFor={inputId}>Nom du dossier</Label>
						<Input
							id={inputId}
							value={name}
							maxLength={100}
							autoComplete="off"
							autoFocus
							aria-invalid={error !== null}
							onChange={(event) => onNameChange(event.target.value)}
						/>
						{error ? (
							<p role="alert" className="text-[13px] text-state-alert">
								{error}
							</p>
						) : null}
					</div>
					<DialogFooter>
						<Button
							type="button"
							variant="outline"
							onClick={() => onOpenChange(false)}
						>
							Annuler
						</Button>
						<Button type="submit" disabled={isSaving || name.trim() === ''}>
							{submitLabel}
						</Button>
					</DialogFooter>
				</form>
			</DialogContent>
		</Dialog>
	)
}
