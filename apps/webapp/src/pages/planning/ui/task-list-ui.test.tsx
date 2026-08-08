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
		...overrides,
	}
}

describe('TaskListUI — rendu des lignes', () => {
	it('affiche chaque tâche racine avec son titre et ses assignés', () => {
		render(<TaskListUI {...baseProps()} />)
		expect(screen.getByText('Chantier toiture')).toBeDefined()
		expect(screen.getByText('Réunion hebdo')).toBeDefined()
		expect(screen.getByText('Alix Martin')).toBeDefined()
	})

	it("une racine sans enfant n'a pas de chevron", () => {
		render(<TaskListUI {...baseProps()} />)
		const row = screen.getByTestId('task-row-root-2')
		expect(
			row.querySelector('[aria-label="Afficher les sous-tâches"]'),
		).toBeNull()
	})

	it('une racine avec enfants affiche un chevron et son nombre d’enfants', () => {
		render(<TaskListUI {...baseProps()} />)
		const row = screen.getByTestId('task-row-root-1')
		expect(
			row.querySelector('[aria-label="Afficher les sous-tâches"]'),
		).not.toBeNull()
		expect(screen.getByText('2 sous-tâches')).toBeDefined()
	})

	it('affiche un état de chargement', () => {
		render(<TaskListUI {...baseProps({ isLoading: true, rows: [] })} />)
		expect(screen.getByText(/Chargement/)).toBeDefined()
	})

	it('affiche la fenêtre d’une tâche racine journée entière comme une date, pas une plage de minuit à minuit', () => {
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

	it('affiche un état vide', () => {
		render(<TaskListUI {...baseProps({ rows: [] })} />)
		expect(screen.getByText(/Aucune tâche/)).toBeDefined()
	})

	it("affiche l'erreur sans planter", () => {
		render(
			<TaskListUI {...baseProps({ error: 'Échec du chargement', rows: [] })} />,
		)
		expect(screen.getByText('Échec du chargement')).toBeDefined()
	})
})

describe('TaskListUI — dépliage', () => {
	it('appelle onToggleExpand avec l’id quand le chevron est cliqué, sans ouvrir la tâche', async () => {
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

	it('rend les lignes de sous-tâches fournies pour une racine dépliée', () => {
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

	it('ne rend aucune ligne de sous-tâche pour une racine repliée', () => {
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
	it('appelle onOpenTask avec l’id quand la ligne est cliquée', async () => {
		const user = userEvent.setup()
		const onOpenTask = vi.fn()
		render(<TaskListUI {...baseProps({ onOpenTask })} />)

		await user.click(screen.getByTestId('task-row-root-2'))
		expect(onOpenTask).toHaveBeenCalledWith('root-2')
	})

	it('rend le taskSheet fourni par la feature', () => {
		render(
			<TaskListUI
				{...baseProps({ taskSheet: <div data-testid="fake-sheet" /> })}
			/>,
		)
		expect(screen.getByTestId('fake-sheet')).toBeDefined()
	})
})

describe('TaskListUI — pagination', () => {
	it('désactive les boutons quand il n’y a pas de page suivante/précédente', () => {
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

	it('appelle onNextPage/onPreviousPage quand activés', async () => {
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

describe('TaskListUI — pas d’appel réseau', () => {
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
