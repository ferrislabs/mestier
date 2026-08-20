import { screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import type {
	EmployeeProfitability,
	JobProfitability,
} from '#/hooks/use-reporting'
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

function employee(
	overrides: Partial<EmployeeProfitability> = {},
): EmployeeProfitability {
	return {
		employee_id: 'employee-1',
		worked_minutes: 180,
		labour_cost_cents: 10_500,
		rate_missing: false,
		open_entries: 0,
		...overrides,
	}
}

function baseProps() {
	return {
		period: { from: '2026-08-01', to: '2026-08-20' },
		organizationSlug: 'atelier-vert',
		jobs: [job()],
		mostProfitable: [job()],
		leastProfitable: [job()],
		incomplete: [],
		employees: [employee()],
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

	/**
	 * Devisé and Coût réel used to sum every job, incomplete ones included,
	 * while Marge already excluded them — three headline figures disagreeing
	 * on what counts. They now share the exact same rule.
	 */
	it('excludes an incomplete job from Devisé and Coût réel, like Marge already excludes it', async () => {
		const complete = job({
			task_id: 'task-complete',
			quoted_cents: 100_000,
			labour_cost_cents: 10_000,
			equipment_cost_cents: 0,
			margin_cents: 90_000,
		})
		const incomplete = job({
			task_id: 'task-incomplete',
			quoted_cents: 500_000,
			labour_cost_cents: 99_900,
			equipment_cost_cents: 0,
			margin_cents: null,
			employees_without_rate: ['employee-9'],
		})

		await renderWithRouter(
			<ProfitabilityUI
				{...baseProps()}
				jobs={[complete, incomplete]}
				mostProfitable={[complete]}
				leastProfitable={[complete]}
				incomplete={[incomplete]}
			/>,
		)

		// Only the complete job's figures reach the totals: 1 000,00 € quoted,
		// 100,00 € cost — the incomplete job's 5 000 € quote and its cost never
		// make it into either tile.
		// Each value also appears in the row's own "Devisé"/"Coût" figure, so
		// this counts occurrences rather than expecting exactly one.
		expect(screen.getAllByText('1 000,00 €').length).toBeGreaterThan(0)
		expect(screen.getAllByText('100,00 €').length).toBeGreaterThan(0)
		expect(screen.queryByText('6 000,00 €')).toBeNull()

		// All three headline tiles now state the same inclusion rule.
		expect(
			screen.getAllByText('Chantiers complets uniquement').length,
		).toBeGreaterThanOrEqual(2)
	})

	it('warns that a payroll total is short when a clock-in is open', async () => {
		await renderWithRouter(
			<ProfitabilityUI
				{...baseProps()}
				employees={[employee({ open_entries: 2 })]}
			/>,
		)

		expect(screen.getByText(/ce total est incomplet/i)).toBeDefined()
	})

	it('names the employee rather than showing an id', async () => {
		await renderWithRouter(<ProfitabilityUI {...baseProps()} />)

		expect(screen.getAllByText('Martin Alix').length).toBeGreaterThan(0)
		expect(screen.queryByText('employee-1')).toBeNull()
	})

	/**
	 * The profitability response already carries per-employee cost and a
	 * precise missing-rate flag — this screen used to fetch hours only and
	 * silently drop both.
	 */
	it("shows each employee's cost and a precise missing-rate warning", async () => {
		await renderWithRouter(
			<ProfitabilityUI
				{...baseProps()}
				employees={[
					employee({ labour_cost_cents: 12_345, rate_missing: false }),
					employee({
						employee_id: 'employee-2',
						labour_cost_cents: 0,
						rate_missing: true,
					}),
				]}
				employeeName={(id) =>
					id === 'employee-1' ? 'Martin Alix' : 'Chloé Renard'
				}
			/>,
		)

		expect(screen.getByText('123,45 €')).toBeDefined()
		expect(screen.getByText(/taux horaire manquant/i)).toBeDefined()
	})

	it('says so plainly when no time was clocked in the period', async () => {
		await renderWithRouter(
			<ProfitabilityUI
				{...baseProps()}
				jobs={[]}
				mostProfitable={[]}
				leastProfitable={[]}
				employees={[]}
			/>,
		)

		expect(
			screen.getByText(/aucun temps pointé sur cette période/i),
		).toBeDefined()
	})

	/**
	 * Job rows used to be plain text: a manager reading "sans taux horaire"
	 * had no way to act on it from this screen. The title is now a link into
	 * Planning's task list — not a deep link to the exact task (nothing in
	 * the app opens one from outside Planning yet), but still somewhere to go
	 * rather than something to memorize.
	 */
	it('links a job title into the Planning task list, in the list, both rankings and the incomplete banner', async () => {
		const incomplete = job({
			task_id: 'task-incomplete',
			title: 'Terrasse Bernard',
			margin_cents: null,
			employees_without_rate: ['employee-9'],
		})

		await renderWithRouter(
			<ProfitabilityUI
				{...baseProps()}
				jobs={[job(), incomplete]}
				incomplete={[incomplete]}
			/>,
		)

		const jardinLinks = screen.getAllByRole('link', { name: /Jardin Duval/i })
		expect(jardinLinks.length).toBeGreaterThan(0)
		for (const link of jardinLinks) {
			expect(link.getAttribute('href')).toBe('/o/atelier-vert/planning/tasks')
		}

		const terrasseLinks = screen.getAllByRole('link', {
			name: /Terrasse Bernard/i,
		})
		expect(terrasseLinks.length).toBeGreaterThan(0)
	})
})
