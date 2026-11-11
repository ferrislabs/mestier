import { fireEvent, render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { TableColumn, TableRow } from '#/pages/knowledge-tables/lib/types'
import { groupRows } from '#/pages/knowledge-tables/lib/view-config'
import { KanbanBoard } from '#/pages/knowledge-tables/ui/kanban-board'

const name: TableColumn = {
	id: 'name',
	name: 'Nom',
	kind: 'text',
	config: {},
	is_primary: true,
	position: 0,
}
const status: TableColumn = {
	...name,
	id: 'status',
	name: 'Statut',
	kind: 'single_select',
	is_primary: false,
	position: 1,
	config: {
		options: [
			{ id: 'todo', label: 'À faire', color: 'gray' },
			{ id: 'doing', label: 'En cours', color: 'blue' },
		],
	},
}
const amount: TableColumn = {
	...name,
	id: 'amount',
	name: 'Montant',
	kind: 'currency',
	is_primary: false,
	position: 2,
}

function row(id: string, values: Record<string, unknown>): TableRow {
	return {
		id,
		values,
		labels: {},
		created_at: '2026-11-11T08:00:00Z',
		updated_at: '2026-11-11T08:00:00Z',
	}
}

const rows = [
	row('r1', { name: 'Martin', status: 'todo', amount: 125050 }),
	row('r2', { name: 'Durand', status: 'doing' }),
	row('r3', {}),
]

function renderBoard(
	overrides: Partial<Parameters<typeof KanbanBoard>[0]> = {},
) {
	const props: Parameters<typeof KanbanBoard>[0] = {
		groups: groupRows(rows, status),
		primary: name,
		fields: [amount],
		groupColumn: status,
		canManage: true,
		isLoading: false,
		onMoveRow: vi.fn(),
		onOpenRow: vi.fn(),
		onAddRow: vi.fn(),
		...overrides,
	}
	render(<KanbanBoard {...props} />)
	return props
}

describe('KanbanBoard', () => {
	it('shows one column per option and one for the rows without value', () => {
		renderBoard()
		const sansValeur = screen.getByRole('region', {
			name: 'Colonne Sans valeur',
		})
		const todo = screen.getByRole('region', { name: 'Colonne À faire' })
		const doing = screen.getByRole('region', { name: 'Colonne En cours' })
		expect(within(sansValeur).getByText('Sans titre')).toBeTruthy()
		expect(within(todo).getByText('Martin')).toBeTruthy()
		expect(within(todo).getByText(/1\s250,50\s€/)).toBeTruthy()
		expect(within(doing).getByText('Durand')).toBeTruthy()
	})

	it('moves a dropped card to the option of its new column', () => {
		const props = renderBoard()
		const card = screen.getByRole('button', { name: /Martin/ })
		const doing = screen.getByRole('region', { name: 'Colonne En cours' })
		fireEvent.dragStart(card)
		fireEvent.dragOver(doing)
		fireEvent.drop(doing)
		expect(props.onMoveRow).toHaveBeenCalledWith('r1', 'doing')
	})

	it('does not move a card dropped in its own column', () => {
		const props = renderBoard()
		const card = screen.getByRole('button', { name: /Martin/ })
		const todo = screen.getByRole('region', { name: 'Colonne À faire' })
		fireEvent.dragStart(card)
		fireEvent.drop(todo)
		expect(props.onMoveRow).not.toHaveBeenCalled()
	})

	it('opens a card on click', async () => {
		const props = renderBoard()
		await userEvent.click(screen.getByRole('button', { name: /Durand/ }))
		expect(props.onOpenRow).toHaveBeenCalledWith('r2')
	})

	it('adds a row in a column with its option', async () => {
		const props = renderBoard()
		await userEvent.click(
			screen.getByRole('button', { name: 'Ajouter une ligne dans En cours' }),
		)
		expect(props.onAddRow).toHaveBeenCalledWith('doing')
	})

	it('hides the writes without the right to manage', () => {
		renderBoard({ canManage: false })
		expect(
			screen.queryByRole('button', { name: 'Ajouter une ligne dans En cours' }),
		).toBeNull()
		expect(
			screen.getByRole('button', { name: /Martin/ }).getAttribute('draggable'),
		).toBe('false')
	})
})
