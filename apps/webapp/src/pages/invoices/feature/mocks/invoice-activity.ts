// MOCK: invoice activity (send, reminders, internal note) — no backend endpoint yet. Replace with the invoice send/reminder/note endpoints once they exist.
export interface InvoiceActivityRecord {
	sentAt: string | null
	reminders: string[]
	internalNote: string
}

const EMPTY: InvoiceActivityRecord = {
	sentAt: null,
	reminders: [],
	internalNote: '',
}

const records = new Map<string, InvoiceActivityRecord>()
const listeners = new Set<() => void>()

export function readInvoiceActivity(invoiceId: string): InvoiceActivityRecord {
	return records.get(invoiceId) ?? EMPTY
}

export function writeInvoiceActivity(
	invoiceId: string,
	patch: Partial<InvoiceActivityRecord>,
): void {
	records.set(invoiceId, { ...readInvoiceActivity(invoiceId), ...patch })
	for (const listener of listeners) listener()
}

export function subscribeInvoiceActivity(listener: () => void): () => void {
	listeners.add(listener)
	return () => listeners.delete(listener)
}

export function resetInvoiceActivity(): void {
	records.clear()
	for (const listener of listeners) listener()
}
