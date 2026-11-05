// MOCK: Stripe events — no backend endpoint yet. Replace with the Stripe webhook events list and the review actions (link customer, allocate to invoice, ignore) of the payments backend.
import {
	createMockStore,
	resolveAfterDelay,
} from '#/pages/customers/feature/mocks/mock-store'
import type {
	StripeEvent,
	StripeReviewReason,
	StripeScenario,
} from '#/pages/stripe/types'

function iso(daysAgo: number, hour: number): string {
	const moment = new Date()
	moment.setDate(moment.getDate() - daysAgo)
	moment.setHours(hour, 12, 0, 0)
	return moment.toISOString()
}

function event(
	partial: Partial<StripeEvent> & Pick<StripeEvent, 'id'>,
): StripeEvent {
	return {
		type: 'payment_intent.succeeded',
		receivedAt: iso(1, 10),
		customerId: null,
		customerEmail: '',
		customerName: '',
		amount: 0,
		description: '',
		stripeInvoiceNumber: null,
		paymentIntentId: null,
		mestierInvoiceNumber: null,
		lines: [],
		status: 'processed',
		reviewReason: null,
		accountId: null,
		accountName: '',
		paymentId: null,
		outcome: '',
		handledById: 'webhook',
		...partial,
	}
}

const INITIAL: readonly StripeEvent[] = [
	event({
		id: 'evt_1Qx9aLemoine',
		receivedAt: iso(0, 9),
		customerId: 'cus_QLemoine01',
		customerEmail: 'compta@boulangerie-lemoine.fr',
		customerName: 'Boulangerie Lemoine',
		amount: 125000,
		paymentIntentId: 'pi_3Qx9aLemoine',
		mestierInvoiceNumber: 'F-2026-109',
		accountName: 'Boulangerie Lemoine',
		paymentId: 'p-1',
		outcome: 'Paiement affecté à F-2026-109',
	}),
	event({
		id: 'evt_1Qx8bAtelier',
		receivedAt: iso(3, 15),
		customerId: 'cus_QAtelier0X',
		customerEmail: 'contact@atelier-bois.fr',
		customerName: 'Atelier Bois & Co',
		amount: 49000,
		description: 'Acompte atelier',
		paymentIntentId: 'pi_3Qx8bAtelier',
		status: 'to_review',
		reviewReason: 'no_open_invoice',
		accountName: 'Atelier Bois & Co',
		paymentId: 'p-2',
		handledById: null,
	}),
	event({
		id: 'evt_1Qx7cInconnu',
		receivedAt: iso(5, 11),
		customerId: 'cus_QInconnu77',
		customerEmail: 'pierre.martel@example.com',
		customerName: 'Pierre Martel',
		amount: 32000,
		description: 'Paiement en ligne',
		paymentIntentId: 'pi_3Qx7cInconnu',
		status: 'to_review',
		reviewReason: 'unknown_customer',
		paymentId: 'p-3',
		handledById: null,
	}),
	event({
		id: 'evt_1Qx6dInvoice',
		type: 'invoice.paid',
		receivedAt: iso(9, 14),
		customerId: 'cus_QDurand0001',
		customerEmail: 'famille.durand@example.com',
		customerName: 'Famille Durand',
		amount: 186000,
		stripeInvoiceNumber: 'STR-0042',
		accountName: 'Famille Durand',
		lines: [
			{
				description: 'Pose de volets roulants',
				quantity: 3,
				unitPrice: 51667,
				vatRate: 10,
			},
		],
		outcome: 'Facture Stripe STR-0042 importée',
	}),
	event({
		id: 'evt_1Qx5eRefund',
		type: 'charge.refunded',
		receivedAt: iso(14, 16),
		customerId: 'cus_QLemoine01',
		customerEmail: 'compta@boulangerie-lemoine.fr',
		customerName: 'Boulangerie Lemoine',
		amount: 12000,
		accountName: 'Boulangerie Lemoine',
		outcome: 'Remboursement enregistré',
	}),
	event({
		id: 'evt_1Qx4fTest',
		receivedAt: iso(21, 8),
		customerId: 'cus_QTest00001',
		customerEmail: 'test@stripe.com',
		customerName: 'Test Stripe',
		amount: 100,
		paymentIntentId: 'pi_3Qx4fTest',
		status: 'ignored',
		handledById: 'baptiste',
	}),
]

