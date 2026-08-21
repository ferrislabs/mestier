import { screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import type {
	MemberProfitability,
	ProjectProfitability,
} from '#/hooks/use-reporting'
import { ProfitabilityUI } from '#/pages/reporting/ui/profitability-ui'
import { renderWithRouter } from '#/test/render-with-router'

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

function member(
	overrides: Partial<MemberProfitability> = {},
): MemberProfitability {
	return {
		member_id: 'member-1',
		planned_minutes: 180,
		labour_cost_cents: 10_500,
		rate_missing: false,
		...overrides,
	}
}

function baseProps() {
	return {
		period: { from: '2026-08-01', to: '2026-08-20' },
		organizationSlug: 'atelier-vert',
		projects: [project()],
		mostProfitable: [project()],
		leastProfitable: [project()],
		incomplete: [],
		doubleBooked: [],
		members: [member()],
		memberName: () => 'Martin Alix',
		isLoading: false,
		error: null,
		onPeriodChange: vi.fn(),
		onRetry: vi.fn(),
	}
}

describe('ProfitabilityUI', () => {
	it('shows a project with its cost, margin and time', async () => {
		await renderWithRouter(<ProfitabilityUI {...baseProps()} />)

		// The same project legitimately appears in the list and in both rankings,
		// which is what the API returns, so this counts rather than expecting one.
		expect(screen.getAllByText('Jardin Duval').length).toBeGreaterThan(0)
		expect(screen.getAllByText(/3 h 00/).length).toBeGreaterThan(0)
	})

	/**
	 * The point of the whole screen: the API withholds a margin it cannot state,
	 * and this has to explain the hole rather than print a dash and move on.
	 */
	it('explains why a project has no margin', async () => {
		const incomplete = project({
			margin_cents: null,
			members_without_rate: ['member-9'],
		})
		await renderWithRouter(
			<ProfitabilityUI
				{...baseProps()}
				projects={[incomplete]}
				incomplete={[incomplete]}
			/>,
		)

		expect(screen.getByText(/1 projet sans marge calculable/i)).toBeDefined()
		expect(
			screen.getAllByText(/sans coût horaire renseigné/i).length,
		).toBeGreaterThan(0)
	})

	it('says nothing about incompleteness when everything is known', async () => {
		await renderWithRouter(<ProfitabilityUI {...baseProps()} />)

		expect(screen.queryByText(/sans marge calculable/i)).toBeNull()
	})

	/**
	 * The headline change: work that bills nobody used to be invisible, because
	 * the report only knew chantiers with a customer. A meeting now has a row,
	 * a cost, and a badge saying why it has no margin.
	 */
	it('marks an internal project and still shows its cost', async () => {
		const meeting = project({
			project_id: 'project-internal',
			name: 'Réunion hebdo',
			customer_id: null,
			quoted_cents: null,
			margin_cents: null,
			labour_cost_cents: 14_000,
			equipment_cost_cents: 0,
			planned_minutes: 240,
		})

		await renderWithRouter(
			<ProfitabilityUI
				{...baseProps()}
				projects={[meeting]}
				mostProfitable={[]}
				leastProfitable={[]}
			/>,
		)

		expect(screen.getByText('Interne')).toBeDefined()
		expect(screen.getAllByText('140,00 €').length).toBeGreaterThan(0)
		expect(screen.queryByText(/sans marge calculable/i)).toBeNull()
	})

	/**
	 * An overlap is not incomplete data — the minutes are known, they are just
	 * charged twice. It gets its own warning, and the project keeps its margin.
	 */
	it('warns about double-booked time without withholding the margin', async () => {
		const overlapping = project({ overlapping_minutes: 60 })

		await renderWithRouter(
			<ProfitabilityUI
				{...baseProps()}
				projects={[overlapping]}
				doubleBooked={[overlapping]}
			/>,
		)

		expect(
			screen.getByText(/1 projet avec du temps compté deux fois/i),
		).toBeDefined()
		expect(screen.getAllByText(/comptées deux fois/i).length).toBeGreaterThan(0)
		expect(screen.queryByText(/sans marge calculable/i)).toBeNull()
	})

	it('shows expenses as their own figure', async () => {
		await renderWithRouter(
			<ProfitabilityUI
				{...baseProps()}
				projects={[project({ expenses_cents: 4_500 })]}
				mostProfitable={[]}
				leastProfitable={[]}
			/>,
		)

		expect(screen.getAllByText('45,00 €').length).toBeGreaterThan(0)
	})

	/**
	 * Devisé, Coût planifié and Marge share one inclusion rule, so the three
	 * headline figures cannot disagree about what counts.
	 */
	it('excludes an incomplete project from every headline tile', async () => {
		const complete = project({
			project_id: 'project-complete',
			quoted_cents: 100_000,
			labour_cost_cents: 10_000,
			equipment_cost_cents: 0,
			margin_cents: 90_000,
		})
		const incomplete = project({
			project_id: 'project-incomplete',
			quoted_cents: 500_000,
			labour_cost_cents: 99_900,
			equipment_cost_cents: 0,
			margin_cents: null,
			members_without_rate: ['member-9'],
		})

		await renderWithRouter(
			<ProfitabilityUI
				{...baseProps()}
				projects={[complete, incomplete]}
				mostProfitable={[complete]}
				leastProfitable={[complete]}
				incomplete={[incomplete]}
			/>,
		)

		expect(screen.getAllByText('1 000,00 €').length).toBeGreaterThan(0)
		expect(screen.getAllByText('100,00 €').length).toBeGreaterThan(0)
		expect(screen.queryByText('6 000,00 €')).toBeNull()
		expect(
			screen.getAllByText('Projets complets uniquement').length,
		).toBeGreaterThanOrEqual(2)
	})

	it('names the person rather than showing an id', async () => {
		await renderWithRouter(<ProfitabilityUI {...baseProps()} />)

		expect(screen.getAllByText('Martin Alix').length).toBeGreaterThan(0)
		expect(screen.queryByText('member-1')).toBeNull()
	})

	it("shows each person's cost and a precise missing-rate warning", async () => {
		await renderWithRouter(
			<ProfitabilityUI
				{...baseProps()}
				members={[
					member({ labour_cost_cents: 12_345, rate_missing: false }),
					member({
						member_id: 'member-2',
						labour_cost_cents: 0,
						rate_missing: true,
					}),
				]}
				memberName={(id) =>
					id === 'member-1' ? 'Martin Alix' : 'Chloé Renard'
				}
			/>,
		)

		expect(screen.getByText('123,45 €')).toBeDefined()
		expect(screen.getByText(/coût horaire manquant/i)).toBeDefined()
	})

	/** The hours are the plan's now, and a payroll screen has to say so. */
	it('labels the hours as planned rather than clocked', async () => {
		await renderWithRouter(<ProfitabilityUI {...baseProps()} />)

		expect(screen.getByText('Heures planifiées')).toBeDefined()
		expect(
			screen.getByText(/heures du planning, pas des pointages/i),
		).toBeDefined()
	})

	it('says so plainly when nothing was planned in the period', async () => {
		await renderWithRouter(
			<ProfitabilityUI
				{...baseProps()}
				projects={[]}
				mostProfitable={[]}
				leastProfitable={[]}
				members={[]}
			/>,
		)

		expect(
			screen.getByText(/rien de planifié sur cette période/i),
		).toBeDefined()
	})

	/**
	 * A project is an addressable thing now, so the title links at its own page
	 * instead of dumping the reader on the planning task list.
	 */
	it('links a project name at its own page, in the list, both rankings and the banner', async () => {
		const incomplete = project({
			project_id: 'project-incomplete',
			name: 'Terrasse Bernard',
			margin_cents: null,
			members_without_rate: ['member-9'],
		})

		await renderWithRouter(
			<ProfitabilityUI
				{...baseProps()}
				projects={[project(), incomplete]}
				incomplete={[incomplete]}
			/>,
		)

		const jardinLinks = screen.getAllByRole('link', { name: /Jardin Duval/i })
		expect(jardinLinks.length).toBeGreaterThan(0)
		for (const link of jardinLinks) {
			expect(link.getAttribute('href')).toContain(
				'/o/atelier-vert/planning/projects',
			)
			expect(link.getAttribute('href')).toContain('projectId=project-1')
		}

		const terrasseLinks = screen.getAllByRole('link', {
			name: /Terrasse Bernard/i,
		})
		expect(terrasseLinks.length).toBeGreaterThan(0)
	})
})
