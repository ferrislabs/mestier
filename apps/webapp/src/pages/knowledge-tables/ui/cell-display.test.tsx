import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import type {
	ColumnKind,
	TableColumn,
} from '#/pages/knowledge-tables/lib/types'
import { CellDisplay } from '#/pages/knowledge-tables/ui/cell-display'

function column(kind: ColumnKind, overrides: Partial<TableColumn> = {}) {
	return {
		id: 'c1',
		name: 'Colonne',
		kind,
		config: {},
		is_primary: false,
		position: 1,
		...overrides,
	} satisfies TableColumn
}

describe('CellDisplay', () => {
	it('shows Sans titre in an empty primary cell', () => {
		render(
			<CellDisplay
				column={column('text', { is_primary: true })}
				value={undefined}
				label={null}
			/>,
		)
		expect(screen.getByText('Sans titre')).toBeTruthy()
	})

	it('shows an amount in euros', () => {
		render(
			<CellDisplay column={column('currency')} value={125050} label={null} />,
		)
		expect(screen.getByText(/1\s250,50\s€/)).toBeTruthy()
	})

	it('shows a deleted reference as Élément supprimé', () => {
		render(
			<CellDisplay
				column={column('link', { config: { link_kind: 'customer' } })}
				value={{ kind: 'customer', id: 'x' }}
				label={{ label: null, missing: true }}
			/>,
		)
		expect(screen.getByText('Élément supprimé')).toBeTruthy()
	})

	it('shows each chosen option as a chip', () => {
		render(
			<CellDisplay
				column={column('multi_select', {
					config: {
						options: [
							{ id: 'o1', label: 'Urgent', color: 'red' },
							{ id: 'o2', label: 'Client', color: 'blue' },
						],
					},
				})}
				value={['o2', 'o1']}
				label={null}
			/>,
		)
		expect(screen.getByText('Urgent')).toBeTruthy()
		expect(screen.getByText('Client')).toBeTruthy()
	})

	it('shows a checkbox as a mark read as Oui or Non', () => {
		const { rerender } = render(
			<CellDisplay
				column={column('checkbox', { name: 'Traité' })}
				value
				label={null}
			/>,
		)
		expect(screen.getByText('Oui')).toBeTruthy()
		expect(screen.queryByRole('checkbox')).toBeNull()
		rerender(
			<CellDisplay
				column={column('checkbox', { name: 'Traité' })}
				value={false}
				label={null}
			/>,
		)
		expect(screen.getByText('Non')).toBeTruthy()
	})
})
