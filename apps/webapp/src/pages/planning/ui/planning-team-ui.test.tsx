import { fireEvent, render, screen } from '@testing-library/react'
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

describe('PlanningTeamUI — édition', () => {
	it("n'expose plus de bouton pour ajouter une absence — géré depuis le module RH", () => {
		render(<PlanningTeamUI {...baseProps()} />)
		expect(
			screen.queryByRole('button', { name: /Ajouter une absence/ }),
		).toBeNull()
	})

	it("n'expose plus de sheet d'édition d'absence, même avec un entry absence dans la grille", () => {
		render(
			<PlanningTeamUI
				{...baseProps({
					data: {
						...planningResponse(),
						entries: [
							{
								kind: 'absence',
								id: 'ab-1',
								starts_at: '2026-08-03T00:00:00+02:00',
								ends_at: '2026-08-04T00:00:00+02:00',
								all_day: true,
								absence_kind: 'LEAVE',
								employee_id: 'employee-1',
							},
						],
					},
				})}
			/>,
		)

		fireEvent.click(screen.getByTestId('grid-segment'))
		expect(screen.queryByRole('dialog')).toBeNull()
	})

	it('affiche le dialogue d’avertissement quand warningDialog.open est vrai', () => {
		render(
			<PlanningTeamUI
				{...baseProps({
					warningDialog: {
						open: true,
						warnings: [{ kind: 'missing_employee_record' }],
						isPending: false,
						onConfirm: vi.fn(),
						onCancel: vi.fn(),
					},
				})}
			/>,
		)

		expect(screen.getByRole('alertdialog')).toBeDefined()
		expect(screen.getByText(/Aucune fiche employé/)).toBeDefined()
	})

	it('affiche les fiches employé créées et laisse les fermer', () => {
		const onDismiss = vi.fn()
		render(
			<PlanningTeamUI
				{...baseProps({
					createdEmployeeNames: ['Marie Leroy'],
					onDismissCreatedEmployees: onDismiss,
				})}
			/>,
		)

		expect(screen.getByText(/Marie Leroy/)).toBeDefined()
		expect(screen.getByText(/taux horaire/)).toBeDefined()
		fireEvent.click(screen.getByRole('button', { name: /Fermer/ }))
		expect(onDismiss).toHaveBeenCalledTimes(1)
	})
})

describe('PlanningTeamUI — nouvelle tâche', () => {
	it('affiche un bouton « Nouvelle tâche » et le reporte via onCreateTask', () => {
		const onCreateTask = vi.fn()
		render(<PlanningTeamUI {...baseProps({ onCreateTask })} />)

		fireEvent.click(screen.getByRole('button', { name: /Nouvelle tâche/ }))
		expect(onCreateTask).toHaveBeenCalledTimes(1)
	})

	it("n'affiche pas le bouton sans onCreateTask", () => {
		render(<PlanningTeamUI {...baseProps()} />)
		expect(screen.queryByRole('button', { name: /Nouvelle tâche/ })).toBeNull()
	})

	it('reporte le clic sur un segment de tâche via onOpenTask', () => {
		const onOpenTask = vi.fn()
		render(
			<PlanningTeamUI
				{...baseProps({
					onOpenTask,
					data: {
						...planningResponse(),
						entries: [
							{
								kind: 'task',
								labels: [],
								title: 'Tâche',
								blocks_availability: true,
								child_count: 0,
								id: 'task-1',
								starts_at: '2026-08-03T08:00:00+02:00',
								ends_at: '2026-08-03T10:00:00+02:00',
								all_day: false,
								status: 'PLANNED',
								employee_ids: ['employee-1'],
								customer_name: null,
								context_label: null,
							},
						],
					},
				})}
			/>,
		)

		fireEvent.click(screen.getByTestId('grid-segment'))
		expect(onOpenTask).toHaveBeenCalledWith({ entryId: 'task-1' })
	})

	it('rend le slot taskSheet quand fourni', () => {
		render(
			<PlanningTeamUI
				{...baseProps({ taskSheet: <div data-testid="fake-task-sheet" /> })}
			/>,
		)
		expect(screen.getByTestId('fake-task-sheet')).toBeDefined()
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
