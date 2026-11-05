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
	type ContactForm,
	validateContactForm,
} from '#/pages/customers/lib/account-form'
import { ContactFields } from '#/pages/customers/ui/contact-fields'
import { FormErrors } from '#/pages/customers/ui/form-section'
import { useFormBinding } from '#/pages/customers/ui/use-form-binding'

interface ContactFormDialogProps {
	open: boolean
	onOpenChange: (open: boolean) => void
	title: string
	initial: ContactForm
	trigger?: ReactNode
	isEditing: boolean
	isPending: boolean
	error: string | null
	onSubmit: (form: ContactForm) => Promise<void>
}

export function ContactFormDialog({
	open,
	onOpenChange,
	trigger,
	...form
}: ContactFormDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			{trigger && <DialogTrigger asChild>{trigger}</DialogTrigger>}
			<DialogContent className="flex max-h-[calc(100dvh-2rem)] flex-col gap-0 p-0 sm:max-w-lg">
				<ContactFormBody {...form} />
			</DialogContent>
		</Dialog>
	)
}

function ContactFormBody({
	title,
	initial,
	isEditing,
	isPending,
	error,
	onSubmit,
}: Omit<ContactFormDialogProps, 'open' | 'onOpenChange' | 'trigger'>) {
	const [draft, setDraft] = useState<ContactForm>(initial)
	const errors = validateContactForm(draft)
	const { bind, revealAll, formRef } = useFormBinding(errors)

	function submit(event: FormEvent) {
		event.preventDefault()
		revealAll()
		if (Object.keys(errors).length > 0) return
		void onSubmit(draft).catch(() => undefined)
	}

	return (
		<form
			ref={formRef}
			onSubmit={submit}
			noValidate
			className="flex min-h-0 flex-1 flex-col"
		>
			<DialogHeader className="px-6 pt-6 pb-4">
				<DialogTitle>{title}</DialogTitle>
				<DialogDescription>
					Un prénom et un nom sont requis. L’e-mail sert à retrouver le client
					quand un paiement arrive par Stripe.
				</DialogDescription>
			</DialogHeader>
			<div className="grid min-h-0 flex-1 gap-3 overflow-y-auto border-t px-6 py-5 sm:grid-cols-2">
				<ContactFields value={draft} onChange={setDraft} bind={bind} />
				<div className="sm:col-span-2">
					<FormErrors messages={error ? [error] : []} />
				</div>
			</div>
			<div className="flex justify-end gap-2 border-t px-6 py-3">
				<DialogClose asChild>
					<Button type="button" variant="outline">
						Annuler
					</Button>
				</DialogClose>
				<Button type="submit" disabled={isPending}>
					{isEditing ? 'Enregistrer' : 'Ajouter le contact'}
				</Button>
			</div>
		</form>
	)
}
