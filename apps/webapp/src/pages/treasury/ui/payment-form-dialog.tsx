import { type FormEvent, useId, useState } from 'react'
import { FormField } from '#/components/form-field'
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
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import type { Customer } from '#/hooks/use-customers'
import { centsToInput } from '#/pages/customers/lib/format'
import { CustomerPicker } from '#/pages/customers/ui/customer-picker'
import { TextArea } from '#/pages/customers/ui/text-area'
import {
	type PaymentFormValues,
	paymentFormErrors,
} from '#/pages/treasury/lib/ledger-model'
import {
	type OpenInvoice,
	PAYMENT_METHOD_LABELS,
	PAYMENT_METHODS,
	type PaymentMethod,
} from '#/pages/treasury/types'
import { OpenInvoiceSelect } from '#/pages/treasury/ui/open-invoice-select'

interface PaymentFormDialogProps {
	open: boolean
	onOpenChange: (open: boolean) => void
	initial: PaymentFormValues
	today: string
	customers: readonly Customer[]
	openInvoices: readonly OpenInvoice[]
	onSubmit: (values: PaymentFormValues) => Promise<void>
}

export function PaymentFormDialog({
	open,
	onOpenChange,
	...form
}: PaymentFormDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="flex max-h-[calc(100dvh-2rem)] flex-col gap-0 p-0 sm:max-w-xl">
				<PaymentForm onClose={() => onOpenChange(false)} {...form} />
			</DialogContent>
		</Dialog>
	)
}

function PaymentForm({
	initial,
	today,
	customers,
	openInvoices,
	onClose,
	onSubmit,
}: Omit<PaymentFormDialogProps, 'open' | 'onOpenChange'> & {
	onClose: () => void
}) {
	const baseId = useId()
	const [values, setValues] = useState<PaymentFormValues>(initial)
	const [attempted, setAttempted] = useState(false)
	const id = (field: string) => `${baseId}-${field}`
	const errors = paymentFormErrors(
		values,
		today,
		(invoiceId) =>
			openInvoices.find((invoice) => invoice.id === invoiceId)?.balance ?? null,
	)
	const options = openInvoices

	function change(patch: Partial<PaymentFormValues>) {
		setValues((current) => ({ ...current, ...patch }))
	}

	function submit(event: FormEvent) {
		event.preventDefault()
		if (errors.length > 0) {
			setAttempted(true)
			return
		}
		void onSubmit(values).then(onClose)
	}

	return (
		<form onSubmit={submit} noValidate className="flex min-h-0 flex-1 flex-col">
			<DialogHeader className="px-6 pt-6 pb-4">
				<DialogTitle>Enregistrer une recette</DialogTitle>
				<DialogDescription>
					Note un paiement reçu. Imputé à une facture, il met à jour son reste à
					payer ; sinon il reste à affecter.
				</DialogDescription>
			</DialogHeader>
			<div className="grid min-h-0 flex-1 gap-4 overflow-y-auto border-t px-6 py-5 sm:grid-cols-2">
				<FormField id={id('date')} label="Date" required>
					<Input
						id={id('date')}
						type="date"
						max={today}
						value={values.date}
						onChange={(event) => change({ date: event.target.value })}
					/>
				</FormField>
				<FormField id={id('method')} label="Moyen de paiement" required>
					<Select
						value={values.method}
						onValueChange={(value) =>
							change({ method: value as PaymentMethod })
						}
					>
						<SelectTrigger id={id('method')} className="w-full">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							{PAYMENT_METHODS.map((method) => (
								<SelectItem key={method} value={method}>
									{PAYMENT_METHOD_LABELS[method]}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
				</FormField>
				<FormField
					id={id('account')}
					label="Client"
					className="sm:col-span-2"
					hint="Facultatif si tu imputes le paiement à une facture."
				>
					<div className="flex items-center gap-2">
						<CustomerPicker
							id={id('account')}
							customers={customers}
							value={values.customerId}
							onChange={(customerId) => change({ customerId })}
							placeholder="Aucun client"
						/>
						{values.customerId !== null && (
							<Button
								type="button"
								variant="ghost"
								size="sm"
								onClick={() => change({ customerId: null })}
							>
								Retirer
							</Button>
						)}
					</div>
				</FormField>
				<FormField
					id={id('invoice')}
					label="Facture à imputer"
					className="sm:col-span-2"
					hint={
						options.length === 0
							? 'Aucune facture émise n’a de solde à payer.'
							: 'Le montant proposé est le reste à payer de la facture.'
					}
				>
					<OpenInvoiceSelect
						id={id('invoice')}
						options={options}
						value={values.invoiceId}
						onChange={(invoiceId) => {
							const chosen = options.find((option) => option.id === invoiceId)
							change({
								invoiceId,
								amount: chosen ? centsToInput(chosen.balance) : values.amount,
							})
						}}
						noneLabel="Aucune facture (recette non affectée)"
						disabled={options.length === 0 && values.invoiceId === null}
					/>
				</FormField>
				<FormField
					id={id('amount')}
					label="Montant (€)"
					required
					hint="Pour un remboursement, saisis un montant négatif."
				>
					<Input
						id={id('amount')}
						inputMode="decimal"
						value={values.amount}
						onChange={(event) => change({ amount: event.target.value })}
						autoComplete="off"
						placeholder="0,00"
					/>
				</FormField>
				<FormField id={id('reference')} label="Référence">
					<Input
						id={id('reference')}
						value={values.reference}
						onChange={(event) => change({ reference: event.target.value })}
						autoComplete="off"
						placeholder="VIR 1105 DUPONT"
					/>
				</FormField>
				<FormField id={id('note')} label="Note" className="sm:col-span-2">
					<TextArea
						id={id('note')}
						rows={2}
						value={values.note}
						onChange={(event) => change({ note: event.target.value })}
					/>
				</FormField>
			</div>
			<div className="grid gap-3 border-t px-6 py-4">
				{attempted && errors.length > 0 && (
					<ul
						role="alert"
						className="list-disc rounded-md bg-state-alert/8 py-2 pr-3 pl-7 text-[13px] text-state-alert"
					>
						{errors.map((message) => (
							<li key={message}>{message}</li>
						))}
					</ul>
				)}
				<DialogFooter>
					<Button type="button" variant="outline" onClick={onClose}>
						Annuler
					</Button>
					<Button type="submit">Enregistrer la recette</Button>
				</DialogFooter>
			</div>
		</form>
	)
}
