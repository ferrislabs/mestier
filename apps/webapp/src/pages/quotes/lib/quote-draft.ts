import type { QuoteStatus } from '#/hooks/use-quotes'
import { formatDocumentDate } from '#/pages/quotes/lib/doc-format'
import type { QuoteLineFormValues } from '#/pages/quotes/types'

export const NO_OWNER = 'none'
export const DEFAULT_VALIDITY_DAYS = 30

export interface ValidityNote {
	text: string
	expired: boolean
}

export function addDays(isoDate: string, days: number): string {
	const [year = 0, month = 1, day = 1] = isoDate.split('-').map(Number)
	const moved = new Date(Date.UTC(year, month - 1, day + days))
	return moved.toISOString().slice(0, 10)
}

export function defaultValidUntil(today: string): string {
	return addDays(today, DEFAULT_VALIDITY_DAYS)
}

export function parsePercent(input: string): number | null {
	const normalized = input.replace(',', '.').trim()
	if (!/^\d+(\.\d+)?$/.test(normalized)) return null
	const value = Number(normalized)
	return value >= 0 && value <= 100 ? value : null
}

export function ownerToSelect(ownerId: string | null): string {
	return ownerId ?? NO_OWNER
}

export function ownerFromSelect(value: string): string | null {
	return value === NO_OWNER ? null : value
}

export function canChangeClient(
	lines: readonly Pick<
		QuoteLineFormValues,
		'label' | 'unitPrice' | 'catalogItemId'
	>[],
): boolean {
	return !lines.some(
		(line) =>
			line.label.trim() !== '' ||
			line.unitPrice.trim() !== '' ||
			line.catalogItemId !== '',
	)
}

export function isClientLocked(
	hasCustomer: boolean,
	savedLineCount: number,
	lines: Parameters<typeof canChangeClient>[0],
): boolean {
	return hasCustomer && (savedLineCount > 0 || !canChangeClient(lines))
}

export function validityNote(
	status: QuoteStatus,
	validUntil: string | null,
	today: string,
	isExpired = false,
): ValidityNote {
	if (validUntil === null)
		return {
			text: `Valable ${DEFAULT_VALIDITY_DAYS} jours après l’envoi`,
			expired: false,
		}
	return {
		text: `Valable jusqu’au ${formatDocumentDate(validUntil)}`,
		expired: status === 'DRAFT' ? validUntil < today : isExpired,
	}
}
