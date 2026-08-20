import { screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import type { JobProfitability, WorkedHoursRow } from '#/hooks/use-reporting'
import { ProfitabilityUI } from '#/pages/reporting/ui/profitability-ui'
import { renderWithRouter } from '#/test/render-with-router'

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

function hours(overrides: Partial<WorkedHoursRow> = {}): WorkedHoursRow {
	return {
		employee_id: 'employee-1',
		worked_minutes: 180,
		open_entries: 0,
		...overrides,
	}
}

function baseProps() {
	return {
		period: { from: '2026-08-01', to: '2026-08-20' },
		jobs: [job()],
		mostProfitable: [job()],
		leastProfitable: [job()],
		incomplete: [],
		workedHours: [hours()],
		totalWorkedMinutes: 180,
		employeeName: () => 'Martin Alix',
		isLoading: false,
		error: null,
		onPeriodChange: vi.fn(),
		onRetry: vi.fn(),
	}
}

describe('ProfitabilityUI', () => {
	it('shows a job with its cost, margin and time', async () => {
		await renderWithRouter(<ProfitabilityUI {...baseProps()} />)

		// The same job legitimately appears in the list and in both rankings,
		// which is what the API returns, so this counts rather than expecting one.
		expect(screen.getAllByText('Jardin Duval').length).toBeGreaterThan(0)
		expect(screen.getAllByText(/3 h 00/).length).toBeGreaterThan(0)
	})

	/**
	 * The point of the whole screen: the API withholds a margin it cannot state,
	 * and this has to explain the hole rather than print a dash and move on.
	 */
	it('explains why a job has no margin', async () => {
		const incomplete = job({
			margin_cents: null,
			employees_without_rate: ['employee-9'],
		})
		await renderWithRouter(
			<ProfitabilityUI
				{...baseProps()}
				jobs={[incomplete]}
				incomplete={[incomplete]}
			/>,
		)

		expect(screen.getByText(/1 chantier sans marge calculable/i)).toBeDefined()
		expect(
			screen.getAllByText(/sans taux horaire renseigné/i).length,
		).toBeGreaterThan(0)
	})

	it('says nothing about incompleteness when everything is known', async () => {
		await renderWithRouter(<ProfitabilityUI {...baseProps()} />)

		expect(screen.queryByText(/sans marge calculable/i)).toBeNull()
	})

	/**
	 * Recollected time is informational, never a reason the margin is missing:
	 * the job in `baseProps` has a stated margin and still shows the note.
	 */
	it('notes recollected time without treating the job as incomplete', async () => {
		await renderWithRouter(
			<ProfitabilityUI
				{...baseProps()}
				jobs={[job({ recollected_minutes: 45 })]}
			/>,
		)

		expect(screen.getByText(/45 min déclarées a posteriori/i)).toBeDefined()
		expect(screen.queryByText(/sans marge calculable/i)).toBeNull()
	})

	it('warns that a payroll total is short when a clock-in is open', async () => {
		await renderWithRouter(
			<ProfitabilityUI
				{...baseProps()}
				workedHours={[hours({ open_entries: 2 })]}
			/>,
		)

		expect(screen.getByText(/ce total est incomplet/i)).toBeDefined()
	})

	it('names the employee rather than showing an id', async () => {
		await renderWithRouter(<ProfitabilityUI {...baseProps()} />)

		expect(screen.getAllByText('Martin Alix').length).toBeGreaterThan(0)
		expect(screen.queryByText('employee-1')).toBeNull()
	})

	it('says so plainly when no time was clocked in the period', async () => {
		await renderWithRouter(
			<ProfitabilityUI
				{...baseProps()}
				jobs={[]}
				mostProfitable={[]}
				leastProfitable={[]}
				workedHours={[]}
				totalWorkedMinutes={0}
			/>,
		)

		expect(
			screen.getByText(/aucun temps pointé sur cette période/i),
		).toBeDefined()
	})
})
