import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { SubtaskList } from '#/pages/planning/ui/subtask-list'

const SUBTASKS = [
	{
		id: 'sub-1',
		title: 'Poser les tuiles',
		status: 'PLANNED' as const,
		assigneeCount: 1,
		inheritedWindow: false,
	},
	{
		id: 'sub-2',
		title: 'Nettoyer le projet',
		status: 'DONE' as const,
		assigneeCount: 0,
		inheritedWindow: true,
	},
]

function baseProps() {
	return {
		subtasks: SUBTASKS,
		isLoading: false,
		error: null,
		canAddSubtask: true,
		onAddSubtask: vi.fn(),
		onOpenSubtask: vi.fn(),
	}
}

describe('SubtaskList — rendu', () => {
	it('renders every subtask title', () => {
		render(<SubtaskList {...baseProps()} />)
		expect(screen.getByText('Poser les tuiles')).toBeDefined()
		expect(screen.getByText('Nettoyer le projet')).toBeDefined()
	})

	it('flags a subtask whose window is inherited from the parent', () => {
		render(<SubtaskList {...baseProps()} />)
		const inheriting = screen.getByTestId('subtask-sub-2')
		expect(inheriting.textContent).toMatch(/hérité/i)

		const ownDates = screen.getByTestId('subtask-sub-1')
		expect(ownDates.textContent).not.toMatch(/hérité/i)
	})

	it('shows a loading state', () => {
		render(<SubtaskList {...baseProps()} subtasks={[]} isLoading={true} />)
		expect(screen.getByText(/Chargement/)).toBeDefined()
	})

	it('shows an empty state', () => {
		render(<SubtaskList {...baseProps()} subtasks={[]} />)
		expect(screen.getByText(/Aucune sous-tâche/)).toBeDefined()
	})

	it('shows an error state', () => {
		render(<SubtaskList {...baseProps()} subtasks={[]} error="Échec" />)
		expect(screen.getByText('Échec')).toBeDefined()
	})
})

describe('SubtaskList — two-level depth', () => {
	it('offers to add a subtask on a root task', () => {
		render(<SubtaskList {...baseProps()} canAddSubtask={true} />)
		expect(
			screen.getByRole('button', { name: /Ajouter une sous-tâche/ }),
		).toBeDefined()
	})

	it('never offers to add a subtask under a subtask — the third level is not proposed', () => {
		render(<SubtaskList {...baseProps()} canAddSubtask={false} />)
		expect(
			screen.queryByRole('button', { name: /Ajouter une sous-tâche/ }),
		).toBeNull()
	})
})

describe('SubtaskList — interactions', () => {
	it('calls onAddSubtask when the add button is clicked', async () => {
		const user = userEvent.setup()
		const onAddSubtask = vi.fn()
		render(<SubtaskList {...baseProps()} onAddSubtask={onAddSubtask} />)

		await user.click(
			screen.getByRole('button', { name: /Ajouter une sous-tâche/ }),
		)
		expect(onAddSubtask).toHaveBeenCalledTimes(1)
	})

	it('calls onOpenSubtask with the id when a row is clicked', async () => {
		const user = userEvent.setup()
		const onOpenSubtask = vi.fn()
		render(<SubtaskList {...baseProps()} onOpenSubtask={onOpenSubtask} />)

		await user.click(screen.getByText('Poser les tuiles'))
		expect(onOpenSubtask).toHaveBeenCalledWith('sub-1')
	})
})

describe('SubtaskList — no network call', () => {
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
		render(<SubtaskList {...baseProps()} />)
		expect(fetchSpy).not.toHaveBeenCalled()
	})
})
