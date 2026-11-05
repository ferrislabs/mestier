import type { RecordInvoicePaymentPayload } from '#/hooks/use-invoices'
import { centsToEuros, eurosToCents, formatMoney } from '#/pages/invoices/types'

export const PAYMENT_METHODS = [
	'Virement',
	'Chèque',
	'Espèces',
	'Carte bancaire',
	'Prélèvement',
	'Autre',
] as const

export interface PaymentValues {
	date: string
	amount: string
	method: string
	reference: string
	note: string
}

export function todayDate(now: Date = new Date()): string {
	const month = String(now.getMonth() + 1).padStart(2, '0')
	const day = String(now.getDate()).padStart(2, '0')
	return `${now.getFullYear()}-${month}-${day}`
}

export function initialPaymentValues(
	remainingCents: number | null,
	today: string,
): PaymentValues {
	return {
		date: today,
		amount:
			remainingCents !== null && remainingCents > 0
				? centsToEuros(remainingCents).replace('.', ',')
				: '',
		method: PAYMENT_METHODS[0],
		reference: '',
		note: '',
	}
}

export function paymentErrors(
	values: PaymentValues,
	remainingCents: number | null,
	today: string,
): string[] {
	const errors: string[] = []
	if (values.date === '') errors.push('La date du paiement est obligatoire.')
	else if (values.date > today)
		errors.push('La date du paiement ne peut pas être dans le futur.')
	const cents = eurosToCents(values.amount)
	if (!(cents > 0)) errors.push('Saisis un montant supérieur à zéro.')
	else if (remainingCents !== null && cents > remainingCents)
		errors.push(
			`Le montant dépasse le reste dû (${formatMoney(remainingCents)}).`,
		)
	if (values.method.trim() === '') errors.push('Choisis un moyen de paiement.')
	return errors
}

export function paymentRequest(
	values: PaymentValues,
): RecordInvoicePaymentPayload {
	return {
		amount_cents: eurosToCents(values.amount),
		paid_on: values.date,
		method: values.method.trim(),
		reference: values.reference.trim() || null,
		note: values.note.trim() || null,
	}
}
