// MOCK, pas d'endpoint, remplacé au lot B5 (création depuis un devis, date d'émission, remboursement, suppression de brouillon).
import type { RefundInput } from '#/pages/invoices/lib/payment-form'

export interface QuoteLinkRecord {
	quoteId: string
	kind: 'full' | 'deposit'
	percent: number
}

export type RefundRecord = RefundInput & { id: string }

export interface InvoiceExtras {
	quoteLinks: Record<string, QuoteLinkRecord>
	issueDates: Record<string, string>
	refunds: Record<string, RefundRecord[]>
	hiddenDrafts: string[]
}

const STORAGE_KEY = 'mestier.mock.invoice-extras'

const EMPTY: InvoiceExtras = {
	quoteLinks: {},
	issueDates: {},
	refunds: {},
	hiddenDrafts: [],
}

function load(): InvoiceExtras {
	try {
		const raw = window.localStorage.getItem(STORAGE_KEY)
		return raw ? { ...EMPTY, ...JSON.parse(raw) } : EMPTY
	} catch {
		return EMPTY
	}
}

let current: InvoiceExtras | null = null
const listeners = new Set<() => void>()

export function readInvoiceExtras(): InvoiceExtras {
	current ??= load()
	return current
}

function persist(next: InvoiceExtras): void {
	try {
		window.localStorage.setItem(STORAGE_KEY, JSON.stringify(next))
	} catch {
		current = next
	}
}

function write(next: InvoiceExtras): void {
	current = next
	persist(next)
	for (const listener of listeners) listener()
}

export function subscribeInvoiceExtras(listener: () => void): () => void {
	listeners.add(listener)
	return () => listeners.delete(listener)
}

export function setQuoteLink(invoiceId: string, link: QuoteLinkRecord): void {
	const extras = readInvoiceExtras()
	write({ ...extras, quoteLinks: { ...extras.quoteLinks, [invoiceId]: link } })
}

export function setIssueDate(invoiceId: string, day: string): void {
	const extras = readInvoiceExtras()
	write({ ...extras, issueDates: { ...extras.issueDates, [invoiceId]: day } })
}

export function hideDraft(invoiceId: string): void {
	const extras = readInvoiceExtras()
	if (extras.hiddenDrafts.includes(invoiceId)) return
	write({ ...extras, hiddenDrafts: [...extras.hiddenDrafts, invoiceId] })
}

export function recordRefund(invoiceId: string, refund: RefundInput): void {
	const extras = readInvoiceExtras()
	const record: RefundRecord = { ...refund, id: crypto.randomUUID() }
	write({
		...extras,
		refunds: {
			...extras.refunds,
			[invoiceId]: [...(extras.refunds[invoiceId] ?? []), record],
		},
	})
}

export function resetInvoiceExtras(): void {
	write(EMPTY)
	window.localStorage.removeItem(STORAGE_KEY)
}
