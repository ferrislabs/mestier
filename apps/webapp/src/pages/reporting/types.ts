import type { JobProfitability } from '#/hooks/use-reporting'

/** `3 h 45`, the way a foreman reads a timesheet. */
export function formatMinutes(minutes: number): string {
	const hours = Math.floor(minutes / 60)
	const rest = minutes % 60

	return hours > 0
		? `${hours} h ${String(rest).padStart(2, '0')}`
		: `${rest} min`
}

export function formatCents(cents: number): string {
	return new Intl.NumberFormat('fr-FR', {
		style: 'currency',
		currency: 'EUR',
	}).format(cents / 100)
}

export function realCostCents(job: JobProfitability): number {
	return job.labour_cost_cents + job.equipment_cost_cents
}

/**
 * Why a job's figures cannot be trusted, in words, or `null` when they can.
 *
 * The API says what is missing; this turns it into the sentence a foreman needs
 * in order to fix it. A margin is withheld rather than approximated, so the
 * screen has to explain the hole instead of showing a number with an asterisk.
 */
export function incompleteReason(job: JobProfitability): string | null {
	const missingRates = job.employees_without_rate.length
	const open = job.open_entries

	if (missingRates > 0 && open > 0) {
		return `${missingRates} salarié${missingRates > 1 ? 's' : ''} sans taux horaire, et ${open} pointage${open > 1 ? 's' : ''} non clôturé${open > 1 ? 's' : ''}`
	}
	if (missingRates > 0) {
		return `${missingRates} salarié${missingRates > 1 ? 's' : ''} sans taux horaire renseigné`
	}
	if (open > 0) {
		return `${open} pointage${open > 1 ? 's' : ''} jamais clôturé${open > 1 ? 's' : ''}`
	}

	return null
}

/**
 * The margin as a share of what was quoted, or `null` when either is unknown.
 *
 * Guards a zero quote as well as an absent one: dividing by it would produce an
 * infinity that renders as a percentage.
 */
export function marginRate(job: JobProfitability): number | null {
	if (job.margin_cents === null || job.margin_cents === undefined) return null
	if (!job.quoted_cents) return null

	return job.margin_cents / job.quoted_cents
}

export function formatMarginRate(job: JobProfitability): string {
	const rate = marginRate(job)
	if (rate === null) return '—'

	return new Intl.NumberFormat('fr-FR', {
		style: 'percent',
		maximumFractionDigits: 0,
	}).format(rate)
}

/** The first day of the current month, and today, as the API wants them. */
export function currentMonthPeriod(today: Date): { from: string; to: string } {
	const first = new Date(today.getFullYear(), today.getMonth(), 1)

	return { from: isoDate(first), to: isoDate(today) }
}

export function isoDate(date: Date): string {
	return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`
}
