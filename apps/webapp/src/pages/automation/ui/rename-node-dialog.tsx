import { useId, useState } from 'react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'

export function RenameNodeDialog({
	initialName,
	validate,
	onConfirm,
	onCancel,
}: {
	initialName: string
	validate: (name: string) => string | null
	onConfirm: (name: string) => void
	onCancel: () => void
}) {
	const [name, setName] = useState(initialName)
	const inputId = useId()
	const trimmed = name.trim()
	const error = trimmed === initialName ? null : validate(trimmed)

	return (
		<Dialog
			open
			onOpenChange={(open) => {
				if (!open) onCancel()
			}}
		>
			<DialogContent showCloseButton={false} className="sm:max-w-sm">
				<DialogHeader>
					<DialogTitle>Renommer le nœud</DialogTitle>
				</DialogHeader>
				<form
					className="grid gap-2"
					onSubmit={(event) => {
						event.preventDefault()
						if (trimmed && !error) onConfirm(trimmed)
					}}
				>
					<Label htmlFor={inputId}>Nom du nœud</Label>
					<Input
						id={inputId}
						value={name}
						onChange={(event) => setName(event.target.value)}
						onFocus={(event) => event.target.select()}
						aria-invalid={error ? true : undefined}
						autoFocus
					/>
					{error && <p className="text-sm text-state-alert">{error}</p>}
					<DialogFooter className="mt-3">
						<Button type="button" variant="outline" onClick={onCancel}>
							Annuler
						</Button>
						<Button type="submit" disabled={!trimmed || error !== null}>
							Renommer
						</Button>
					</DialogFooter>
				</form>
			</DialogContent>
		</Dialog>
	)
}
