import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
	type TaskListRowVM,
	TaskListUI,
	type TaskListUIProps,
} from '#/pages/planning/ui/task-list-ui'

const ROOT_WITH_CHILDREN: TaskListRowVM = {
	id: 'root-1',
	title: 'Chantier toiture',
	status: 'PLANNED',
	labels: [{ id: 'l1', name: 'Réunion', color: '#2563EB' }],
	childCount: 2,
	hasChildren: true,
	window: {
		startsAt: '2026-08-10T07:00:00.000Z',
		endsAt: '2026-08-10T09:00:00.000Z',
	},
	allDay: false,
	assigneeNames: ['Alix Martin'],
	isExpanded: false,
}

const ROOT_WITHOUT_CHILDREN: TaskListRowVM = {
	id: 'root-2',
	title: 'Réunion hebdo',
	status: 'DONE',
	labels: [],
	childCount: 0,
	hasChildren: false,
	window: {
		startsAt: '2026-08-11T07:00:00.000Z',
		endsAt: '2026-08-11T09:00:00.000Z',
	},
	allDay: false,
	assigneeNames: [],
	isExpanded: false,
}

function baseProps(overrides: Partial<TaskListUIProps> = {}): TaskListUIProps {
	return {
		organizationName: 'Atelier Bois & Co',
		isLoading: false,
		error: null,
		rows: [ROOT_WITH_CHILDREN, ROOT_WITHOUT_CHILDREN],
		timeZone: 'Europe/Paris',
		pagination: {
			page: 1,
			canGoToNext: false,
			canGoToPrevious: false,
			total: 2,
		},
		onNextPage: vi.fn(),
		onPreviousPage: vi.fn(),
		onToggleExpand: vi.fn(),
		subtaskRowsByTaskId: {},
		onOpenTask: vi.fn(),
		onCreateTask: vi.fn(),
		selectedTaskIds: [],
		onToggleRowSelection: vi.fn(),
		onToggleSelectAll: vi.fn(),
		...overrides,
	}
}

describe('TaskListUI — row rendering', () => {
	it('shows every root task with its title and its assignees', () => {
		render(<TaskListUI {...baseProps()} />)
		expect(screen.getByText('Chantier toiture')).toBeDefined()
		expect(screen.getByText('Réunion hebdo')).toBeDefined()
		expect(screen.getByText('Alix Martin')).toBeDefined()
	})

	it('a root with no child has no chevron', () => {
		render(<TaskListUI {...baseProps()} />)
		const row = screen.getByTestId('task-row-root-2')
		expect(
			row.querySelector('[aria-label="Afficher les sous-tâches"]'),
		).toBeNull()
	})

	it('a root with children shows a chevron and its child count', () => {
		render(<TaskListUI {...baseProps()} />)
		const row = screen.getByTestId('task-row-root-1')
		expect(
			row.querySelector('[aria-label="Afficher les sous-tâches"]'),
		).not.toBeNull()
		expect(screen.getByText('2 sous-tâches')).toBeDefined()
	})

	it('shows a loading state', () => {
		render(<TaskListUI {...baseProps({ isLoading: true, rows: [] })} />)
		expect(screen.getByText(/Chargement/)).toBeDefined()
	})

	it("shows a full-day root task's window as a date, not a midnight-to-midnight range", () => {
		const allDayRoot: TaskListRowVM = {
			...ROOT_WITHOUT_CHILDREN,
			id: 'root-3',
			title: 'Salon professionnel',
			window: {
				startsAt: '2026-08-09T22:00:00.000Z',
				endsAt: '2026-08-10T22:00:00.000Z',
			},
			allDay: true,
		}
		render(<TaskListUI {...baseProps({ rows: [allDayRoot] })} />)

		const row = screen.getByTestId('task-row-root-3')
		expect(row.textContent).toContain('10/08/2026')
		expect(row.textContent).not.toContain('00:00')
	})

	it('shows an empty state', () => {
		render(<TaskListUI {...baseProps({ rows: [] })} />)
		expect(screen.getByText(/Aucune tâche/)).toBeDefined()
	})

	it('shows the error without crashing', () => {
		render(
			<TaskListUI {...baseProps({ error: 'Échec du chargement', rows: [] })} />,
		)
		expect(screen.getByText('Échec du chargement')).toBeDefined()
	})
})

