import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { ExpressionPicker } from '#/pages/automation/ui/expression-picker'

describe('ExpressionPicker — no upstream connector', () => {
	it('explains there is nothing to reference yet, offers no path', async () => {
		const user = userEvent.setup()
		render(<ExpressionPicker upstream={[]} onInsert={vi.fn()} />)

		await user.click(
			screen.getByRole('button', { name: 'Insérer une expression' }),
		)

		expect(screen.getByText(/Aucun connecteur en amont/)).toBeDefined()
	})
})

describe('ExpressionPicker — picking a path', () => {
	it('inserts the whole-output expression for the root path', async () => {
		const user = userEvent.setup()
		const onInsert = vi.fn()
		render(
			<ExpressionPicker
				upstream={[
					{
						id: 'c1',
						label: 'Requête HTTP',
						paths: [{ path: '', preview: '{…}' }],
					},
				]}
				onInsert={onInsert}
			/>,
		)

		await user.click(
			screen.getByRole('button', { name: 'Insérer une expression' }),
		)
		await user.click(screen.getByText('output'))

		expect(onInsert).toHaveBeenCalledWith('{{ connectors.c1.output }}')
	})

	it('inserts a nested path exactly as built by buildConnectorExpression', async () => {
		const user = userEvent.setup()
		const onInsert = vi.fn()
		render(
			<ExpressionPicker
				upstream={[
					{
						id: 'c1',
						label: 'Requête HTTP',
						paths: [{ path: 'quote.total', preview: '42' }],
					},
				]}
				onInsert={onInsert}
			/>,
		)

		await user.click(
			screen.getByRole('button', { name: 'Insérer une expression' }),
		)
		await user.click(screen.getByText('quote.total'))

		expect(onInsert).toHaveBeenCalledWith(
			'{{ connectors.c1.output.quote.total }}',
		)
	})

	it('groups paths under their connector label', async () => {
		const user = userEvent.setup()
		render(
			<ExpressionPicker
				upstream={[
					{
						id: 'c1',
						label: 'Requête HTTP',
						paths: [{ path: '', preview: '{…}' }],
					},
					{
						id: 'c2',
						label: 'Lecture Odoo',
						paths: [{ path: '', preview: '{…}' }],
					},
				]}
				onInsert={vi.fn()}
			/>,
		)

		await user.click(
			screen.getByRole('button', { name: 'Insérer une expression' }),
		)

		expect(screen.getByText('Requête HTTP')).toBeDefined()
		expect(screen.getByText('Lecture Odoo')).toBeDefined()
	})
})
