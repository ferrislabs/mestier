import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
	TaskListSubtaskRows,
	type TaskListSubtaskRowVM,
} from '#/pages/planning/ui/task-list-subtask-rows'

const SUBTASKS: TaskListSubtaskRowVM[] = [
	{
		id: 'sub-1',
		title: 'Poser les tuiles',
		status: 'PLANNED',
		labels: [{ id: 'l1', name: 'Réunion', color: '#2563EB' }],
		window: {
			startsAt: '2026-08-10T07:00:00.000Z',
			endsAt: '2026-08-10T09:00:00.000Z',
			inherited: false,
		},
		allDay: false,
		assigneeNames: ['Alix Martin'],
	},
	{
		id: 'sub-2',
		title: 'Nettoyer le projet',
		status: 'DONE',
		labels: [],
		window: {
			startsAt: '2026-08-11T07:00:00.000Z',
			endsAt: '2026-08-11T09:00:00.000Z',
			inherited: true,
		},
		allDay: false,
		assigneeNames: [],
	},
]

function renderInTable(children: React.ReactNode) {
	return render(
		<table>
			<tbody>{children}</tbody>
		</table>,
	)
}

function baseProps() {
	return {
		subtasks: SUBTASKS,
		isLoading: false,
		error: null,
		timeZone: 'Europe/Paris',
		onOpenSubtask: vi.fn(),
	}
}

describe('TaskListSubtaskRows — rendu', () => {
	it('renders every subtask title', () => {
		renderInTable(<TaskListSubtaskRows {...baseProps()} />)
		expect(screen.getByText('Poser les tuiles')).toBeDefined()
		expect(screen.getByText('Nettoyer le projet')).toBeDefined()
	})

	it("flags a subtask's inherited window", () => {
		renderInTable(<TaskListSubtaskRows {...baseProps()} />)
		const inheriting = screen.getByTestId('subtask-row-sub-2')
		expect(inheriting.textContent).toMatch(/hérité/i)

		const ownDates = screen.getByTestId('subtask-row-sub-1')
		expect(ownDates.textContent).not.toMatch(/hérité/i)
	})

	it('formats an unassigned subtask distinctly from a blank cell', () => {
		renderInTable(<TaskListSubtaskRows {...baseProps()} />)
		expect(screen.getByText('Personne assigné')).toBeDefined()
	})

	it('shows a loading state', () => {
		renderInTable(
			<TaskListSubtaskRows {...baseProps()} subtasks={[]} isLoading={true} />,
		)
		expect(screen.getByText(/Chargement/)).toBeDefined()
	})

	it('shows an empty state', () => {
		renderInTable(<TaskListSubtaskRows {...baseProps()} subtasks={[]} />)
		expect(screen.getByText(/Aucune sous-tâche/)).toBeDefined()
	})

	it('shows an error state', () => {
		renderInTable(
			<TaskListSubtaskRows {...baseProps()} subtasks={[]} error="Échec" />,
		)
		expect(screen.getByText('Échec')).toBeDefined()
	})
})

describe('TaskListSubtaskRows — full day', () => {
	it('renders an all-day subtask window as its single date, not a midnight-to-midnight range', () => {
		const allDaySubtask: TaskListSubtaskRowVM = {
			id: 'sub-3',
			title: 'Livraison matériaux',
			status: 'PLANNED',
			labels: [],
			window: {
				startsAt: '2026-08-09T22:00:00.000Z',
				endsAt: '2026-08-10T22:00:00.000Z',
				inherited: false,
			},
			allDay: true,
			assigneeNames: [],
		}
		renderInTable(
			<TaskListSubtaskRows {...baseProps()} subtasks={[allDaySubtask]} />,
		)

		const row = screen.getByTestId('subtask-row-sub-3')
		expect(row.textContent).toContain('10/08/2026')
		expect(row.textContent).not.toContain('00:00')
	})
})

describe('TaskListSubtaskRows — interactions', () => {
	it('calls onOpenSubtask with the id when a row is clicked', async () => {
		const user = userEvent.setup()
		const onOpenSubtask = vi.fn()
		renderInTable(
			<TaskListSubtaskRows {...baseProps()} onOpenSubtask={onOpenSubtask} />,
		)

		await user.click(screen.getByTestId('subtask-row-sub-1'))
		expect(onOpenSubtask).toHaveBeenCalledWith('sub-1')
	})
})

describe('TaskListSubtaskRows — no network call', () => {
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

	it('never triggers fetch', () => {
		renderInTable(<TaskListSubtaskRows {...baseProps()} />)
		expect(fetchSpy).not.toHaveBeenCalled()
	})
})
