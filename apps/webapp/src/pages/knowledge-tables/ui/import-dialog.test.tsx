import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { TableImportReport } from '#/pages/knowledge-tables/lib/types'
import { ImportDialog } from '#/pages/knowledge-tables/ui/import-dialog'

const REPORT: TableImportReport = {
	rows_created: 2,
	ignored_columns: ['Remarque'],
	invalid_cells: [
		{
			row: 3,
			column: 'Début',
			value: '31/02/2026',
			reason: 'not_date',
			message: 'expected a date written AAAA-MM-JJ',
		},
	],
	invalid_cell_count: 2,
}

function renderDialog(props: Partial<Parameters<typeof ImportDialog>[0]> = {}) {
	const handlers = {
		onFileChange: vi.fn(),
		onNameChange: vi.fn(),
		onSubmit: vi.fn(),
		onFinish: vi.fn(),
	}
	render(
		<ImportDialog
			open
			mode="rows"
			fileName={null}
			name=""
			report={null}
			error={null}
			isImporting={false}
			onOpenChange={() => {}}
			{...handlers}
			{...props}
		/>,
	)
	return handlers
}

describe('ImportDialog', () => {
	it('lists the rows created, the ignored columns and the invalid cells', async () => {
		const user = userEvent.setup()
		const { onFinish } = renderDialog({ report: REPORT, fileName: 'a.csv' })

		expect(screen.getByRole('status').textContent).toBe(
			'2 lignes importées, 1 colonne ignorée, 2 cellules laissées vides.',
		)
		expect(screen.getByText('Remarque')).toBeTruthy()
		const cell = screen.getByRole('row', { name: /31\/02\/2026/ })
		expect(cell.textContent).toContain('Début')
		expect(cell.textContent).toContain(
			'Date attendue (JJ/MM/AAAA ou AAAA-MM-JJ)',
		)
		expect(screen.getByText('Et 1 autre cellule.')).toBeTruthy()

		await user.click(screen.getByRole('button', { name: 'Terminer' }))
		expect(onFinish).toHaveBeenCalledOnce()
	})

	it('imports a new table only once a file is chosen and the table named', async () => {
		const user = userEvent.setup()
		const { onFileChange, onSubmit } = renderDialog({ mode: 'table' })
		const importer = () =>
			screen.getByRole('button', { name: 'Importer' }) as HTMLButtonElement

		expect(importer().disabled).toBe(true)
		const file = new File(['Nom\nA\n'], 'Stock.csv', { type: 'text/csv' })
		await user.upload(screen.getByLabelText('Fichier CSV'), file)
		expect(onFileChange).toHaveBeenCalledWith(file)
		expect(onSubmit).not.toHaveBeenCalled()
	})

	it('submits once the file and the name are there', async () => {
		const user = userEvent.setup()
		const { onSubmit } = renderDialog({
			mode: 'table',
			fileName: 'Stock.csv',
			name: 'Stock',
		})

		await user.click(screen.getByRole('button', { name: 'Importer' }))

		expect(onSubmit).toHaveBeenCalledOnce()
	})
})
