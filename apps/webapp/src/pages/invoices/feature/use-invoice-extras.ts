// MOCK, pas d'endpoint, remplacé au lot B5 (création depuis un devis, date d'émission, remboursement, suppression de brouillon).
import { useSyncExternalStore } from 'react'
import {
	type InvoiceExtras,
	readInvoiceExtras,
	subscribeInvoiceExtras,
} from '#/pages/invoices/feature/mocks/invoice-extras'

export function useInvoiceExtras(): InvoiceExtras {
	return useSyncExternalStore(
		subscribeInvoiceExtras,
		readInvoiceExtras,
		readInvoiceExtras,
	)
}
