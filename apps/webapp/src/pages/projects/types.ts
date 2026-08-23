import { z } from 'zod'

/**
 * `?projectId=` highlights one row, so the profitability screen can link
 * straight at the project it just costed. `?archived=` widens the list.
 *
 * `.catch` on both: a hand-edited URL should land on the list rather than a
 * router error page.
 */
export const projectsSearchSchema = z.object({
	projectId: z.string().optional().catch(undefined),
	archived: z.boolean().catch(false),
})

export type ProjectsSearch = z.infer<typeof projectsSearchSchema>

export interface ProjectFormValues {
	name: string
	/** `''` means internal — no customer, deliberately. */
	customerId: string
	quoteId: string
	/**
	 * The address a deposit or a final invoice bills to. Pinned on the
	 * project itself, not chosen per invoice: `IssueDepositCommand`/
	 * `IssueFinalInvoiceCommand` take no address of their own — see
	 * `required_customer_fields` in `domain::invoice::service`, which
	 * refuses to issue either without one already set on the project row.
	 * A free-amount invoice (`#322`'s third Invoicer mode) does not need
	 * this: it composes a normal `create_invoice` call, which still takes
	 * its own address like any manually created invoice.
	 */
	customerContextId: string
}

export const EMPTY_PROJECT_FORM: ProjectFormValues = {
	name: '',
	customerId: '',
	quoteId: '',
	customerContextId: '',
}

/** `''` is how a `<select>` says "none"; the API wants `null`. */
export function optionalId(value: string): string | null {
	const trimmed = value.trim()

	return trimmed === '' ? null : trimmed
}

/** "Start a project from a template" — picks the template, then the same name/customer/quote shape a blank project asks for, plus a start date every task shape's offset resolves against. */
export interface InstantiateTemplateFormValues {
	templateId: string
	name: string
	startDate: string
	customerId: string
	quoteId: string
}

export function emptyInstantiateTemplateForm(
	today: string,
): InstantiateTemplateFormValues {
	return {
		templateId: '',
		name: '',
		startDate: today,
		customerId: '',
		quoteId: '',
	}
}

/**
 * The Invoicer's three ways to bill a project (#322): everything still
 * owed, a percentage of the quote, or a free-typed amount.
 */
export type InvoicerMode = 'REMAINING' | 'PERCENTAGE' | 'FREE_AMOUNT'

/**
 * `REMAINING` and `PERCENTAGE` both need a quote to compute against — the
 * same precondition `issue_deposit`/`issue_final_invoice` enforce
 * server-side (`project ... has no quote`, see `domain::invoice::service`).
 * `FREE_AMOUNT` never needed one: `create_invoice` never required a quote
 * either.
 */
export function canInvoiceAgainstQuote(quotedCents: number | null): boolean {
	return quotedCents !== null
}

/**
 * A percent string (`"30"`, `"33,33"`) to basis points, rounded to the
 * nearest whole basis point. `null` when there is nothing usable to read.
 */
export function percentToBasisPoints(value: string): number | null {
	const normalized = value.replace(',', '.').trim()
	if (
		!/^\d*\.?\d*$/.test(normalized) ||
		normalized === '' ||
		normalized === '.'
	) {
		return null
	}

	const percent = Number(normalized)
	if (!Number.isFinite(percent) || percent <= 0) return null

	return Math.round(percent * 100)
}

/**
 * A *preview* of what `issue_deposit`'s own `deposit_amount_cents` will
 * compute server-side for `percentageBp` basis points of `quotedCents` — see
 * `libs/core/src/domain/invoice/service.rs`: `deposit_amount_cents` is
 * `div_round_half_even(percentage_bp * quoted_cents, 10_000)`, restated here
 * with plain numbers rather than `rust_decimal` because both operands are
 * bounded well under `Number.MAX_SAFE_INTEGER`.
 *
 * This is browser-side money arithmetic, which CLAUDE.md otherwise forbids
 * for anything that gets acted on. The exception: this number is never sent
 * anywhere and never persisted — it only decides what the preview panel
 * shows before the click. The invoice that actually gets issued is built
 * from whatever `useIssueDeposit` returns from the server, computed by the
 * exact same formula. Do not "fix" this into a server round-trip on every
 * keystroke of the percentage input: that would defeat the point of a
 * preview, and the backend already re-derives and validates the real amount
 * at submit time regardless of what this function returned.
 */
export function depositPreviewCents(
	quotedCents: number,
	percentageBp: number,
): number {
	const numerator = percentageBp * quotedCents
	const denominator = 10_000

	const quotient = Math.floor(numerator / denominator)
	const remainder = numerator - quotient * denominator
	const doubled = remainder * 2

	if (doubled > denominator) return quotient + 1
	if (doubled < denominator) return quotient
	return quotient % 2 === 0 ? quotient : quotient + 1
}
