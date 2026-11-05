import { Loader2 } from 'lucide-react'
import { type FormEvent, useId, useState } from 'react'
import { FormField } from '#/components/form-field'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogClose,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import { ProjectTextarea } from '#/pages/projects/ui/project-textarea'

export function NewModelDialog({
	open,
	isPending,
	error,
	onOpenChange,
	onSubmit,
}: {
	open: boolean
	isPending: boolean
	error: string | null
	onOpenChange: (open: boolean) => void
	onSubmit: (values: { name: string; description: string }) => void
}) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="sm:max-w-md">
				<NewModelForm isPending={isPending} error={error} onSubmit={onSubmit} />
			</DialogContent>
		</Dialog>
	)
}

function NewModelForm({
	isPending,
	error,
	onSubmit,
}: {
	isPending: boolean
	error: string | null
	onSubmit: (values: { name: string; description: string }) => void
}) {
	const baseId = useId()
	const [name, setName] = useState('')
	const [description, setDescription] = useState('')
	const [submitted, setSubmitted] = useState(false)
	const nameError =
		submitted && name.trim() === ''
			? 'Le nom du modèle est obligatoire.'
			: undefined

	function submit(event: FormEvent) {
		event.preventDefault()
		if (name.trim() === '') {
			setSubmitted(true)
			document.getElementById(`${baseId}-name`)?.focus()
			return
		}
		onSubmit({ name, description })
	}

	return (
		<form onSubmit={submit} noValidate className="grid gap-4">
			<DialogHeader>
				<DialogTitle>Nouveau modèle de DOE</DialogTitle>
				<DialogDescription>
					Donne un nom au modèle. Tu ajouteras et régleras ses thèmes à l’étape
					suivante.
				</DialogDescription>
			</DialogHeader>
			<FormField id={`${baseId}-name`} label="Nom" required error={nameError}>
				<Input
					id={`${baseId}-name`}
					value={name}
					onChange={(event) => setName(event.target.value)}
					autoComplete="off"
					placeholder="Paysagiste, Gros œuvre…"
					aria-invalid={nameError ? true : undefined}
					aria-describedby={nameError ? `${baseId}-name-error` : undefined}
				/>
			</FormField>
			<FormField
				id={`${baseId}-description`}
				label="Description"
				hint="Facultative."
			>
				<ProjectTextarea
					id={`${baseId}-description`}
					rows={3}
					value={description}
					onChange={(event) => setDescription(event.target.value)}
				/>
			</FormField>
			{error ? (
				<p role="alert" className="text-[13px] text-state-alert">
					{error}
				</p>
			) : null}
			<DialogFooter>
				<DialogClose asChild>
					<Button type="button" variant="outline">
						Annuler
					</Button>
				</DialogClose>
				<Button type="submit" disabled={isPending}>
					{isPending ? <Loader2 className="animate-spin" /> : null}
					Créer le modèle
				</Button>
			</DialogFooter>
		</form>
	)
}
