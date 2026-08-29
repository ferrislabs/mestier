import type { MissingCost, ProjectProfitability } from '#/hooks/use-reporting'

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

/**
 * Labour, machines and expenses. Mirrors
 * `ProjectProfitability::planned_cost_cents` — `null` when any of the three
 * is (#306's redaction nulls all three together, never one alone, but this
 * stays defensive rather than assuming that pairing holds forever).
 */
export function plannedCostCents(project: ProjectProfitability): number | null {
	if (
		project.labour_cost_cents == null ||
		project.equipment_cost_cents == null ||
		project.expenses_cents == null
	) {
		return null
	}

	return (
		project.labour_cost_cents +
		project.equipment_cost_cents +
		project.expenses_cents
	)
}

/**
 * Why a project's figures cannot be trusted, in words, or `null` when they can.
 *
 * One cause, three shapes: an hourly rate nobody set, a salaried person with no
 * monthly amount, or a salary with no contracted hours to spread over. The
 * wording covers all three rather than naming only the hourly case, which is how
 * a salaried person's hour came to read as 0,00 € with nothing said about it.
 *
 * The clocked model also withheld a margin for a pointage left open, its most
 * common cause; there is no clock left to leave open.
 */
export function incompleteReason(project: ProjectProfitability): string | null {
	const missingRates = project.members_without_rate.length
	if (missingRates === 0) return null

	return `${missingRates} personne${missingRates > 1 ? 's' : ''} sans coût horaire renseigné`
}

/**
 * Whether a project's figures rest on complete data — the rule the headline
 * tiles (Devisé, Coût planifié, Marge) all use to decide what to sum.
 *
 * Mirrors the backend's `ProjectProfitability::is_complete`.
 */
export function isCompleteProject(project: ProjectProfitability): boolean {
	return incompleteReason(project) === null
}

/**
 * How much of a project's time is booked twice over, in words, or `null` when
 * none is.
 *
 * Unlike `incompleteReason` this never withholds a figure: the minutes are
 * known, they are simply charged twice because one person is on two tasks at
 * once. That is a calendar to fix, not a number to distrust, so it reads as its
 * own warning rather than joining the incomplete list.
 */
export function overlapNote(project: ProjectProfitability): string | null {
	if (project.overlapping_minutes <= 0) return null

	return `dont ${formatMinutes(project.overlapping_minutes)} comptées deux fois (chevauchement)`
}

/** `null` when there is nothing to declare, or redacted, so the caller can
 * skip a row. */
export function expensesNote(project: ProjectProfitability): string | null {
	if (!project.expenses_cents || project.expenses_cents <= 0) return null

	return `${formatCents(project.expenses_cents)} de frais`
}

/**
 * The margin as a share of what was quoted, or `null` when either is unknown.
 *
 * Guards a zero quote as well as an absent one: dividing by it would produce an
 * infinity that renders as a percentage.
 */
export function marginRate(project: ProjectProfitability): number | null {
	if (project.margin_cents === null || project.margin_cents === undefined) {
		return null
	}
	if (!project.quoted_cents) return null

	return project.margin_cents / project.quoted_cents
}

export function formatMarginRate(project: ProjectProfitability): string {
	const rate = marginRate(project)
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

/**
 * What to go and fill in, in words, for a person whose cost could not be
 * computed.
 *
 * Three gaps fixed in three different places, so the sentence names one rather
 * than listing candidates. The previous wording said "hourly rate or salary
 * missing" and was shown to somebody who had just entered the salary: what was
 * actually missing was the contract, on another screen entirely.
 */
export function missingCostLabel(missing: MissingCost): string {
	switch (missing) {
		case 'HOURLY_RATE':
			return 'Taux horaire non renseigné'
		case 'MONTHLY_COST':
			return 'Salarié, coût mensuel non renseigné'
		case 'CONTRACTED_HOURS':
			return 'Heures contractuelles à renseigner pour répartir le salaire'
	}
}
