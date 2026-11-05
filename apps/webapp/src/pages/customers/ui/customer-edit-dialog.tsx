import { type FormEvent, type ReactNode, useState } from 'react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogClose,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
	DialogTrigger,
} from '#/components/ui/dialog'
import {
	type AccountForm,
	validateAccountForm,
} from '#/pages/customers/lib/account-form'
import { AccountFields } from '#/pages/customers/ui/account-fields'
import { FormErrors, FormSection } from '#/pages/customers/ui/form-section'
import { useFormBinding } from '#/pages/customers/ui/use-form-binding'

interface CustomerEditDialogProps {
	open: boolean
	onOpenChange: (open: boolean) => void
	name: string
	initial: AccountForm
	trigger?: ReactNode
	defaultTerms: number
	isPending: boolean
	error: string | null
	onSubmit: (form: AccountForm) => Promise<void>
}

export function CustomerEditDialog({
	open,
	onOpenChange,
	trigger,
	...form
}: CustomerEditDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			{trigger && <DialogTrigger asChild>{trigger}</DialogTrigger>}
			<DialogContent className="flex max-h-[calc(100dvh-2rem)] flex-col gap-0 p-0 sm:max-w-2xl">
				<CustomerEditForm {...form} />
			</DialogContent>
		</Dialog>
	)
}

function CustomerEditForm({
	name,
	initial,
	defaultTerms,
	isPending,
	error,
	onSubmit,
}: Omit<CustomerEditDialogProps, 'open' | 'onOpenChange' | 'trigger'>) {
	const [form, setForm] = useState<AccountForm>(initial)
	const errors = validateAccountForm(form, 'edit')
	const { bind, revealAll, formRef } = useFormBinding(errors)

	function change(patch: Partial<AccountForm>) {
		setForm((current) => ({ ...current, ...patch }))
	}

	function submit(event: FormEvent) {
		event.preventDefault()
		revealAll()
		if (Object.keys(errors).length > 0) return
		void onSubmit(form).catch(() => undefined)
	}

	return (
		<form
			ref={formRef}
			onSubmit={submit}
			noValidate
			className="flex min-h-0 flex-1 flex-col"
		>
			<DialogHeader className="px-6 pt-6 pb-4">
				<DialogTitle>Modifier {name}</DialogTitle>
				<DialogDescription>
					Mets à jour l’identité et le suivi commercial. Les contacts et les
					adresses se gèrent dans leurs rubriques.
				</DialogDescription>
			</DialogHeader>
			<div className="grid min-h-0 flex-1 gap-6 overflow-y-auto border-t px-6 py-5">
				<FormSection label="Identité et suivi">
					<AccountFields
						form={form}
						onChange={change}
						bind={bind}
						mode="edit"
						defaultTerms={defaultTerms}
					/>
				</FormSection>
				<FormErrors messages={error ? [error] : []} />
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
						Enregistrer
					</Button>
				</div>
			</div>
		</form>
	)
}
