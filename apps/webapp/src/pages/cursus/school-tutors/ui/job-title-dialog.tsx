import { useForm } from '@tanstack/react-form'
import type { FormEvent } from 'react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import {
	FieldInput,
	FormError,
} from '#/pages/cursus/participants/ui/field-input'

export interface JobTitleDialogProps {
	open: boolean
	tutorName: string
	initial: string
	isPending: boolean
	error: string | null
	onOpenChange: (open: boolean) => void
	onSubmit: (jobTitle: string) => void
}

export function JobTitleDialog({
	open,
	onOpenChange,
	...body
}: JobTitleDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="sm:max-w-sm">
				<JobTitleForm {...body} onCancel={() => onOpenChange(false)} />
			</DialogContent>
		</Dialog>
	)
}

interface JobTitleFormProps
	extends Omit<JobTitleDialogProps, 'open' | 'onOpenChange'> {
	onCancel: () => void
}

function JobTitleForm({
	tutorName,
	initial,
	isPending,
	error,
	onCancel,
	onSubmit,
}: JobTitleFormProps) {
	const form = useForm({
		defaultValues: { jobTitle: initial },
		onSubmit: ({ value }) => onSubmit(value.jobTitle),
	})

	function submit(event: FormEvent) {
		event.preventDefault()
		if (!isPending) void form.handleSubmit()
	}

	return (
		<form className="grid gap-4" onSubmit={submit}>
			<DialogHeader>
				<DialogTitle>Modifier la fonction</DialogTitle>
				<DialogDescription>{tutorName}</DialogDescription>
			</DialogHeader>
			<form.Field name="jobTitle">
				{(field) => (
					<FieldInput
						id="school-tutor-edit-job-title"
						label="Fonction"
						value={field.state.value}
						onChange={field.handleChange}
					/>
				)}
			</form.Field>
			<FormError message={error} />
			<DialogFooter>
				<Button type="button" variant="ghost" onClick={onCancel}>
					Annuler
				</Button>
				<Button type="submit" disabled={isPending}>
					Enregistrer
				</Button>
			</DialogFooter>
		</form>
	)
}
