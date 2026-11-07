import type { InvoiceLine } from '#/hooks/use-invoices'
import {
	centsToEuros,
	formatMoney,
	invoiceLineTotalCents,
} from '#/pages/invoices/types'

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
				discountPercent: line.discount_percent,
				productId: null,
				serviceRateId: null,
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

export interface CreditNoteRequest {
	full: boolean
	lines: { source_line_id: string; quantity: string }[]
}

export function creditNoteRequest(
	lines: readonly InvoiceLine[],
	mode: CreditMode,
	inputs: QuantityInputs,
): CreditNoteRequest {
	if (mode === 'total') return { full: true, lines: [] }

	return {
		full: false,
		lines: lines.flatMap((line) => {
			const quantity = creditedQuantity(line, mode, inputs)
			return quantity > 0
				? [{ source_line_id: line.id, quantity: String(quantity) }]
				: []
		}),
	}
}

export interface CreditPreviewTotals {
	netCents: number
	vatCents: number
	grossCents: number
}

export function creditPreviewTotals(
	lines: readonly InvoiceLine[],
	mode: CreditMode,
	inputs: QuantityInputs,
): CreditPreviewTotals {
	const bases = new Map<number, number>()
	for (const line of lines) {
		const net = creditPreviewNetCents([line], mode, inputs)
		const rate = line.vat_rate_basis_points ?? 0
		bases.set(rate, (bases.get(rate) ?? 0) + net)
	}
	let netCents = 0
	let vatCents = 0
	for (const [rate, net] of bases) {
		netCents += net
		vatCents += Math.round((net * rate) / 10000)
	}
	return { netCents, vatCents, grossCents: netCents + vatCents }
}

export interface CreditConsequence {
	kind: 'none' | 'reduces' | 'settles' | 'refund'
	after: number
}

export function creditConsequence(
	balanceCents: number,
	creditCents: number,
): CreditConsequence {
	if (!(creditCents > 0)) return { kind: 'none', after: balanceCents }
	const after = balanceCents - creditCents
	if (after < 0) return { kind: 'refund', after }
	return { kind: after === 0 ? 'settles' : 'reduces', after }
}

export function creditConsequenceText(
	balanceCents: number,
	creditCents: number,
): string | null {
	const consequence = creditConsequence(balanceCents, creditCents)
	if (consequence.kind === 'none') return null
	if (consequence.kind === 'settles')
		return 'Avec cet avoir, il ne reste plus rien à encaisser sur la facture.'
	if (consequence.kind === 'reduces')
		return `Le reste dû passera de ${formatMoney(balanceCents)} à ${formatMoney(consequence.after)}.`
	return `Le client aura payé trop : ${formatMoney(-consequence.after)} seront à lui rembourser.`
}
