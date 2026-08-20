import { describe, expect, it } from 'vitest'
import type { JobProfitability } from '#/hooks/use-reporting'
import {
	currentMonthPeriod,
	formatMarginRate,
	formatMinutes,
	incompleteReason,
	isCompleteJob,
	marginRate,
	realCostCents,
	recollectedNote,
} from '#/pages/reporting/types'

function job(overrides: Partial<JobProfitability> = {}): JobProfitability {
	return {
		task_id: 'task-1',
		title: 'Jardin Duval',
		customer_id: 'customer-1',
		quoted_cents: 420_000,
		labour_cost_cents: 10_500,
		equipment_cost_cents: 3_600,
		worked_minutes: 180,
		occupied_minutes: 180,
		margin_cents: 405_900,
		employees_without_rate: [],
		open_entries: 0,
		recollected_minutes: 0,
		...overrides,
	}
}

describe('formatMinutes', () => {
	it('reads like a timesheet', () => {
		expect(formatMinutes(180)).toBe('3 h 00')
		expect(formatMinutes(225)).toBe('3 h 45')
		expect(formatMinutes(45)).toBe('45 min')
		expect(formatMinutes(0)).toBe('0 min')
	})
})

describe('realCostCents', () => {
	it('is labour plus equipment', () => {
		expect(realCostCents(job())).toBe(14_100)
	})
})

describe('incompleteReason', () => {
	it('says nothing when nothing is missing', () => {
		expect(incompleteReason(job())).toBeNull()
	})

	/**
	 * The API withholds the margin when data is missing, so this screen owes the
	 * reader the reason. A number with an asterisk would be worse than none.
	 */
	it('names a missing rate so it can be fixed', () => {
		expect(
			incompleteReason(
				job({ employees_without_rate: ['e1'], margin_cents: null }),
			),
		).toBe('1 salarié sans taux horaire renseigné')
		expect(
			incompleteReason(
				job({ employees_without_rate: ['e1', 'e2'], margin_cents: null }),
			),
		).toBe('2 salariés sans taux horaire renseigné')
	})

	it('names an unfinished clock-in', () => {
		expect(incompleteReason(job({ open_entries: 1, margin_cents: null }))).toBe(
			'1 pointage jamais clôturé',
		)
		expect(incompleteReason(job({ open_entries: 3, margin_cents: null }))).toBe(
			'3 pointages jamais clôturés',
		)
	})

	it('names both when both are missing', () => {
		expect(
			incompleteReason(
				job({
					employees_without_rate: ['e1'],
					open_entries: 2,
					margin_cents: null,
				}),
			),
		).toBe('1 salarié sans taux horaire, et 2 pointages non clôturés')
	})
})

describe('isCompleteJob', () => {
	/**
	 * The rule the three headline tiles (Devisé, Coût réel, Marge) all share:
	 * this is exactly `incompleteReason(job) === null`, kept as a named check
	 * rather than three inline comparisons.
	 */
	it('is true exactly when incompleteReason is null', () => {
		expect(isCompleteJob(job())).toBe(true)
		expect(
			isCompleteJob(
				job({ employees_without_rate: ['e1'], margin_cents: null }),
			),
		).toBe(false)
		expect(isCompleteJob(job({ open_entries: 1, margin_cents: null }))).toBe(
			false,
		)
	})
})

describe('recollectedNote', () => {
	it('says nothing when nothing was declared after the fact', () => {
		expect(recollectedNote(job())).toBeNull()
	})

	/**
	 * Recollected time still counts fully in the cost and the margin — this note
	 * is informational, never a warning like `incompleteReason`.
	 */
	it('names the recollected time without withholding anything', () => {
		expect(recollectedNote(job({ recollected_minutes: 45 }))).toBe(
			'dont 45 min déclarées a posteriori',
		)
	})
})

describe('marginRate', () => {
	it('is the margin over what was quoted', () => {
		expect(
			marginRate(job({ quoted_cents: 100_000, margin_cents: 25_000 })),
		).toBe(0.25)
		// Whitespace normalised: `Intl` puts a narrow no-break space before the
		// sign, and pinning that character would tie the test to an ICU detail
		// rather than to the number it is checking.
		expect(
			formatMarginRate(
				job({ quoted_cents: 100_000, margin_cents: 25_000 }),
			).replace(/\s/g, ' '),
		).toBe('25 %')
	})

	it('has no rate when the margin was withheld', () => {
		expect(marginRate(job({ margin_cents: null }))).toBeNull()
		expect(formatMarginRate(job({ margin_cents: null }))).toBe('—')
	})

	it('has no rate when there is no quote', () => {
		expect(
			marginRate(job({ quoted_cents: null, margin_cents: null })),
		).toBeNull()
	})

	/** A zero quote would divide into an infinity that renders as a percentage. */
	it('refuses to divide by a quote of zero', () => {
		expect(marginRate(job({ quoted_cents: 0, margin_cents: 0 }))).toBeNull()
	})
})

describe('currentMonthPeriod', () => {
	it('runs from the first of the month to today', () => {
		expect(currentMonthPeriod(new Date(2026, 7, 20))).toEqual({
			from: '2026-08-01',
			to: '2026-08-20',
		})
	})

	it('is a single day on the first of the month', () => {
		expect(currentMonthPeriod(new Date(2026, 7, 1))).toEqual({
			from: '2026-08-01',
			to: '2026-08-01',
		})
	})
})
