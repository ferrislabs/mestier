import { describe, expect, it } from 'vitest'
import type { ProjectProfitability } from '#/hooks/use-reporting'
import {
	currentMonthPeriod,
	expensesNote,
	formatMarginRate,
	formatMinutes,
	incompleteReason,
	isCompleteProject,
	marginRate,
	overlapNote,
	plannedCostCents,
} from '#/pages/reporting/types'

function project(
	overrides: Partial<ProjectProfitability> = {},
): ProjectProfitability {
	return {
		project_id: 'project-1',
		name: 'Jardin Duval',
		customer_id: 'customer-1',
		quoted_cents: 420_000,
		labour_cost_cents: 10_500,
		equipment_cost_cents: 3_600,
		expenses_cents: 0,
		planned_minutes: 180,
		occupied_minutes: 180,
		overlapping_minutes: 0,
		margin_cents: 405_900,
		members_without_rate: [],
		...overrides,
	}
}

describe('formatMinutes', () => {
	it('reads like a timesheet', () => {
		expect(formatMinutes(180)).toBe('3 h 00')
		expect(formatMinutes(225)).toBe('3 h 45')
		expect(formatMinutes(30)).toBe('30 min')
	})
})

describe('plannedCostCents', () => {
	it('is labour plus machines plus expenses', () => {
		expect(plannedCostCents(project({ expenses_cents: 4_500 }))).toBe(
			10_500 + 3_600 + 4_500,
		)
	})
})

describe('incompleteReason', () => {
	it('says nothing when nothing is missing', () => {
		expect(incompleteReason(project())).toBeNull()
		expect(isCompleteProject(project())).toBe(true)
	})

	it('names a missing rate so it can be fixed', () => {
		const reason = incompleteReason(
			project({ members_without_rate: ['member-1'], margin_cents: null }),
		)

		expect(reason).toBe('1 personne sans taux horaire renseigné')
	})

	it('pluralises several missing rates', () => {
		const reason = incompleteReason(
			project({ members_without_rate: ['member-1', 'member-2'] }),
		)

		expect(reason).toBe('2 personnes sans taux horaire renseigné')
	})

	/// An overlap is a calendar to fix, not a figure to distrust.
	it('ignores an overlap entirely', () => {
		const overlapping = project({ overlapping_minutes: 60 })

		expect(incompleteReason(overlapping)).toBeNull()
		expect(isCompleteProject(overlapping)).toBe(true)
	})
})

describe('overlapNote', () => {
	it('says nothing when nobody is double booked', () => {
		expect(overlapNote(project())).toBeNull()
	})

	it('says how much time is charged twice', () => {
		expect(overlapNote(project({ overlapping_minutes: 90 }))).toBe(
			'dont 1 h 30 comptées deux fois (chevauchement)',
		)
	})
})

describe('expensesNote', () => {
	it('says nothing when there is nothing to declare', () => {
		expect(expensesNote(project())).toBeNull()
	})

	it('reads as money', () => {
		expect(expensesNote(project({ expenses_cents: 4_500 }))).toContain('45,00')
	})
})

describe('marginRate', () => {
	it('is the margin over the quote', () => {
		expect(
			marginRate(project({ quoted_cents: 1_000, margin_cents: 250 })),
		).toBe(0.25)
	})

	it('withholds a rate with no margin', () => {
		expect(marginRate(project({ margin_cents: null }))).toBeNull()
		expect(formatMarginRate(project({ margin_cents: null }))).toBe('—')
	})

	/// Dividing by a zero quote would render as an infinity percentage.
	it('withholds a rate on a quote of zero', () => {
		expect(
			marginRate(project({ quoted_cents: 0, margin_cents: 100 })),
		).toBeNull()
	})

	it('withholds a rate on an internal project', () => {
		expect(
			marginRate(
				project({ customer_id: null, quoted_cents: null, margin_cents: null }),
			),
		).toBeNull()
	})
})

describe('currentMonthPeriod', () => {
	it('runs from the first of the month to today', () => {
		expect(currentMonthPeriod(new Date(2026, 7, 21))).toEqual({
			from: '2026-08-01',
			to: '2026-08-21',
		})
	})
})
