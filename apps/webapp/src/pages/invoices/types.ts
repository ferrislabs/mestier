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
}

export function emptyInvoiceLine(clientId = 'line-1'): InvoiceLineFormValues {
	return {
		clientId,
		label: '',
		quantity: '1',
		unitPrice: '',
		vatRateBp: '',
	}
}

export interface InvoiceFormValues {
	kind: InvoiceKind
	projectId: string
	customerId: string
	customerContextId: string
	dueAt: string
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

export function operationNatureLabel(nature: string): string {
	if (nature === 'GOODS') return 'Livraison de biens'
	if (nature === 'SERVICES') return 'Prestation de services'
	if (nature === 'BOTH') return 'Biens et services'
	return nature
}

/**
 * "An overdue invoice reads as overdue in the list, not only on its own
 * page" (#321) — a pure date comparison over fields the backend already
 * computed (`status`, `due_at`), not a recomputation of any amount, so this
 * is safe to run client-side.
 */
export function isInvoiceOverdue(
	invoice: Pick<Invoice, 'status' | 'due_at'>,
	now: Date = new Date(),
): boolean {
	return (
		invoice.status === 'ISSUED' &&
		invoice.due_at != null &&
		new Date(invoice.due_at) < now
	)
}

/**
 * A hand-typed decimal, split into digits and scale: `"12,50"` becomes
 * `{ digits: 1250n, scale: 2 }`. `null` when there is nothing usable to read.
 *
 * Copied from `pages/quotes/types.ts` rather than imported: the backend rule
 * it mirrors (`rust_decimal`'s banker's rounding) is shared, but the two
 * pages' form-value types are not, and `parseDecimal`/`scaleTo` are not
 * exported from that module.
 */
function parseDecimal(value: string): { digits: bigint; scale: number } | null {
	const normalized = value.replace(',', '.').trim()
	if (!/^\d*\.?\d*$/.test(normalized) || normalized === '.' || !normalized) {
		return null
	}

	const [whole = '', fraction = ''] = normalized.split('.')
	const digits = `${whole}${fraction}`
	if (!digits) return null

	return { digits: BigInt(digits), scale: fraction.length }
}

/**
 * Restates `digits × 10^-scale` at `targetScale`, rounding halves to even —
 * see `pages/quotes/types.ts`'s `scaleTo` for the full rationale.
 */
function scaleTo(digits: bigint, scale: number, targetScale: number): bigint {
	if (scale <= targetScale) {
		return digits * 10n ** BigInt(targetScale - scale)
	}

	const divisor = 10n ** BigInt(scale - targetScale)
	const quotient = digits / divisor
	const doubled = (digits % divisor) * 2n

	if (doubled > divisor) return quotient + 1n
	if (doubled < divisor) return quotient
	// Exactly a half: land on the even neighbour.
	return quotient % 2n === 0n ? quotient : quotient + 1n
}

export function centsToEuros(cents: number): string {
	return (cents / 100).toFixed(2)
}

export function eurosToCents(value: string): number {
	const parsed = parseDecimal(value)
	if (!parsed) return 0

	return Number(scaleTo(parsed.digits, parsed.scale, 2))
}

/**
 * The line's total, to the cent, computed the way the backend computes it —
 * see `pages/quotes/types.ts`'s `quoteLineTotalCents` for the full
 * rationale; this is the same arithmetic over the invoice line's own
 * (smaller) set of fields.
 */
export function invoiceLineTotalCents(line: InvoiceLineFormValues): number {
	const quantity = parseDecimal(line.quantity)
	if (!quantity || quantity.digits === 0n) return 0

	const unitPriceCents = BigInt(eurosToCents(line.unitPrice))

	return Number(scaleTo(quantity.digits * unitPriceCents, quantity.scale, 0))
}

export function invoiceLineSummary(line: InvoiceLineFormValues): string {
	const quantity = line.quantity.trim() || '0'
	return `${quantity} × ${formatMoney(eurosToCents(line.unitPrice))}`
}

export type { InvoiceKind, InvoiceStatus, OperationNature }
