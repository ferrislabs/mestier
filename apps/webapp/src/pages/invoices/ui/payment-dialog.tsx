import { Loader2 } from 'lucide-react'
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
import type { PaymentMethod } from '#/hooks/use-invoices'
import {
	initialPaymentValues,
	PAYMENT_METHOD_LABELS,
	PAYMENT_METHODS,
	type PaymentMode,
	type PaymentValues,
	paymentErrors,
	todayDate,
} from '#/pages/invoices/lib/payment-form'
import { AutoTextarea } from '#/pages/quotes/ui/doc/primitives'

interface PaymentDialogProps {
	open: boolean
	onOpenChange: (open: boolean) => void
	mode: PaymentMode
	remainingCents: number | null
	isPending: boolean
	error: string | null
	onSubmit: (values: PaymentValues) => Promise<boolean>
}

const COPY: Record<
	PaymentMode,
	{
		title: string
		description: string
		amount: string
		submit: string
	}
> = {
	payment: {
		title: 'Enregistrer un paiement',
		description:
			'Note un paiement reçu pour cette facture. Le reste dû se met à jour aussitôt.',
		amount: 'Montant reçu (€)',
		submit: 'Enregistrer le paiement',
	},
	refund: {
		title: 'Rembourser le client',
		description:
			'Le client a payé plus que le total de la facture : note le remboursement effectué pour solder le trop-perçu.',
		amount: 'Montant remboursé (€)',
		submit: 'Enregistrer le remboursement',
	},
}

function PaymentForm({
	mode,
	remainingCents,
	isPending,
	error,
	onClose,
	onSubmit,
}: Omit<PaymentDialogProps, 'open' | 'onOpenChange'> & {
	onClose: () => void
}) {
	const baseId = useId()
	const today = todayDate()
	const [values, setValues] = useState<PaymentValues>(() =>
		initialPaymentValues(remainingCents, today, mode),
	)
	const [attempted, setAttempted] = useState(false)
	const errors = paymentErrors(values, remainingCents, today, mode)
	const id = (field: string) => `${baseId}-${field}`
	const copy = COPY[mode]

	const change = (patch: Partial<PaymentValues>) =>
		setValues((current) => ({ ...current, ...patch }))

	const submit = async (event: FormEvent) => {
		event.preventDefault()
		if (errors.length > 0) {
			setAttempted(true)
			return
		}
		if (await onSubmit(values)) onClose()
	}

	return (
		<form onSubmit={submit} noValidate className="grid gap-4">
			<DialogHeader>
				<DialogTitle>{copy.title}</DialogTitle>
				<DialogDescription>{copy.description}</DialogDescription>
			</DialogHeader>
			<div className="grid gap-4 sm:grid-cols-2">
				<FormField id={id('date')} label="Date" required>
					<Input
						id={id('date')}
						type="date"
						max={today}
						value={values.date}
						onChange={(event) => change({ date: event.target.value })}
					/>
				</FormField>
				<FormField id={id('amount')} label={copy.amount} required>
					<Input
						id={id('amount')}
						inputMode="decimal"
						autoComplete="off"
						value={values.amount}
						onChange={(event) => change({ amount: event.target.value })}
						className="text-right tabular-nums"
					/>
				</FormField>
				<FormField id={id('method')} label="Moyen de paiement" required>
					<Select
						value={values.method}
						onValueChange={(method) =>
							change({ method: method as PaymentMethod })
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
				<FormField id={id('reference')} label="Référence">
					<Input
						id={id('reference')}
						autoComplete="off"
						placeholder="VIR 1105 DUPONT"
						value={values.reference}
						onChange={(event) => change({ reference: event.target.value })}
					/>
				</FormField>
				<FormField id={id('note')} label="Note" className="sm:col-span-2">
					<AutoTextarea
						id={id('note')}
						value={values.note}
						onChange={(event) => change({ note: event.target.value })}
						className="min-h-16"
					/>
				</FormField>
			</div>
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
			{error && (
				<p role="alert" className="text-sm text-state-alert">
					{error}
				</p>
			)}
			<DialogFooter>
				<Button type="button" variant="outline" onClick={onClose}>
					Annuler
				</Button>
				<Button type="submit" disabled={isPending}>
					{isPending ? <Loader2 className="animate-spin" /> : null}
					{copy.submit}
				</Button>
			</DialogFooter>
		</form>
	)
}

export function PaymentDialog({
	open,
	onOpenChange,
	...rest
}: PaymentDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="max-h-[calc(100dvh-2rem)] overflow-y-auto sm:max-w-xl">
				<PaymentForm {...rest} onClose={() => onOpenChange(false)} />
			</DialogContent>
		</Dialog>
	)
}
