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

export type PaymentMode = 'payment' | 'refund'

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

function owedCents(
	remainingCents: number | null,
	mode: PaymentMode,
): number | null {
	if (remainingCents === null) return null
	return mode === 'payment' ? remainingCents : Math.max(0, -remainingCents)
}

export function initialPaymentValues(
	remainingCents: number | null,
	today: string,
	mode: PaymentMode = 'payment',
): PaymentValues {
	const owed = owedCents(remainingCents, mode)
	return {
		date: today,
		amount:
			owed !== null && owed > 0 ? centsToEuros(owed).replace('.', ',') : '',
		method: PAYMENT_METHODS[0],
		reference: '',
		note: '',
	}
}

export function paymentErrors(
	values: PaymentValues,
	remainingCents: number | null,
	today: string,
	mode: PaymentMode = 'payment',
): string[] {
	const errors: string[] = []
	if (values.date === '') errors.push('La date du paiement est obligatoire.')
	else if (values.date > today)
		errors.push('La date du paiement ne peut pas être dans le futur.')
	const cents = eurosToCents(values.amount)
	if (!(cents > 0)) errors.push('Saisis un montant supérieur à zéro.')
	else if (mode === 'refund') {
		const owed = owedCents(remainingCents, mode)
		if (owed !== null && cents > owed)
			errors.push(`Le montant dépasse le trop-perçu (${formatMoney(owed)}).`)
	} else if (remainingCents !== null && cents > remainingCents)
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

export interface RefundInput {
	date: string
	amountCents: number
	method: string
	reference: string
	note: string
}

export function refundRecord(values: PaymentValues): RefundInput {
	return {
		date: values.date,
		amountCents: eurosToCents(values.amount),
		method: values.method.trim(),
		reference: values.reference.trim(),
		note: values.note.trim(),
	}
}

export function effectiveRemainingCents(
	remainingCents: number | null,
	refundedCents: number,
): number | null {
	if (remainingCents === null || remainingCents >= 0) return remainingCents
	return Math.min(0, remainingCents + refundedCents)
}
