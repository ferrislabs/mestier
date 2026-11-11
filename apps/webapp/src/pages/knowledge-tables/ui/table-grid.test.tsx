import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterAll, beforeAll, describe, expect, it, vi } from 'vitest'
import type { TableColumn, TableRow } from '#/pages/knowledge-tables/lib/types'
import { TableGrid } from '#/pages/knowledge-tables/ui/table-grid'

const name: TableColumn = {
	id: 'name',
	name: 'Nom',
	kind: 'text',
	config: {},
	is_primary: true,
	position: 0,
}
const qty: TableColumn = {
	...name,
	id: 'qty',
	name: 'Quantité',
	kind: 'number',
	is_primary: false,
	position: 1,
	config: { decimals: 1 },
}
const done: TableColumn = {
	...name,
	id: 'done',
	name: 'Traité',
	kind: 'checkbox',
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
	row('r1', { name: 'Martin', qty: 2 }),
	row('r2', { name: 'Durand', done: true }),
	row('r3', {}),
]

beforeAll(() => {
	Object.defineProperty(HTMLElement.prototype, 'clientHeight', {
		configurable: true,
		get: () => 600,
	})
	Object.defineProperty(HTMLElement.prototype, 'clientWidth', {
		configurable: true,
		get: () => 1000,
	})
})

afterAll(() => {
	Reflect.deleteProperty(HTMLElement.prototype, 'clientHeight')
	Reflect.deleteProperty(HTMLElement.prototype, 'clientWidth')
})

function renderGrid(overrides: Partial<Parameters<typeof TableGrid>[0]> = {}) {
	const props: Parameters<typeof TableGrid>[0] = {
		columns: [name, qty, done],
		rows,
		total: rows.length,
		widths: {},
		sorts: [],
		canManage: true,
		isLoading: false,
		hasMore: false,
		isLoadingMore: false,
		selected: new Set(),
		focusRowId: null,
		choicesFor: () => [],
		onLoadMore: vi.fn(),
		onCellChange: vi.fn(),
		onToggleRow: vi.fn(),
		onToggleAll: vi.fn(),
		onExpandRow: vi.fn(),
		onAddRow: vi.fn(),
		onAddColumn: vi.fn(),
		onColumnAction: vi.fn(),
		onRowFocused: vi.fn(),
		...overrides,
	}
	render(<TableGrid {...props} />)
	return props
}

describe('TableGrid', () => {
	it('shows the rows, Sans titre for an empty primary cell', () => {
		renderGrid()
		expect(screen.getByText('Martin')).toBeTruthy()
		expect(screen.getByText('Sans titre')).toBeTruthy()
		expect(screen.getByText('3 lignes')).toBeTruthy()
	})

	it('edits a cell with Enter and commits with Enter', async () => {
		const props = renderGrid()
		await userEvent.click(screen.getByText('Martin'))
		await userEvent.keyboard('{Enter}')
		const input = screen.getByRole('textbox', { name: 'Nom' })
		await userEvent.clear(input)
		await userEvent.type(input, 'Martin et fils{Enter}')
		expect(props.onCellChange).toHaveBeenCalledWith(
			'r1',
			'name',
			'Martin et fils',
		)
		expect(screen.queryByRole('textbox', { name: 'Nom' })).toBeNull()
	})

	it('commits with Tab and moves to the right', async () => {
		const props = renderGrid()
		await userEvent.click(screen.getByText('Martin'))
		await userEvent.keyboard('{Enter}')
		await userEvent.type(screen.getByRole('textbox', { name: 'Nom' }), ' SA')
		await userEvent.keyboard('{Tab}')
		expect(props.onCellChange).toHaveBeenCalledWith('r1', 'name', 'Martin SA')
		await userEvent.keyboard('{Enter}')
		expect(
			(screen.getByRole('textbox', { name: 'Quantité' }) as HTMLInputElement)
				.value,
		).toBe('2')
	})

	it('cancels with Escape', async () => {
		const props = renderGrid()
		await userEvent.click(screen.getByText('Martin'))
		await userEvent.keyboard('{Enter}')
		await userEvent.type(screen.getByRole('textbox', { name: 'Nom' }), 'xyz')
		await userEvent.keyboard('{Escape}')
		expect(props.onCellChange).not.toHaveBeenCalled()
		expect(screen.getByText('Martin')).toBeTruthy()
	})

	it('starts editing when typing, and reads a French decimal', async () => {
		const props = renderGrid()
		await userEvent.click(screen.getByText('Martin'))
		await userEvent.keyboard('{ArrowRight}{ArrowDown}')
		await userEvent.keyboard('3')
		await userEvent.keyboard(',5{Enter}')
		expect(props.onCellChange).toHaveBeenCalledWith('r2', 'qty', 3.5)
	})

	it('refuses a value that is not a number, without sending it', async () => {
		const props = renderGrid()
		await userEvent.click(screen.getByText('Martin'))
		await userEvent.keyboard('{ArrowRight}{Enter}')
		const input = screen.getByRole('textbox', { name: 'Quantité' })
		await userEvent.clear(input)
		await userEvent.type(input, 'beaucoup{Enter}')
		expect(props.onCellChange).not.toHaveBeenCalled()
		expect(screen.getByRole('alert').textContent).toContain('Saisis un nombre')
	})

	it('toggles a checkbox with the space bar', async () => {
		const props = renderGrid()
		await userEvent.click(screen.getByText('Durand'))
		await userEvent.keyboard('{ArrowRight}{ArrowRight}{ }')
		expect(props.onCellChange).toHaveBeenCalledWith('r2', 'done', false)
	})

	it('toggles a checkbox with a click', async () => {
		const props = renderGrid()
		const marks = screen.getAllByText('Non')
		await userEvent.click(marks[0])
		expect(props.onCellChange).toHaveBeenCalledWith('r1', 'done', true)
	})

	it('clears a cell with Delete', async () => {
		const props = renderGrid()
		await userEvent.click(screen.getByText('Martin'))
		await userEvent.keyboard('{Delete}')
		expect(props.onCellChange).toHaveBeenCalledWith('r1', 'name', null)
	})

	it('adds a row and a column', async () => {
		const props = renderGrid()
		await userEvent.click(
			screen.getByRole('button', { name: 'Nouvelle ligne' }),
		)
		await userEvent.click(
			screen.getByRole('button', { name: 'Ajouter une colonne' }),
		)
		expect(props.onAddRow).toHaveBeenCalledOnce()
		expect(props.onAddColumn).toHaveBeenCalledOnce()
	})

	it('opens a row from its expand button', async () => {
		const props = renderGrid()
		await userEvent.click(
			screen.getByRole('button', { name: 'Ouvrir la ligne Durand' }),
		)
		expect(props.onExpandRow).toHaveBeenCalledWith('r2')
	})

	it('is read only without the right to manage', async () => {
		const props = renderGrid({ canManage: false })
		expect(screen.queryByRole('button', { name: 'Nouvelle ligne' })).toBeNull()
		await userEvent.click(screen.getByText('Martin'))
		await userEvent.keyboard('{Enter}')
		expect(screen.queryByRole('textbox', { name: 'Nom' })).toBeNull()
		expect(props.onCellChange).not.toHaveBeenCalled()
	})

	it('asks for more rows near the end', () => {
		const props = renderGrid({ hasMore: true, total: 250 })
		expect(props.onLoadMore).toHaveBeenCalled()
	})
})
