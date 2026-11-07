import { parsePercent } from '#/pages/quotes/lib/quote-draft'
import type { QuoteExtras } from '#/pages/quotes/lib/quote-extras'
import { lineExtrasAt } from '#/pages/quotes/lib/quote-extras'
import type { QuoteFormValues, QuoteLineFormValues } from '#/pages/quotes/types'

export function extrasPatchFromForm(
	values: QuoteFormValues,
): Partial<QuoteExtras> {
	return {
		ownerId: values.ownerId,
		validUntil: values.validUntil === '' ? null : values.validUntil,
		depositPercent: parsePercent(values.depositPercent) ?? 0,
		notes: values.notes.trim(),
		lines: values.lines.map((line) => ({
			discountPercent: parsePercent(line.discountPercent) ?? 0,
			frequency: line.frequency,
		})),
	}
}

export function extrasFormFields(
	extras: QuoteExtras,
): Pick<
	QuoteFormValues,
	'ownerId' | 'validUntil' | 'depositPercent' | 'notes'
> {
	return {
		ownerId: extras.ownerId,
		validUntil: extras.validUntil ?? '',
		depositPercent:
			extras.depositPercent > 0 ? String(extras.depositPercent) : '',
		notes: extras.notes,
	}
}

export function lineWithExtras(
	line: QuoteLineFormValues,
	extras: Pick<QuoteExtras, 'lines'>,
	index: number,
): QuoteLineFormValues {
	const { discountPercent, frequency } = lineExtrasAt(extras, index)
	return {
		...line,
		discountPercent: discountPercent > 0 ? String(discountPercent) : '',
		frequency,
	}
}

export function copiedExtras(extras: QuoteExtras): Partial<QuoteExtras> {
	return {
		ownerId: extras.ownerId,
		depositPercent: extras.depositPercent,
		notes: extras.notes,
		dealId: extras.dealId,
		lines: extras.lines,
	}
}
