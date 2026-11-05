import { Loader2 } from 'lucide-react'
import type { FormEvent } from 'react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogClose,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import type { CustomerContext } from '#/hooks/use-customers'
import { hasErrors } from '#/pages/projects/lib/form-model'
import type { Person } from '#/pages/projects/lib/model'
import {
	type ProjectFullForm,
	projectFormErrors,
} from '#/pages/projects/lib/project-form'
import { useFormBinding } from '#/pages/projects/lib/use-form-binding'
import {
	ProjectFields,
	type ProjectOption,
} from '#/pages/projects/ui/project-fields'

export interface ProjectEditDialogProps {
	open: boolean
	form: ProjectFullForm
	customers: readonly ProjectOption[]
	quotes: readonly ProjectOption[]
	customerContexts: CustomerContext[]
	isCustomerContextsLoading: boolean
	people: readonly Person[]
	isPending: boolean
	error: string | null
	onOpenChange: (open: boolean) => void
	onChange: (patch: Partial<ProjectFullForm>) => void
	onSubmit: () => void
}

export function ProjectEditDialog({
	open,
	onOpenChange,
	...rest
}: ProjectEditDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="flex max-h-[calc(100dvh-2rem)] flex-col gap-0 p-0 sm:max-w-xl">
				<ProjectEditForm {...rest} />
			</DialogContent>
		</Dialog>
	)
}

function ProjectEditForm({
	form,
	customers,
	quotes,
	customerContexts,
	isCustomerContextsLoading,
	people,
	isPending,
	error,
	onChange,
	onSubmit,
}: Omit<ProjectEditDialogProps, 'open' | 'onOpenChange'>) {
	const errors = projectFormErrors(form)
	const { bind, baseId, revealAll, formRef } = useFormBinding(errors)

	function submit(event: FormEvent) {
		event.preventDefault()
		revealAll()
		if (hasErrors(errors) || isPending) return
		onSubmit()
	}

	return (
		<form
			ref={formRef}
			onSubmit={submit}
			noValidate
			className="flex min-h-0 flex-1 flex-col"
		>
			<DialogHeader className="px-6 pt-6 pb-4">
				<DialogTitle>Modifier le projet</DialogTitle>
				<DialogDescription>
					Corrige les informations du projet. Ses missions et son DOE ne
					changent pas.
				</DialogDescription>
			</DialogHeader>
			<div className="grid min-h-0 flex-1 gap-4 overflow-y-auto border-t px-6 py-5 sm:grid-cols-2">
				<ProjectFields
					form={form}
					onChange={onChange}
					bind={bind}
					baseId={baseId}
					customers={customers}
					quotes={quotes}
					customerContexts={customerContexts}
					isCustomerContextsLoading={isCustomerContextsLoading}
					people={people}
				/>
				{errors.other.length > 0 && (
					<ul className="grid gap-1 text-[13px] text-state-alert sm:col-span-2">
						{errors.other.map((message) => (
							<li key={message}>{message}</li>
						))}
					</ul>
				)}
				{error ? (
					<p
						role="alert"
						className="text-[13px] text-state-alert sm:col-span-2"
					>
						{error}
					</p>
				) : null}
			</div>
			<div className="flex items-center justify-between gap-3 border-t px-6 py-3">
				<p className="text-[12px] text-muted-foreground">
					Les champs marqués d’une * sont obligatoires.
				</p>
				<div className="flex gap-2">
					<DialogClose asChild>
						<Button type="button" variant="outline">
							Annuler
						</Button>
					</DialogClose>
					<Button type="submit" disabled={isPending}>
						{isPending ? <Loader2 className="animate-spin" /> : null}
						Enregistrer
					</Button>
				</div>
			</div>
		</form>
	)
}
