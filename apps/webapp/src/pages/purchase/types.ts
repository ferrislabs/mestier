import { formatMoney } from '#/components/reference-table'
import { formatDate } from '#/pages/invoices/types'
import type {
	SupplierInvoice,
	SupplierInvoiceLine,
	SupplierInvoiceSource,
	SupplierInvoiceStatus,
} from '#/hooks/use-supplier-invoices'

export { formatMoney, formatDate }

export function supplierInvoiceStatusLabel(status: SupplierInvoiceStatus): string {
	if (status === 'RECEIVED') return 'Reçue'
	if (status === 'CONFIRMED') return 'Confirmée'
	if (status === 'REJECTED') return 'Rejetée'
	return status
}

export function supplierInvoiceSourceLabel(source: SupplierInvoiceSource): string {
	if (source === 'FACTUR_X') return 'Factur-X'
	return 'Saisie manuelle'
}

/** What is left to attribute on this line — a project cost silently 80 %
 * allocated is a cost that quietly went missing (#340's own warning), so
 * this is what a line editor keeps visible at all times. */
export function unallocatedCents(
	line: SupplierInvoiceLine,
	allocatedCents: number,
): number {
	return line.line_total_cents - allocatedCents
}

export function sumAllocatedCents(amounts: number[]): number {
	return amounts.reduce((sum, amount) => sum + amount, 0)
}
