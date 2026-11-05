import type { InvoiceLine, InvoiceLinePayload } from '#/hooks/use-invoices'
import { centsToEuros, invoiceLineTotalCents } from '#/pages/invoices/types'

export type CreditMode = 'total' | 'partial'

export type QuantityInputs = Record<string, string>

function parseQuantity(text: string | undefined): number | null {
	const cleaned = (text ?? '').replace(/\s/g, '').replace(',', '.')
	if (cleaned === '') return null
	if (!/^\d+(\.\d+)?$/.test(cleaned)) return Number.NaN
	return Number(cleaned)
}

export function creditedQuantity(
	line: InvoiceLine,
	mode: CreditMode,
	inputs: QuantityInputs,
): number {
	if (mode === 'total') return Number(line.quantity.replace(',', '.'))
	const parsed = parseQuantity(inputs[line.id])
	return parsed === null || Number.isNaN(parsed) ? 0 : parsed
}

export function creditLineIssue(
	line: InvoiceLine,
	mode: CreditMode,
	inputs: QuantityInputs,
): string | null {
	if (mode === 'total') return null
	const parsed = parseQuantity(inputs[line.id])
	if (parsed === null) return null
	if (Number.isNaN(parsed)) return 'Quantité illisible.'
	if (parsed > Number(line.quantity.replace(',', '.')))
		return 'La quantité dépasse celle de la facture.'
	return null
}

export function fillAll(lines: readonly InvoiceLine[]): QuantityInputs {
	return Object.fromEntries(lines.map((line) => [line.id, line.quantity]))
}

export function selectedCount(
	lines: readonly InvoiceLine[],
	mode: CreditMode,
	inputs: QuantityInputs,
): number {
	return lines.filter((line) => creditedQuantity(line, mode, inputs) > 0).length
}

export function creditPreviewNetCents(
	lines: readonly InvoiceLine[],
	mode: CreditMode,
	inputs: QuantityInputs,
): number {
	return lines.reduce((sum, line) => {
		const quantity = creditedQuantity(line, mode, inputs)
		if (!(quantity > 0)) return sum
		return (
			sum +
			invoiceLineTotalCents({
				clientId: line.id,
				label: line.label,
				quantity: String(quantity),
				unitPrice: centsToEuros(line.unit_price_cents),
				vatRateBp: '',
			})
		)
	}, 0)
}

export function creditNoteErrors(
	lines: readonly InvoiceLine[],
	mode: CreditMode,
	inputs: QuantityInputs,
	reason: string,
): string[] {
	const errors: string[] = []
	if (reason.trim() === '') errors.push('Le motif de l’avoir est obligatoire.')
	if (lines.some((line) => creditLineIssue(line, mode, inputs) !== null))
		errors.push('Corrige les quantités signalées.')
	else if (selectedCount(lines, mode, inputs) === 0)
		errors.push('Indique une quantité à créditer sur au moins une ligne.')
	return errors
}

export function creditNoteLines(
	lines: readonly InvoiceLine[],
	mode: CreditMode,
	inputs: QuantityInputs,
): InvoiceLinePayload[] {
	return lines.flatMap((line) => {
		const quantity = creditedQuantity(line, mode, inputs)
		if (!(quantity > 0)) return []
		return [
			{
				label: line.label,
				quantity: String(quantity),
				unit_price_cents: line.unit_price_cents,
				vat_rate_basis_points: line.vat_rate_basis_points ?? null,
			},
		]
	})
}