describe('TaskListUI — expansion', () => {
	it('calls onToggleExpand with the id when the chevron is clicked, without opening the task', async () => {
		const user = userEvent.setup()
		const onToggleExpand = vi.fn()
		const onOpenTask = vi.fn()
		render(<TaskListUI {...baseProps({ onToggleExpand, onOpenTask })} />)

		await user.click(
			screen.getByRole('button', { name: 'Afficher les sous-tâches' }),
		)
		expect(onToggleExpand).toHaveBeenCalledWith('root-1')
		expect(onOpenTask).not.toHaveBeenCalled()
	})

	it('renders the given subtask rows for an expanded root', () => {
		render(
			<TaskListUI
				{...baseProps({
					rows: [{ ...ROOT_WITH_CHILDREN, isExpanded: true }],
					subtaskRowsByTaskId: {
						'root-1': <tr data-testid="fake-subtask-row" />,
					},
				})}
			/>,
		)
		expect(screen.getByTestId('fake-subtask-row')).toBeDefined()
	})

	it('renders no subtask row for a collapsed root', () => {
		render(
			<TaskListUI
				{...baseProps({
					subtaskRowsByTaskId: {
						'root-1': <tr data-testid="fake-subtask-row" />,
					},
				})}
			/>,
		)
		expect(screen.queryByTestId('fake-subtask-row')).toBeNull()
	})
})

describe('TaskListUI — ouverture', () => {
	it('calls onOpenTask with the id when the row is clicked', async () => {
		const user = userEvent.setup()
		const onOpenTask = vi.fn()
		render(<TaskListUI {...baseProps({ onOpenTask })} />)

		await user.click(screen.getByTestId('task-row-root-2'))
		expect(onOpenTask).toHaveBeenCalledWith('root-2')
	})

	it('renders the taskSheet provided by the feature', () => {
		render(
			<TaskListUI
				{...baseProps({ taskSheet: <div data-testid="fake-sheet" /> })}
			/>,
		)
		expect(screen.getByTestId('fake-sheet')).toBeDefined()
	})
})

describe('TaskListUI — selection', () => {
	it('calls onToggleRowSelection with the id, without opening the task', async () => {
		const user = userEvent.setup()
		const onToggleRowSelection = vi.fn()
		const onOpenTask = vi.fn()
		render(<TaskListUI {...baseProps({ onToggleRowSelection, onOpenTask })} />)

		await user.click(
			screen.getByRole('checkbox', { name: 'Sélectionner Chantier toiture' }),
		)
		expect(onToggleRowSelection).toHaveBeenCalledWith('root-1')
		expect(onOpenTask).not.toHaveBeenCalled()
	})

	it('a selected row shows its checkbox checked', () => {
		render(<TaskListUI {...baseProps({ selectedTaskIds: ['root-1'] })} />)

		const checkbox = screen.getByRole('checkbox', {
			name: 'Sélectionner Chantier toiture',
		})
		expect(checkbox.getAttribute('aria-checked')).toBe('true')
	})

	it('the select-all checkbox is checked once every row is selected', () => {
		render(
			<TaskListUI {...baseProps({ selectedTaskIds: ['root-1', 'root-2'] })} />,
		)

		const selectAll = screen.getByRole('checkbox', {
			name: 'Sélectionner toutes les tâches',
		})
		expect(selectAll.getAttribute('aria-checked')).toBe('true')
	})

	it('calls onToggleSelectAll when the header checkbox is clicked', async () => {
		const user = userEvent.setup()
		const onToggleSelectAll = vi.fn()
		render(<TaskListUI {...baseProps({ onToggleSelectAll })} />)

		await user.click(
			screen.getByRole('checkbox', { name: 'Sélectionner toutes les tâches' }),
		)
		expect(onToggleSelectAll).toHaveBeenCalledTimes(1)
	})

	it('renders the bulkAssignBar provided by the feature', () => {
		render(
			<TaskListUI
				{...baseProps({
					bulkAssignBar: <div data-testid="fake-bulk-assign-bar" />,
				})}
			/>,
		)
		expect(screen.getByTestId('fake-bulk-assign-bar')).toBeDefined()
	})
})

describe('TaskListUI — pagination', () => {
	it('disables the buttons when there is no next/previous page', () => {
		render(<TaskListUI {...baseProps()} />)
		expect(
			screen
				.getByRole('button', { name: 'Précédent' })
				.hasAttribute('disabled'),
		).toBe(true)
		expect(
			screen.getByRole('button', { name: 'Suivant' }).hasAttribute('disabled'),
		).toBe(true)
	})

	it('calls onNextPage/onPreviousPage when enabled', async () => {
		const user = userEvent.setup()
		const onNextPage = vi.fn()
		const onPreviousPage = vi.fn()
		render(
			<TaskListUI
				{...baseProps({
					pagination: {
						page: 2,
						canGoToNext: true,
						canGoToPrevious: true,
						total: 45,
					},
					onNextPage,
					onPreviousPage,
				})}
			/>,
		)

		await user.click(screen.getByRole('button', { name: 'Suivant' }))
		expect(onNextPage).toHaveBeenCalledTimes(1)
		await user.click(screen.getByRole('button', { name: 'Précédent' }))
		expect(onPreviousPage).toHaveBeenCalledTimes(1)
	})
})

describe('TaskListUI — no network call', () => {
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
		render(<TaskListUI {...baseProps()} />)
		expect(fetchSpy).not.toHaveBeenCalled()
	})
})
