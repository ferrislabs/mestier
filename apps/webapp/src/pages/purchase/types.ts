import { formatMoney } from '#/components/reference-table'
import type {
	SupplierInvoiceSource,
	SupplierInvoiceStatus,
} from '#/hooks/use-supplier-invoices'
import { formatDate } from '#/pages/invoices/types'

export { formatMoney, formatDate }

export function supplierInvoiceStatusLabel(
	status: SupplierInvoiceStatus,
): string {
	if (status === 'RECEIVED') return 'Reçue'
	if (status === 'CONFIRMED') return 'Confirmée'
	if (status === 'REJECTED') return 'Rejetée'
	return status
}

export function supplierInvoiceSourceLabel(
	source: SupplierInvoiceSource,
): string {
	if (source === 'FACTUR_X') return 'Factur-X'
	return 'Saisie manuelle'
}
