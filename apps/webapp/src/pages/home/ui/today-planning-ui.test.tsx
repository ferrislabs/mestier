import { screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { TodayPlanningUI } from '#/pages/home/ui/today-planning-ui'
import { renderWithRouter } from '#/test/render-with-router'

describe('TodayPlanningUI', () => {
	it('lists each entry with its time window and subtitle', async () => {
		await renderWithRouter(
			<TodayPlanningUI
				organizationSlug="atelier-bois"
				isLoading={false}
				error={null}
				entries={[
					{
						id: 't1',
						timeWindow: '08:00–11:30',
						title: 'Taille de haies — Résidence Les Tilleuls',
						subtitle: 'Karim Belhadj',
					},
				]}
			/>,
		)

		expect(screen.getByText('08:00–11:30')).toBeDefined()
		expect(
			screen.getByText('Taille de haies — Résidence Les Tilleuls'),
		).toBeDefined()
		expect(screen.getByText('Karim Belhadj')).toBeDefined()
	})

	it('says outright when nothing is planned today, rather than an empty list', async () => {
		await renderWithRouter(
			<TodayPlanningUI
				organizationSlug="atelier-bois"
				isLoading={false}
				error={null}
				entries={[]}
			/>,
		)

		expect(screen.getByText('Rien de planifié aujourd’hui.')).toBeDefined()
	})

	it('surfaces the error message instead of the list when the read failed', async () => {
		await renderWithRouter(
			<TodayPlanningUI
				organizationSlug="atelier-bois"
				isLoading={false}
				error="Impossible de charger le planning"
				entries={[]}
			/>,
		)

		expect(screen.getByText('Impossible de charger le planning')).toBeDefined()
	})

	it('links through to the calendar', async () => {
		await renderWithRouter(
			<TodayPlanningUI
				organizationSlug="atelier-bois"
				isLoading={false}
				error={null}
				entries={[]}
			/>,
		)

		const link = screen.getByRole('link', { name: /Voir le planning/ })
		expect(link.getAttribute('href')).toBe('/o/atelier-bois/planning/calendar')
	})
})
