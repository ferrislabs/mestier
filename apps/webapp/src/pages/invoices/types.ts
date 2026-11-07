import { formatMoney } from '#/components/reference-table'
import type { Customer, CustomerContext } from '#/hooks/use-customers'
import type {
	Invoice,
	InvoiceKind,
	InvoiceStatus,
	OperationNature,
} from '#/hooks/use-invoices'

export { formatMoney } from '#/components/reference-table'

export interface InvoiceLineFormValues {
	clientId: string
	label: string
	quantity: string
	unitPrice: string
	/** Basis points as a string for the input, `''` when unset — only ever
	 * read by the server when the organization is subject to VAT, same rule
	 * as `QuoteLineFormValues.vatRateBp`. */
	vatRateBp: string
	discountPercent: string
	productId: string | null
	serviceRateId: string | null
}

export function emptyInvoiceLine(clientId = 'line-1'): InvoiceLineFormValues {
	return {
		clientId,
		label: '',
		quantity: '1',
		unitPrice: '',
		vatRateBp: '',
		discountPercent: '0',
		productId: null,
		serviceRateId: null,
	}
}

export interface InvoiceFormValues {
	kind: InvoiceKind
	projectId: string
	customerId: string
	customerContextId: string
	title: string
	termsDays: string
	legalNotice: string
	notes: string
	operationNature: OperationNature | ''
	lines: InvoiceLineFormValues[]
}

/** The rates offered as one-click choices before an artisan types a custom
 * one — mirrors `pages/quotes/types.ts`'s `COMMON_VAT_RATES_BP`. */
export const COMMON_VAT_RATES_BP = [2000, 1000, 550, 210, 0] as const

export function formatVatRateBp(rateBp: number): string {
	const value = rateBp / 100
	return `${Number.isInteger(value) ? value : value.toFixed(2)} %`
}

export function customerDisplayName(customer: Customer): string {
	return customer.name.trim()
}

export function customerContextDisplayName(
	customerContext: CustomerContext,
): string {
	const detail = [customerContext.postal_code, customerContext.city]
		.filter(Boolean)
		.join(' ')
	return detail ? `${customerContext.label} · ${detail}` : customerContext.label
}

export function formatDate(value: string): string {
	return new Intl.DateTimeFormat('fr-FR', {
		day: '2-digit',
		month: 'short',
		year: 'numeric',
	}).format(new Date(value))
}

export function invoiceStatusLabel(status: string): string {
	if (status === 'DRAFT') return 'Brouillon'
	if (status === 'ISSUED') return 'Émise'
	if (status === 'PAID') return 'Payée'
	if (status === 'PARTIALLY_PAID') return 'Partiellement payée'
	if (status === 'CANCELLED') return 'Annulée'
	return status
}

export function invoiceKindLabel(kind: string): string {
	if (kind === 'STANDARD') return 'Facture'
	if (kind === 'DEPOSIT') return 'Acompte'
	if (kind === 'FINAL') return 'Solde'
	if (kind === 'CREDIT_NOTE') return 'Avoir'
	return kind
}

export function quoteStatusText(status: string): string {
	if (status === 'DRAFT') return 'Brouillon'
	if (status === 'SENT') return 'Envoyé'
	if (status === 'ACCEPTED') return 'Accepté'
	if (status === 'DECLINED') return 'Refusé'
	if (status === 'CANCELLED') return 'Annulé'
	return status
}

export function operationNatureLabel(nature: string): string {
	if (nature === 'GOODS') return 'Livraison de biens'
	if (nature === 'SERVICES') return 'Prestation de services'
	if (nature === 'BOTH') return 'Biens et services'
	return nature
}

export type InvoiceDisplayState =
	| 'draft'
	| 'unpaid'
	| 'partial'
	| 'overdue'
	| 'paid'
	| 'cancelled'
	| 'refund_due'

export const INVOICE_STATE_LABELS: Record<InvoiceDisplayState, string> = {
	draft: 'Brouillon',
	unpaid: 'À payer',
	partial: 'Partiellement payée',
	overdue: 'En retard',
	paid: 'Payée',
	cancelled: 'Annulée',
	refund_due: 'À rembourser',
}

const DISPLAY_STATES: Record<
	NonNullable<Invoice['display_state']>,
	InvoiceDisplayState
> = {
	DRAFT: 'draft',
	UNPAID: 'unpaid',
	PARTIAL: 'partial',
	OVERDUE: 'overdue',
	PAID: 'paid',
	CANCELLED: 'cancelled',
	REFUND_DUE: 'refund_due',
}

/**
 * The state an invoice reads as in the list and on its page: the backend
 * decides it (draft, refund due, cancelled, paid, overdue, partial, unpaid),
 * this only maps the wire value to the label key.
 */
export function invoiceDisplayState(
	invoice: Pick<Invoice, 'display_state'>,
): InvoiceDisplayState {
	return DISPLAY_STATES[invoice.display_state]
}

function parseDecimal(value: string): { digits: bigint; scale: number } | null {
	const normalized = value.replace(',', '.').trim()
	if (!/^-?\d*\.?\d*$/.test(normalized) || normalized === '.' || !normalized) {
		return null
	}

	const negative = normalized.startsWith('-')
	const [whole = '', fraction = ''] = normalized.replace('-', '').split('.')
	const digits = `${whole}${fraction}`
	if (!digits) return null

	return {
		digits: negative ? -BigInt(digits) : BigInt(digits),
		scale: fraction.length,
	}
}

function scaleTo(digits: bigint, scale: number, targetScale: number): bigint {
	if (scale <= targetScale) {
		return digits * 10n ** BigInt(targetScale - scale)
	}

	const divisor = 10n ** BigInt(scale - targetScale)
	const magnitude = digits < 0n ? -digits : digits
	const quotient = magnitude / divisor
	const rounded =
		(magnitude % divisor) * 2n >= divisor ? quotient + 1n : quotient
	return digits < 0n ? -rounded : rounded
}

export function centsToEuros(cents: number): string {
	return (cents / 100).toFixed(2)
}

export function eurosToCents(value: string): number {
	const parsed = parseDecimal(value)
	if (!parsed) return 0

	return Number(scaleTo(parsed.digits, parsed.scale, 2))
}

export function invoiceLineTotalCents(line: InvoiceLineFormValues): number {
	const quantity = parseDecimal(line.quantity)
	if (!quantity || quantity.digits === 0n) return 0

	const unitPriceCents = BigInt(eurosToCents(line.unitPrice))
	const discount = parseDecimal(line.discountPercent) ?? {
		digits: 0n,
		scale: 0,
	}
	const remaining = 100n * 10n ** BigInt(discount.scale) - discount.digits
	if (remaining < 0n) return 0

	return Number(
		scaleTo(
			quantity.digits * unitPriceCents * remaining,
			quantity.scale + discount.scale + 2,
			0,
		),
	)
}

export function invoiceLineSummary(line: InvoiceLineFormValues): string {
	const quantity = line.quantity.trim() || '0'
	return `${quantity} × ${formatMoney(eurosToCents(line.unitPrice))}`
}

export type { InvoiceKind, InvoiceStatus, OperationNature }