export const stripeStore = createMockStore<readonly StripeEvent[]>(INITIAL)

export function useStripeEvents() {
	return { data: stripeStore.use(), isLoading: false }
}

function patchEvent(eventId: string, patch: Partial<StripeEvent>) {
	stripeStore.set((events) =>
		events.map((item) => (item.id === eventId ? { ...item, ...patch } : item)),
	)
}

export function useLinkStripeCustomer() {
	return {
		mutateAsync: async (
			eventId: string,
			customer: { id: string; name: string },
		) => {
			await resolveAfterDelay(null)
			const reason: StripeReviewReason = 'no_open_invoice'
			patchEvent(eventId, {
				accountId: customer.id,
				accountName: customer.name,
				status: 'to_review',
				reviewReason: reason,
			})
		},
	}
}

export function useAllocateStripeEvent() {
	return {
		mutateAsync: async (eventId: string, invoiceLabel: string) => {
			await resolveAfterDelay(null)
			patchEvent(eventId, {
				status: 'processed',
				reviewReason: null,
				mestierInvoiceNumber: invoiceLabel,
				outcome: `Paiement affecté à ${invoiceLabel}`,
				handledById: 'baptiste',
			})
		},
	}
}

export function useIgnoreStripeEvent() {
	return {
		mutateAsync: async (eventId: string) => {
			await resolveAfterDelay(null)
			patchEvent(eventId, {
				status: 'ignored',
				reviewReason: null,
				handledById: 'baptiste',
			})
		},
	}
}

export function useSimulateStripeEvent() {
	return {
		mutateAsync: async (
			scenario: StripeScenario,
			invoiceLabel: string | null,
		): Promise<StripeEvent | null> => {
			await resolveAfterDelay(null)
			const id = `evt_${crypto.randomUUID().slice(0, 12)}`
			const base = {
				id,
				receivedAt: new Date().toISOString(),
				customerId: 'cus_QLemoine01',
				customerEmail: 'compta@boulangerie-lemoine.fr',
				customerName: 'Boulangerie Lemoine',
				accountName: 'Boulangerie Lemoine',
				paymentIntentId: `pi_${id.slice(4)}`,
			}
			let created: StripeEvent | null = null
			if (scenario === 'payment_succeeded') {
				if (invoiceLabel === null) return null
				created = event({
					...base,
					amount: 49000,
					mestierInvoiceNumber: invoiceLabel,
					paymentId: `p-${id}`,
					outcome: `Paiement affecté à ${invoiceLabel}`,
				})
			} else if (scenario === 'invoice_paid') {
				created = event({
					...base,
					type: 'invoice.paid',
					amount: 90576,
					paymentIntentId: null,
					stripeInvoiceNumber: 'STR-0043',
					lines: [
						{
							description: 'Entretien annuel',
							quantity: 1,
							unitPrice: 75480,
							vatRate: 20,
						},
					],
					outcome: 'Facture Stripe STR-0043 importée',
				})
			} else {
				created = event({
					...base,
					customerId: 'cus_QNouveau99',
					customerEmail: 'nouveau.client@example.com',
					customerName: 'Nouveau client',
					accountName: '',
					amount: 21000,
					paymentId: `p-${id}`,
					status: 'to_review',
					reviewReason: 'unknown_customer',
					handledById: null,
				})
			}
			stripeStore.set((events) => [created as StripeEvent, ...events])
			return created
		},
	}
}
