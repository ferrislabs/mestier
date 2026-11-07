import type { QuoteInvoicingData } from '#/hooks/use-invoices'

export const MAX_PAYMENT_TERMS_DAYS = 60
export const DEFAULT_PAYMENT_TERMS_DAYS = 30

const PERCENT = /^(\d+)([.,]\d)?$/
const DEFAULT_DEPOSIT_PERCENT = 30

export interface QuoteInvoiceLink {
	invoiceId: string
	number: string | null
	draft: boolean
	grossCents: number
	issuedAt: string | null
}

export interface QuoteInvoicing {
	invoiced: number
	remaining: number
	canFull: boolean
	canDeposit: boolean
	reasons: string[]
	existingInvoiceId: string | null
	existingInvoiceNumber: string | null
	deposits: { issued: QuoteInvoiceLink[]; drafts: QuoteInvoiceLink[] }
}

function roundPercent(value: number): number {
	return Math.round(value * 10) / 10
}

export function parsePercent(input: string): number | null {
	const match = PERCENT.exec(input.replace(/[\s%]/g, ''))
	return match
		? Number(`${match[1]}${(match[2] ?? '').replace(',', '.')}`)
		: null
}

export function formatPercent(value: number): string {
	return `${String(value).replace('.', ',')} %`
}

const BLOCKER_REASONS: Record<
	NonNullable<QuoteInvoicingData['blocker']>,
	string
> = {
	QUOTE_NOT_ACCEPTED: 'Le devis doit être accepté.',
	QUOTE_ALREADY_INVOICED: 'Ce devis a déjà une facture.',
	DEPOSITS_COVER_QUOTE:
		'Les acomptes déjà facturés couvrent 100 % du devis : seule la facture complète est possible.',
}

export function quoteInvoicingView(data: QuoteInvoicingData): QuoteInvoicing {
	const links = data.invoices.map(
		(invoice): QuoteInvoiceLink & { kind: string } => ({
			kind: invoice.kind,
			invoiceId: invoice.id,
			number: invoice.number ?? null,
			draft: invoice.status === 'DRAFT',
			grossCents: invoice.gross_cents,
			issuedAt: invoice.issued_at ?? null,
		}),
	)
	const deposits = links.filter((link) => link.kind === 'DEPOSIT')
	const full = data.invoices.find((invoice) => invoice.kind === 'STANDARD')

	return {
		invoiced: Number(data.invoiced_percent),
		remaining: Number(data.remaining_percent),
		canFull: data.can_full,
		canDeposit: data.can_deposit,
		reasons: data.blocker ? [BLOCKER_REASONS[data.blocker]] : [],
		existingInvoiceId: full?.id ?? null,
		existingInvoiceNumber: full?.number ?? null,
		deposits: {
			issued: deposits.filter((link) => !link.draft),
			drafts: deposits.filter((link) => link.draft),
		},
	}
}

export function defaultDepositPercent(remaining: number): number {
	return Math.min(DEFAULT_DEPOSIT_PERCENT, remaining)
}

export function depositPercentError(
	invoiced: number,
	input: string,
): string | null {
	const percent = parsePercent(input)
	if (percent === null)
		return 'Saisis un pourcentage valide, par exemple 30 ou 12,5.'
	if (!(percent > 0)) return 'Le pourcentage doit être supérieur à zéro.'
	if (percent + invoiced > 100)
		return `Le pourcentage dépasse le reste à facturer (${formatPercent(Math.max(0, roundPercent(100 - invoiced)))}).`
	return null
}

export function parseTerms(input: string): number | null {
	const text = input.trim()
	return /^\d+$/.test(text) ? Number(text) : null
}

export function newInvoiceTermsError(input: string): string | null {
	const terms = parseTerms(input)
	return terms === null || terms > MAX_PAYMENT_TERMS_DAYS
		? `Le délai de paiement doit être compris entre 0 et ${MAX_PAYMENT_TERMS_DAYS} jours.`
		: null
}

export type QuoteInvoiceMode = 'deposit' | 'full'

export function initialQuoteMode(
	info: Pick<QuoteInvoicing, 'canFull' | 'canDeposit' | 'invoiced'>,
	preferred: 'acompte' | 'complete' | undefined,
): QuoteInvoiceMode {
	if (preferred === 'acompte' && info.canDeposit) return 'deposit'
	if (preferred === 'complete' && info.canFull) return 'full'
	return info.canFull ? 'full' : 'deposit'
}

export interface NewInvoiceSearch {
	quoteId: string | undefined
	clientId: string | undefined
	mode: 'acompte' | 'complete' | undefined
}

export function readNewInvoiceSearch(search: string): NewInvoiceSearch {
	const params = new URLSearchParams(search)
	const mode = params.get('mode')
	return {
		quoteId: params.get('devis') || undefined,
		clientId: params.get('client') || undefined,
		mode: mode === 'acompte' || mode === 'complete' ? mode : undefined,
	}
}
