import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { ExpressionPreview } from '#/pages/automation/ui/expression-preview'

describe('ExpressionPreview', () => {
	it('renders nothing while idle', () => {
		const { container } = render(
			<ExpressionPreview state={{ status: 'idle' }} />,
		)

		expect(container.textContent).toBe('')
	})

	it('shows a placeholder while the evaluation is in flight', () => {
		render(<ExpressionPreview state={{ status: 'loading' }} />)

		expect(screen.getByText('Aperçu : …')).toBeDefined()
	})

	it('shows the resolved value as JSON', () => {
		render(
			<ExpressionPreview state={{ status: 'resolved', value: { id: 42 } }} />,
		)

		expect(screen.getByText('Aperçu : {"id":42}')).toBeDefined()
	})

	it('shows a string value without quoting it', () => {
		render(
			<ExpressionPreview
				state={{ status: 'resolved', value: 'https://example.test' }}
			/>,
		)

		expect(screen.getByText('Aperçu : https://example.test')).toBeDefined()
	})

	it('names the missing path as an alert, never a silent null', () => {
		render(
			<ExpressionPreview
				state={{
					status: 'error',
					message: 'missing path: trigger.customer.id',
				}}
			/>,
		)

		const alert = screen.getByRole('alert')
		expect(alert.textContent).toContain('trigger.customer.id')
	})
})
