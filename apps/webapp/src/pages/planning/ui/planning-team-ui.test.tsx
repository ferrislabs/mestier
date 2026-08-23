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
				resource_id: 'member:member-1',
				member_id: 'member-1',
				employee_id: 'employee-1',
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
		organizationSlug: 'atelier-bois',
		pendingReportsCount: null,
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

describe('PlanningTeamUI — states', () => {
	it('shows a loading state without crashing', () => {
		render(<PlanningTeamUI {...baseProps({ isLoading: true, data: null })} />)

		expect(screen.getByTestId('planning-loading')).toBeDefined()
	})

	it('shows the error message without exposing the raw error elsewhere', () => {
		render(
			<PlanningTeamUI
				{...baseProps({ error: 'Fenêtre supérieure à 92 jours', data: null })}
			/>,
		)

		expect(screen.getByTestId('planning-error')).toBeDefined()
		expect(screen.getByText('Fenêtre supérieure à 92 jours')).toBeDefined()
	})

	it('renders the grid once the data is loaded', () => {
		render(<PlanningTeamUI {...baseProps()} />)

		expect(screen.getByTestId('planning-grid')).toBeDefined()
		expect(screen.getByText('Alix Martin')).toBeDefined()
	})
})

describe('PlanningTeamUI — barre d’outils', () => {
	it('shows the navigation bar with the active view', () => {
		render(<PlanningTeamUI {...baseProps()} />)

		expect(
			screen.getByRole('tab', { name: 'Semaine' }).getAttribute('data-state'),
		).toBe('active')
	})
})

describe('PlanningTeamUI — editing', () => {
	it('no longer exposes a button to add an absence — handled from the HR module', () => {
		render(<PlanningTeamUI {...baseProps()} />)
		expect(
			screen.queryByRole('button', { name: /Ajouter une absence/ }),
		).toBeNull()
	})

	it('no longer exposes an absence editing sheet, even with an absence entry in the grid', () => {
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
								member_id: 'member-1',
							},
						],
					},
				})}
			/>,
		)

		fireEvent.click(screen.getByTestId('grid-segment'))
		expect(screen.queryByRole('dialog')).toBeNull()
	})

	it('shows the warning dialog when warningDialog.open is true', () => {
		render(
			<PlanningTeamUI
				{...baseProps({
					warningDialog: {
						open: true,
						warnings: [
							{
								kind: 'overlapping_task',
								taskId: 'wo-1',
								startsAt: '2026-08-03T08:00:00+02:00',
								endsAt: '2026-08-03T10:00:00+02:00',
							},
						],
						isPending: false,
						onConfirm: vi.fn(),
						onCancel: vi.fn(),
					},
				})}
			/>,
		)

		expect(screen.getByRole('alertdialog')).toBeDefined()
		expect(screen.getByText(/Déjà affecté à un autre projet/)).toBeDefined()
	})
})

describe('PlanningTeamUI — new task', () => {
	it('shows a « Nouvelle tâche » button and reports it through onCreateTask', () => {
		const onCreateTask = vi.fn()
		render(<PlanningTeamUI {...baseProps({ onCreateTask })} />)

		fireEvent.click(screen.getByRole('button', { name: /Nouvelle tâche/ }))
		expect(onCreateTask).toHaveBeenCalledTimes(1)
	})

	it('does not show the button without onCreateTask', () => {
		render(<PlanningTeamUI {...baseProps()} />)
		expect(screen.queryByRole('button', { name: /Nouvelle tâche/ })).toBeNull()
	})

	it('reports a click on a task segment through onOpenTask', () => {
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
								member_ids: ['member-1'],
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

	it('renders the taskSheet slot when provided', () => {
		render(
			<PlanningTeamUI
				{...baseProps({ taskSheet: <div data-testid="fake-task-sheet" /> })}
			/>,
		)
		expect(screen.getByTestId('fake-task-sheet')).toBeDefined()
	})
})

describe('PlanningTeamUI — no network call', () => {
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

	it('fires no fetch on render', () => {
		render(<PlanningTeamUI {...baseProps()} />)

		expect(fetchSpy).not.toHaveBeenCalled()
	})
})
