import type { InvoiceLinePayload, InvoiceStatus } from '#/hooks/use-invoices'
import type { Quote } from '#/hooks/use-quotes'
import { addDaysIso } from '#/pages/invoices/lib/iso-date'
import {
	centsToEuros,
	formatVatRateBp,
	invoiceLineTotalCents,
} from '#/pages/invoices/types'

export const MAX_PAYMENT_TERMS_DAYS = 60
export const DEFAULT_PAYMENT_TERMS_DAYS = 30

const PERCENT = /^(\d+)([.,]\d)?$/
const DEFAULT_DEPOSIT_PERCENT = 30

export interface QuoteInvoiceLink {
	invoiceId: string
	kind: 'full' | 'deposit'
	percent: number
	status: InvoiceStatus
}

export interface QuoteInvoicing {
	invoiced: number
	remaining: number
	canFull: boolean
	canDeposit: boolean
	reasons: string[]
	existingInvoiceId: string | null
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

export function quoteInvoicing(
	quote: Pick<Quote, 'status'>,
	links: readonly QuoteInvoiceLink[],
): QuoteInvoicing {
	const active = links.filter((link) => link.status !== 'CANCELLED')
	const deposits = active.filter((link) => link.kind === 'deposit')
	const full = active.find((link) => link.kind === 'full') ?? null
	const invoiced = roundPercent(
		deposits.reduce((sum, link) => sum + link.percent, 0),
	)
	const remaining = Math.max(0, roundPercent(100 - invoiced))
	const accepted = quote.status === 'ACCEPTED'
	const canFull = accepted && full === null
	const canDeposit = canFull && remaining > 0
	const reasons = !accepted
		? ['Le devis doit être accepté.']
		: full !== null
			? ['Ce devis a déjà une facture.']
			: canDeposit
				? []
				: [
						'Les acomptes déjà facturés couvrent 100 % du devis : seule la facture complète est possible.',
					]
	return {
		invoiced,
		remaining,
		canFull,
		canDeposit,
		reasons,
		existingInvoiceId: full?.invoiceId ?? null,
		deposits: {
			issued: deposits.filter((link) => link.status !== 'DRAFT'),
			drafts: deposits.filter((link) => link.status === 'DRAFT'),
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

interface RateBase {
	rateBp: number | null
	netCents: number
}

function netByRate(quote: Pick<Quote, 'lines'>): RateBase[] {
	const bases = new Map<number | null, number>()
	for (const line of quote.lines) {
		const rateBp = line.vat_rate_bp ?? null
		const net = invoiceLineTotalCents({
			clientId: line.id,
			label: line.label,
			quantity: line.quantity,
			unitPrice: centsToEuros(line.unit_price_cents),
			vatRateBp: '',
		})
		bases.set(rateBp, (bases.get(rateBp) ?? 0) + net)
	}
	return [...bases].map(([rateBp, netCents]) => ({ rateBp, netCents }))
}

function depositBase(netCents: number, percent: number): number {
	return Math.round((netCents * percent) / 100)
}

export function depositLines(
	quote: Pick<Quote, 'lines' | 'reference'>,
	percent: number,
): InvoiceLinePayload[] {
	const bases = netByRate(quote)
	const reference = quote.reference ? ` ${quote.reference}` : ''
	return bases.map((base) => {
		const rate =
			bases.length > 1 && base.rateBp !== null
				? ` (TVA ${formatVatRateBp(base.rateBp)})`
				: ''
		return {
			label: `Acompte de ${percent} % sur devis${reference}${rate}`,
			quantity: '1',
			unit_price_cents: depositBase(base.netCents, percent),
			vat_rate_basis_points: base.rateBp,
		}
	})
}

export function depositPreviewCents(
	quote: Pick<Quote, 'lines'>,
	percent: number,
): number {
	if (!(percent > 0)) return 0
	return netByRate(quote).reduce((sum, base) => {
		const net = depositBase(base.netCents, percent)
		const vat = Math.round((net * (base.rateBp ?? 0)) / 10000)
		return sum + net + vat
	}, 0)
}

export function fullInvoiceLines(
	quote: Pick<Quote, 'lines'>,
): InvoiceLinePayload[] {
	return quote.lines.map((line) => ({
		label: line.label,
		quantity: line.quantity,
		unit_price_cents: line.unit_price_cents,
		vat_rate_basis_points: line.vat_rate_bp ?? null,
	}))
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

export function paymentTermsDueDate(
	today: string,
	input: string,
): string | null {
	const terms = parseTerms(input)
	return terms === null ? null : addDaysIso(today, terms)
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

export function activeQuoteLinks(input: {
	quoteId: string
	links: Record<
		string,
		{ quoteId: string; kind: 'full' | 'deposit'; percent: number }
	>
	hiddenIds: readonly string[]
	statusById: ReadonlyMap<string, InvoiceStatus>
}): QuoteInvoiceLink[] {
	return Object.entries(input.links)
		.filter(
			([invoiceId, link]) =>
				link.quoteId === input.quoteId && !input.hiddenIds.includes(invoiceId),
		)
		.map(([invoiceId, link]) => ({
			invoiceId,
			kind: link.kind,
			percent: link.percent,
			status: input.statusById.get(invoiceId) ?? 'ISSUED',
		}))
}
