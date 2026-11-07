import type {
	InvoicePayment,
	PaymentMethod,
	RecordInvoicePaymentPayload,
} from '#/hooks/use-invoices'
import { centsToEuros, eurosToCents, formatMoney } from '#/pages/invoices/types'

export const PAYMENT_METHODS: readonly PaymentMethod[] = [
	'TRANSFER',
	'CHECK',
	'CASH',
	'CARD',
	'DIRECT_DEBIT',
]

export const PAYMENT_METHOD_LABELS: Record<PaymentMethod, string> = {
	TRANSFER: 'Virement',
	CHECK: 'Chèque',
	CASH: 'Espèces',
	CARD: 'Carte bancaire',
	DIRECT_DEBIT: 'Prélèvement',
}

export type PaymentMode = 'payment' | 'refund'

export interface PaymentValues {
	date: string
	amount: string
	method: PaymentMethod
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
	return errors
}

export function paymentRequest(
	values: PaymentValues,
	mode: PaymentMode = 'payment',
): RecordInvoicePaymentPayload {
	const cents = eurosToCents(values.amount)

	return {
		amount_cents: mode === 'refund' ? -cents : cents,
		paid_on: values.date,
		method: values.method,
		reference: values.reference.trim() || null,
		note: values.note.trim() || null,
	}
}

export interface RefundRecord {
	id: string
	date: string
	amountCents: number
	method: PaymentMethod
	reference: string | null
	note: string | null
}

export function paymentsAndRefunds(payments: readonly InvoicePayment[]): {
	payments: InvoicePayment[]
	refunds: RefundRecord[]
} {
	return {
		payments: payments.filter((payment) => payment.amount_cents > 0),
		refunds: payments
			.filter((payment) => payment.amount_cents < 0)
			.map((payment) => ({
				id: payment.id,
				date: payment.paid_on,
				amountCents: -payment.amount_cents,
				method: payment.method,
				reference: payment.reference ?? null,
				note: payment.note ?? null,
			})),
	}
}
