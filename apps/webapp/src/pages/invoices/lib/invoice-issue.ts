import type { Invoice } from '#/hooks/use-invoices'
import { isIsoDate } from '#/pages/invoices/lib/iso-date'
import { formatDocumentDate } from '#/pages/quotes/lib/doc-format'

export function lastIssuedDate(
	invoices: readonly Pick<Invoice, 'kind' | 'status' | 'issued_at'>[],
): string | null {
	const days = invoices
		.filter(
			(invoice) =>
				invoice.kind !== 'CREDIT_NOTE' &&
				invoice.status !== 'DRAFT' &&
				invoice.issued_at != null,
		)
		.map((invoice) => (invoice.issued_at as string).slice(0, 10))
		.sort()
	return days.at(-1) ?? null
}

export function issueDateBounds(
	invoices: readonly Pick<Invoice, 'kind' | 'status' | 'issued_at'>[],
	today: string,
): { min: string | null; max: string } {
	return { min: lastIssuedDate(invoices), max: today }
}

export function issueDateErrors(
	issueDate: string,
	today: string,
	last: string | null,
): string[] {
	if (!isIsoDate(issueDate)) return ['La date d’émission n’est pas valide.']
	if (issueDate > today)
		return ['La date d’émission ne peut pas être dans le futur.']
	if (last !== null && issueDate < last)
		return [
			`La date d’émission ne peut pas précéder celle de la dernière facture émise (${formatDocumentDate(last)}) : la numérotation doit rester chronologique.`,
		]
	return []
}
