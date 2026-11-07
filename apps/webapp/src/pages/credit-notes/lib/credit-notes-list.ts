import type { Customer } from '#/hooks/use-customers'
import type { Invoice } from '#/hooks/use-invoices'
import { creditNoteKindLabel } from '#/pages/invoices/lib/invoice-document'
import { customerDisplayName } from '#/pages/invoices/types'
import { normalizeText, plural } from '#/pages/quotes/lib/doc-format'

export interface CreditNoteRow {
	note: Invoice
	source: Invoice | undefined
	sourceLabel: string
	customerName: string
	kindLabel: string
	totalCents: number
}

function dateKey(note: Invoice): string {
	return note.issued_at ?? note.created_at
}

export function creditNoteRows(
	notes: readonly Invoice[],
	invoices: readonly Invoice[],
	customers: readonly Customer[],
): CreditNoteRow[] {
	return notes
		.map((note) => {
			const source = invoices.find((item) => item.id === note.source_invoice_id)
			const customer = customers.find((item) => item.id === note.customer_id)
			return {
				note,
				source,
				sourceLabel: source ? (source.number ?? 'Brouillon') : '—',
				customerName: customer ? customerDisplayName(customer) : '',
				kindLabel: creditNoteKindLabel(note),
				totalCents: note.gross_cents,
			}
		})
		.sort(
			(a, b) =>
				dateKey(b.note).localeCompare(dateKey(a.note)) ||
				(b.note.number ?? '').localeCompare(a.note.number ?? ''),
		)
}

export function filterCreditNoteRows(
	rows: readonly CreditNoteRow[],
	query: string,
): CreditNoteRow[] {
	const wanted = normalizeText(query)
	if (wanted === '') return [...rows]
	return rows.filter((row) =>
		[
			row.note.number ?? '',
			row.sourceLabel,
			row.customerName,
			row.note.reason ?? '',
		]
			.map(normalizeText)
			.join('\n')
			.includes(wanted),
	)
}

export interface CreditNotesSummary {
	count: number
	totalCents: number
	cancellations: number
	corrections: number
}

export function creditNotesSummary(
	rows: readonly CreditNoteRow[],
): CreditNotesSummary {
	const cancellations = rows.filter(
		(row) => row.kindLabel === 'Annulation totale',
	).length
	return {
		count: rows.length,
		totalCents: rows.reduce((sum, row) => sum + row.totalCents, 0),
		cancellations,
		corrections: rows.length - cancellations,
	}
}

export function kindsLabel(
	summary: Pick<CreditNotesSummary, 'cancellations' | 'corrections'>,
): string {
	const parts = [
		...(summary.cancellations > 0
			? [
					`${summary.cancellations} ${plural(summary.cancellations, 'annulation totale', 'annulations totales')}`,
				]
			: []),
		...(summary.corrections > 0
			? [
					`${summary.corrections} ${plural(summary.corrections, 'correction partielle', 'corrections partielles')}`,
				]
			: []),
	]
	return parts.length === 0 ? 'Aucun avoir émis' : parts.join(', ')
}
