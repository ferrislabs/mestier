import type { Schemas } from '#/api/api.client'

export type BillingPipelineStepId =
	| 'quote'
	| 'project'
	| 'deposit'
	| 'final'
	| 'paid'

export type BillingPipelineStepState =
	| 'done'
	| 'current'
	| 'pending'
	| 'blocked'
	| 'stopped'

export interface BillingPipelineStep {
	id: BillingPipelineStepId
	state: BillingPipelineStepState
	label: string
	/** Deposit is the only optional stop: a project can be billed with a
	 * single final invoice and never issue one. */
	optional?: boolean
}

const INVOICE_STATUS_RANK: Record<Schemas.InvoiceStatus, number> = {
	PAID: 3,
	PARTIALLY_PAID: 2,
	ISSUED: 1,
	DRAFT: 0,
	CANCELLED: 0,
}

/** Any invoice that still counts toward the pipeline — a draft never left
 * the drawer and a cancelled one never happened. */
function isCountedInvoice(invoice: Schemas.InvoiceResponse): boolean {
	return invoice.status !== 'DRAFT' && invoice.status !== 'CANCELLED'
}

function bestStatusLabel(invoices: Schemas.InvoiceResponse[]): string {
	const best = invoices.reduce((a, b) =>
		INVOICE_STATUS_RANK[b.status] > INVOICE_STATUS_RANK[a.status] ? b : a,
	)
	if (best.status === 'PAID') return 'Payée'
	if (best.status === 'PARTIALLY_PAID') return 'Partiellement payée'
	return 'Émise'
}

/**
 * Where a quote's billing stands, for the small stepper on the quote page.
 *
 * Reads only backend-computed statuses and amounts (`ProjectBillingSummary`,
 * `InvoiceStatus`/`InvoiceKind`) — no price arithmetic happens here, per the
 * single-source-of-truth rule on rentability figures.
 *
 * A declined or cancelled quote is a dead end: the quote step reports
 * `stopped` and everything after it is `blocked`, never `pending` — no
 * project can ever come from it.
 */
export function computeBillingPipelineSteps({
	quoteStatus,
	project,
	billingSummary,
	invoices,
}: {
	quoteStatus: Schemas.QuoteStatus
	project: Schemas.ProjectResponse | undefined
	billingSummary: Schemas.ProjectBillingSummaryResponse | undefined
	invoices: Schemas.InvoiceResponse[] | undefined
}): BillingPipelineStep[] {
	const quoteStopped = quoteStatus === 'DECLINED' || quoteStatus === 'CANCELLED'
	const quoteAccepted = quoteStatus === 'ACCEPTED'

	const quoteStep: BillingPipelineStep = quoteStopped
		? { id: 'quote', state: 'stopped', label: quoteStatusLabel(quoteStatus) }
		: {
				id: 'quote',
				state: quoteAccepted || project ? 'done' : 'current',
				label: 'Devis',
			}

	const projectStep: BillingPipelineStep = quoteStopped
		? { id: 'project', state: 'blocked', label: 'Projet créé' }
		: {
				id: 'project',
				state: project ? 'done' : quoteAccepted ? 'current' : 'pending',
				label: 'Projet créé',
			}

	const counted = invoices?.filter(isCountedInvoice) ?? []
	const deposits = counted.filter((invoice) => invoice.kind === 'DEPOSIT')
	const finals = counted.filter((invoice) => invoice.kind === 'FINAL')

	const depositStep: BillingPipelineStep = billingStep({
		id: 'deposit',
		label: 'Acompte facturé',
		unreachable: quoteStopped,
		projectReady: Boolean(project),
		matches: deposits,
		optional: true,
	})

	const finalStep: BillingPipelineStep = billingStep({
		id: 'final',
		label: 'Solde facturé',
		unreachable: quoteStopped,
		projectReady: Boolean(project),
		matches: finals,
	})

	const chargeable = counted.filter((invoice) => invoice.kind !== 'CREDIT_NOTE')
	const fullyBilled = (billingSummary?.remaining_cents ?? null) === 0
	const fullyPaid =
		fullyBilled &&
		chargeable.length > 0 &&
		chargeable.every((invoice) => invoice.status === 'PAID')

	const paidStep: BillingPipelineStep = quoteStopped
		? { id: 'paid', state: 'blocked', label: 'Payé' }
		: {
				id: 'paid',
				state: fullyPaid
					? 'done'
					: finalStep.state === 'done'
						? 'current'
						: 'pending',
				label: 'Payé',
			}

	return [quoteStep, projectStep, depositStep, finalStep, paidStep]
}

function billingStep({
	id,
	label,
	unreachable,
	projectReady,
	matches,
	optional,
}: {
	id: BillingPipelineStepId
	label: string
	unreachable: boolean
	projectReady: boolean
	matches: Schemas.InvoiceResponse[]
	optional?: boolean
}): BillingPipelineStep {
	if (unreachable) return { id, state: 'blocked', label, optional }
	if (matches.length > 0) {
		return {
			id,
			state: 'done',
			label: `${label} · ${bestStatusLabel(matches)}`,
			optional,
		}
	}
	return { id, state: projectReady ? 'current' : 'pending', label, optional }
}

function quoteStatusLabel(status: Schemas.QuoteStatus): string {
	if (status === 'DECLINED') return 'Devis refusé'
	if (status === 'CANCELLED') return 'Devis annulé'
	return 'Devis'
}
