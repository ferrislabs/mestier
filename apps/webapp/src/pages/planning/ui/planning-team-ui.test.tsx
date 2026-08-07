import { render, screen } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { PlanningResponse } from '#/pages/planning/types'
import {
	PlanningTeamUI,
	type PlanningTeamUIProps,
} from '#/pages/planning/ui/planning-team-ui'

function planningResponse(
	overrides: Partial<PlanningResponse> = {},
): PlanningResponse {
	return {
		timezone: 'Europe/Paris',
		resources: [
			{
				resource_id: 'employee:employee-1',
				kind: 'employee',
				employee_id: 'employee-1',
				user_id: null,
				display_name: 'Alix Martin',
				hourly_rate_cents: 1500,
				weekly_contract_minutes: 2100,
			},
		],
		entries: [],
		work_time: [],
		...overrides,
	}
}

function baseProps(
	overrides: Partial<PlanningTeamUIProps> = {},
): PlanningTeamUIProps {
	return {
		organizationName: 'Atelier Bois & Co',
		view: 'week',
		date: '2026-08-07',
		windowFrom: '2026-08-03',
		windowTo: '2026-08-09',
		onViewChange: vi.fn(),
		onDateChange: vi.fn(),
		isLoading: false,
		error: null,
		data: planningResponse(),
		...overrides,
	}
}

describe('PlanningTeamUI — états', () => {
	it('affiche un état de chargement sans planter', () => {
		render(<PlanningTeamUI {...baseProps({ isLoading: true, data: null })} />)

		expect(screen.getByTestId('planning-loading')).toBeDefined()
	})

	it("affiche le message d'erreur sans exposer l'erreur brute ailleurs", () => {
		render(
			<PlanningTeamUI
				{...baseProps({ error: 'Fenêtre supérieure à 92 jours', data: null })}
			/>,
		)

		expect(screen.getByTestId('planning-error')).toBeDefined()
		expect(screen.getByText('Fenêtre supérieure à 92 jours')).toBeDefined()
	})

	it('rend la grille quand les données sont chargées', () => {
		render(<PlanningTeamUI {...baseProps()} />)

		expect(screen.getByTestId('planning-grid')).toBeDefined()
		expect(screen.getByText('Alix Martin')).toBeDefined()
	})
})

describe('PlanningTeamUI — barre d’outils', () => {
	it('affiche la barre de navigation avec la vue active', () => {
		render(<PlanningTeamUI {...baseProps()} />)

		expect(
			screen.getByRole('tab', { name: 'Semaine' }).getAttribute('data-state'),
		).toBe('active')
	})
})

describe('PlanningTeamUI — pas d’appel réseau', () => {
	let fetchSpy: ReturnType<typeof createFetchSpy>

	function createFetchSpy() {
		return vi.spyOn(global, 'fetch').mockImplementation(() => {
			throw new Error('le composant ui/ ne doit jamais appeler fetch')
		})
	}

	beforeEach(() => {
		fetchSpy = createFetchSpy()
	})

	afterEach(() => {
		fetchSpy.mockRestore()
	})

	it('ne déclenche aucun fetch au rendu', () => {
		render(<PlanningTeamUI {...baseProps()} />)

		expect(fetchSpy).not.toHaveBeenCalled()
	})
})
