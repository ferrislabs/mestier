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
	emptyAccountForm,
	validateAccountForm,
} from '#/pages/customers/lib/account-form'
import { AccountFields } from '#/pages/customers/ui/account-fields'
import { AddressFields } from '#/pages/customers/ui/address-fields'
import { ContactFields } from '#/pages/customers/ui/contact-fields'
import { FormErrors, FormSection } from '#/pages/customers/ui/form-section'
import { useFormBinding } from '#/pages/customers/ui/use-form-binding'

interface CustomerCreateDialogProps {
	open: boolean
	onOpenChange: (open: boolean) => void
	trigger?: ReactNode
	defaultTerms: number
	defaultOwnerId: string | null
	isPending: boolean
	error: string | null
	onSubmit: (form: AccountForm) => Promise<void>
}

export function CustomerCreateDialog({
	open,
	onOpenChange,
	trigger,
	defaultTerms,
	defaultOwnerId,
	isPending,
	error,
	onSubmit,
}: CustomerCreateDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			{trigger && <DialogTrigger asChild>{trigger}</DialogTrigger>}
			<DialogContent className="flex max-h-[calc(100dvh-2rem)] flex-col gap-0 p-0 sm:max-w-2xl">
				<CustomerCreateForm
					defaultTerms={defaultTerms}
					defaultOwnerId={defaultOwnerId}
					isPending={isPending}
					error={error}
					onSubmit={onSubmit}
				/>
			</DialogContent>
		</Dialog>
	)
}

function CustomerCreateForm({
	defaultTerms,
	defaultOwnerId,
	isPending,
	error,
	onSubmit,
}: Omit<CustomerCreateDialogProps, 'open' | 'onOpenChange' | 'trigger'>) {
	const [form, setForm] = useState<AccountForm>(() =>
		emptyAccountForm(defaultOwnerId),
	)
	const errors = validateAccountForm(form, 'create')
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
				<DialogTitle>Nouveau client</DialogTitle>
				<DialogDescription>
					Renseigne l’essentiel et un premier interlocuteur. Tu pourras
					compléter la fiche ensuite.
				</DialogDescription>
			</DialogHeader>
			<div className="grid min-h-0 flex-1 gap-6 overflow-y-auto border-t px-6 py-5">
				<FormSection label="Identité">
					<AccountFields
						form={form}
						onChange={change}
						bind={bind}
						mode="create"
						defaultTerms={defaultTerms}
					/>
				</FormSection>
				<FormSection
					label="Premier contact"
					hint="Il sera repris sur les devis et factures comme contact principal."
				>
					<ContactFields
						value={form.contact}
						onChange={(contact) => change({ contact })}
						bind={bind}
					/>
				</FormSection>
				<FormSection
					label="Première adresse de facturation"
					hint="Laisse vide pour la renseigner plus tard."
				>
					<AddressFields
						value={form.address}
						onChange={(address) => change({ address })}
						bind={bind}
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
						Créer le client
					</Button>
				</div>
			</div>
		</form>
	)
}
