import type { Schemas } from '#/api/api.client'
import { apiOfFrequency } from '#/pages/quotes/lib/billing-frequency'
import { lineTermsAt, type QuoteTerms } from '#/pages/quotes/lib/quote-terms'
import type { QuoteFormValues, QuoteLineFormValues } from '#/pages/quotes/types'

const DEPOSIT_DECIMALS = 1
const DISCOUNT_DECIMALS = 2

export function percentText(input: string): string {
	const normalized = input.replace(',', '.').trim()
	return normalized === '' ? '0' : normalized
}

export function isPercentValid(input: string, decimals: number): boolean {
	const normalized = input.replace(',', '.').trim()
	if (normalized === '') return true
	if (!/^\d+(\.\d+)?$/.test(normalized)) return false
	const [, fraction = ''] = normalized.split('.')
	const trimmed = fraction.replace(/0+$/, '')
	const value = Number(normalized)
	return value >= 0 && value <= 100 && trimmed.length <= decimals
}

export function areTermsValid(values: QuoteFormValues): boolean {
	return (
		isPercentValid(values.depositPercent, DEPOSIT_DECIMALS) &&
		values.lines.every((line) =>
			isPercentValid(line.discountPercent, DISCOUNT_DECIMALS),
		)
	)
}

export function termsFormFields(
	terms: QuoteTerms,
): Pick<
	QuoteFormValues,
	'ownerId' | 'validUntil' | 'depositPercent' | 'notes'
> {
	return {
		ownerId: terms.ownerId,
		validUntil: terms.validUntil ?? '',
		depositPercent:
			terms.depositPercent > 0 ? String(terms.depositPercent) : '',
		notes: terms.notes,
	}
}

export function lineWithTerms(
	line: QuoteLineFormValues,
	terms: Pick<QuoteTerms, 'lines'>,
	index: number,
): QuoteLineFormValues {
	const { discountPercent, frequency } = lineTermsAt(terms, index)
	return {
		...line,
		discountPercent: discountPercent > 0 ? String(discountPercent) : '',
		frequency,
	}
}

export interface QuoteTermsBody {
	valid_until: string | null
	deposit_percent: string
	owner_member_id: string | null
	notes: string | null
}

export function termsBody(values: QuoteFormValues): QuoteTermsBody {
	return {
		valid_until: values.validUntil === '' ? null : values.validUntil,
		deposit_percent: percentText(values.depositPercent),
		owner_member_id: values.ownerId,
		notes: values.notes.trim() || null,
	}
}

export function lineTermsBody(line: QuoteLineFormValues): {
	discount_percent: string
	billing_frequency: Schemas.BillingFrequency
} {
	return {
		discount_percent: percentText(line.discountPercent),
		billing_frequency: apiOfFrequency(line.frequency),
	}
}
