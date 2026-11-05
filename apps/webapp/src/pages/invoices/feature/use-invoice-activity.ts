// MOCK: invoice activity (send, reminders, internal note) — no backend endpoint yet. Replace with the invoice send/reminder/note endpoints once they exist.
import { useState, useSyncExternalStore } from 'react'
import {
	type InvoiceActivityRecord,
	readInvoiceActivity,
	subscribeInvoiceActivity,
	writeInvoiceActivity,
} from '#/pages/invoices/feature/mocks/invoice-activity'

const LATENCY_MS = 250

function after<T>(work: () => T): Promise<T> {
	return new Promise((resolve) => {
		setTimeout(() => resolve(work()), LATENCY_MS)
	})
}

export function useInvoiceActivity(invoiceId: string) {
	const data = useSyncExternalStore(
		subscribeInvoiceActivity,
		() => readInvoiceActivity(invoiceId),
		() => readInvoiceActivity(invoiceId),
	)
	return { data, isLoading: false }
}

function useMockMutation<Args extends unknown[]>(
	run: (...args: Args) => Promise<void>,
) {
	const [isPending, setIsPending] = useState(false)
	const mutateAsync = async (...args: Args) => {
		setIsPending(true)
		try {
			await run(...args)
		} finally {
			setIsPending(false)
		}
	}
	return { mutateAsync, isPending }
}

export function useSendInvoice(invoiceId: string) {
	return useMockMutation(() =>
		after(() =>
			writeInvoiceActivity(invoiceId, { sentAt: new Date().toISOString() }),
		),
	)
}

export function useRemindInvoice(invoiceId: string) {
	return useMockMutation(() =>
		after(() => {
			const current: InvoiceActivityRecord = readInvoiceActivity(invoiceId)
			writeInvoiceActivity(invoiceId, {
				reminders: [...current.reminders, new Date().toISOString()],
			})
		}),
	)
}

export function useSaveInvoiceNote(invoiceId: string) {
	return useMockMutation((internalNote: string) =>
		after(() => writeInvoiceActivity(invoiceId, { internalNote })),
	)
}
