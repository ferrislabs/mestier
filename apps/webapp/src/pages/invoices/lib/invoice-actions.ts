import type { InvoiceStatus } from '#/hooks/use-invoices'

export interface ActionState {
	visible: boolean
	enabled: boolean
	reason: string | null
}

export type InvoiceActionKey =
	| 'issue'
	| 'cancel'
	| 'recordPayment'
	| 'send'
	| 'remind'
	| 'creditNote'

export interface InvoiceActions extends Record<InvoiceActionKey, ActionState> {
	sendLabel: 'Envoyer au client' | 'Renvoyer'
}

export const ACTION_LABELS: Record<InvoiceActionKey, string> = {
	issue: 'Émettre la facture',
	cancel: 'Annuler la facture',
	recordPayment: 'Enregistrer un paiement',
	send: 'Envoyer au client',
	remind: 'Relancer',
	creditNote: 'Émettre un avoir',
}

const ACTION_ORDER: readonly InvoiceActionKey[] = [
	'issue',
	'recordPayment',
	'send',
	'remind',
	'creditNote',
	'cancel',
]

const HIDDEN: ActionState = { visible: false, enabled: false, reason: null }
const ENABLED: ActionState = { visible: true, enabled: true, reason: null }

function blocked(reason: string): ActionState {
	return { visible: true, enabled: false, reason }
}

export interface InvoiceActionsInput {
	status: InvoiceStatus
	grossCents: number
	creditedCents: number | null
	remainingCents: number | null
	sentAt: string | null
}

export function invoiceActions(input: InvoiceActionsInput): InvoiceActions {
	const sendLabel = input.sentAt === null ? 'Envoyer au client' : 'Renvoyer'
	if (input.status === 'DRAFT')
		return {
			issue: ENABLED,
			cancel: ENABLED,
			recordPayment: HIDDEN,
			send: HIDDEN,
			remind: HIDDEN,
			creditNote: HIDDEN,
			sendLabel,
		}
	if (input.status === 'CANCELLED')
		return {
			issue: HIDDEN,
			cancel: HIDDEN,
			recordPayment: blocked(
				'La facture est annulée : il n’y a plus rien à encaisser.',
			),
			send: blocked('La facture est annulée : il n’y a plus rien à envoyer.'),
			remind: HIDDEN,
			creditNote: HIDDEN,
			sendLabel,
		}
	const settled =
		input.status === 'PAID' ||
		(input.remainingCents !== null && input.remainingCents <= 0)
	const nothingToCredit =
		input.creditedCents !== null &&
		input.grossCents > 0 &&
		input.creditedCents >= input.grossCents
	return {
		issue: HIDDEN,
		cancel: input.status === 'ISSUED' ? ENABLED : HIDDEN,
		recordPayment: settled
			? blocked('La facture est soldée : il n’y a plus rien à encaisser.')
			: ENABLED,
		send: ENABLED,
		remind: settled ? HIDDEN : ENABLED,
		creditNote: nothingToCredit
			? blocked(
					'Plus rien à créditer : les avoirs émis couvrent déjà toute la facture.',
				)
			: ENABLED,
		sendLabel,
	}
}

export function unavailableNotes(
	actions: InvoiceActions,
): { key: InvoiceActionKey; label: string; reason: string }[] {
	return ACTION_ORDER.flatMap((key) => {
		const action = actions[key]
		if (!action.visible || action.enabled || action.reason === null) return []
		const label = key === 'send' ? actions.sendLabel : ACTION_LABELS[key]
		return [{ key, label, reason: action.reason }]
	})
}
