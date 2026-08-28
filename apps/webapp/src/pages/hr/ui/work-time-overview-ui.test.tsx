import { screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import type { WorkTimeOverviewRow } from '#/pages/hr/ui/work-time-overview-ui'
import { WorkTimeOverviewUI } from '#/pages/hr/ui/work-time-overview-ui'
import { renderWithRouter } from '#/test/render-with-router'

function row(
	overrides: Partial<WorkTimeOverviewRow> = {},
): WorkTimeOverviewRow {
	return {
		memberId: 'member-1',
		displayName: 'Nova Alix',
		weeklyContractMinutes: 2100,
		nextAbsence: null,
		...overrides,
	}
}

function baseProps() {
	return {
		organizationName: 'Atelier Bois & Co',
		organizationSlug: 'atelier-bois',
		isLoading: false,
		error: null as string | null,
		hrDataRestricted: false,
		rows: [row()],
	}
}

describe('WorkTimeOverviewUI — states', () => {
	it('shows a loading state', async () => {
		await renderWithRouter(
			<WorkTimeOverviewUI {...baseProps()} isLoading={true} />,
		)
		expect(screen.getByText(/Chargement/)).toBeDefined()
	})

	it('shows the error message when there is one', async () => {
		await renderWithRouter(
			<WorkTimeOverviewUI {...baseProps()} error="Impossible de charger" />,
		)
		expect(screen.getByText('Impossible de charger')).toBeDefined()
	})

	it('shows an empty state when there is no row', async () => {
		await renderWithRouter(<WorkTimeOverviewUI {...baseProps()} rows={[]} />)
		expect(screen.getByText('Aucune personne trouvée')).toBeDefined()
	})
})

describe('WorkTimeOverviewUI — weekly contract duration', () => {
	it('shows the formatted duration for a seat with a profile', async () => {
		await renderWithRouter(
			<WorkTimeOverviewUI
				{...baseProps()}
				rows={[row({ weeklyContractMinutes: 2100 })]}
			/>,
		)
		expect(screen.getByText('35h00')).toBeDefined()
	})

	it('marks a seat without an employee profile instead of showing a duration', async () => {
		await renderWithRouter(
			<WorkTimeOverviewUI
				{...baseProps()}
				rows={[row({ weeklyContractMinutes: null })]}
			/>,
		)
		expect(screen.getByText('Sans profil RH')).toBeDefined()
	})
})

describe('WorkTimeOverviewUI — HR data forbidden (#371)', () => {
	it('shows a neutral notice instead of the red error banner', async () => {
		await renderWithRouter(
			<WorkTimeOverviewUI {...baseProps()} hrDataRestricted={true} />,
		)
		expect(
			screen.getByText(/n’avez pas la permission de consulter/),
		).toBeDefined()
	})

	it('says "Non consultable", never "Sans profil RH", when HR data is restricted', async () => {
		await renderWithRouter(
			<WorkTimeOverviewUI
				{...baseProps()}
				hrDataRestricted={true}
				rows={[row({ weeklyContractMinutes: null })]}
			/>,
		)
		expect(screen.getByText('Non consultable')).toBeDefined()
		expect(screen.queryByText('Sans profil RH')).toBeNull()
	})
})

describe('WorkTimeOverviewUI — next absence', () => {
	it('shows the date and kind of the next absence', async () => {
		await renderWithRouter(
			<WorkTimeOverviewUI
				{...baseProps()}
				rows={[row({ nextAbsence: { date: '2026-08-20', kind: 'LEAVE' } })]}
			/>,
		)
		expect(screen.getByText('20/08/2026')).toBeDefined()
		expect(screen.getByText('Congé')).toBeDefined()
	})

	it('shows a dash when there is no upcoming absence', async () => {
		await renderWithRouter(
			<WorkTimeOverviewUI
				{...baseProps()}
				rows={[row({ nextAbsence: null })]}
			/>,
		)
		expect(screen.getByText('—')).toBeDefined()
	})
})

describe('WorkTimeOverviewUI — link to the detailed page', () => {
	it('links each row to that member’s existing work-time detail page', async () => {
		await renderWithRouter(
			<WorkTimeOverviewUI
				{...baseProps()}
				organizationSlug="atelier-bois"
				rows={[row({ memberId: 'member-42' })]}
			/>,
		)

		const link = screen.getByRole('link', { name: /Détails/ })
		expect(link.getAttribute('href')).toBe(
			'/o/atelier-bois/hr/team/member-42/work-time',
		)
	})
})
