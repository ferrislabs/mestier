import { describe, expect, it } from 'vitest'
import type {
	MaintenanceRecapGroup,
	MaintenanceRecapLine,
} from '#/hooks/use-maintenance-recap'
import {
	dueBadge,
	formatShortDate,
	hoursLabel,
	lineIdsOf,
	lineSiteLabel,
	pruneSelection,
	townLabel,
} from '#/pages/maintenance/lib/recap-lines'

function line(
	id: string,
	overrides: Partial<MaintenanceRecapLine> = {},
): MaintenanceRecapLine {
	return {
		contract_id: 'contract-1',
		due: { on: '2026-10-19', status: 'THIS_WEEK' },
		label: 'Tonte',
		line_id: id,
		site: { customer_name: 'Gaume', site_label: 'Siège' },
		...overrides,
	}
}

describe('lineIdsOf', () => {
	it('lists every line of every group, in order', () => {
		const groups: MaintenanceRecapGroup[] = [
			{ town: 'Vichy', lines: [line('a'), line('b')] },
			{ town: null, lines: [line('c')] },
		]

		expect(lineIdsOf(groups)).toEqual(['a', 'b', 'c'])
	})
})

describe('pruneSelection', () => {
	it('drops the lines that left the recap and keeps the others', () => {
		const groups = [{ town: 'Vichy', lines: [line('c')] }]

		const kept = pruneSelection(new Set(['a', 'b', 'c']), groups)

		expect([...kept]).toEqual(['c'])
	})

	it('returns the same set when nothing left, so state does not churn', () => {
		const selection = new Set(['c'])

		expect(
			pruneSelection(selection, [{ town: null, lines: [line('c')] }]),
		).toBe(selection)
	})
})

describe('townLabel', () => {
	it('names the group without a town instead of leaving it blank', () => {
		expect(townLabel('Vichy')).toBe('Vichy')
		expect(townLabel(null)).toBe('Sans commune')
		expect(townLabel(undefined)).toBe('Sans commune')
	})
})

describe('dueBadge', () => {
	it('flags overdue lines in the error tone', () => {
		expect(dueBadge('OVERDUE')).toEqual({ label: 'En retard', tone: 'error' })
	})

	it('shows a line due in the week as such', () => {
		expect(dueBadge('THIS_WEEK').label).toBe('Dans la semaine')
	})
})

describe('hoursLabel', () => {
	it('reads minutes as hours', () => {
		expect(hoursLabel(90)).toBe('1 h 30')
		expect(hoursLabel(120)).toBe('2 h')
		expect(hoursLabel(45)).toBe('45 min')
	})

	it('shows a dash for a line with no planned time', () => {
		expect(hoursLabel(null)).toBe('—')
		expect(hoursLabel(undefined)).toBe('—')
	})
})

describe('formatShortDate', () => {
	it('writes a day as a person would on the paper sheet', () => {
		expect(formatShortDate('2026-10-19')).toBe('19 oct.')
	})

	it('shows a dash when there is no date', () => {
		expect(formatShortDate(null)).toBe('—')
	})
})

describe('lineSiteLabel', () => {
	it('joins the customer and the site, with the town when known', () => {
		expect(
			lineSiteLabel({
				customer_name: 'Gaume',
				site_label: 'Siège',
				city: 'Vichy',
			}),
		).toBe('Gaume · Siège (Vichy)')
		expect(lineSiteLabel({ customer_name: 'Gaume', site_label: 'Siège' })).toBe(
			'Gaume · Siège',
		)
	})
})
