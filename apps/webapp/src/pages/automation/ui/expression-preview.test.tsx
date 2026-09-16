import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { ExpressionPreview } from '#/pages/automation/ui/expression-preview'

describe('ExpressionPreview', () => {
	it('renders nothing when no field is active', () => {
		const { container } = render(
			<ExpressionPreview fieldLabel={null} state={{ status: 'idle' }} />,
		)

		expect(container.textContent).toBe('')
	})

	it('names the active field while idle', () => {
		render(<ExpressionPreview fieldLabel="URL" state={{ status: 'idle' }} />)

		expect(screen.getByText(/URL/)).toBeDefined()
	})

	it('shows a loading state while the evaluation is in flight', () => {
		render(<ExpressionPreview fieldLabel="URL" state={{ status: 'loading' }} />)

		expect(screen.getByText(/valuation en cours/)).toBeDefined()
	})

	it('shows the resolved value as JSON', () => {
		render(
			<ExpressionPreview
				fieldLabel="URL"
				state={{ status: 'resolved', value: { id: 42 } }}
			/>,
		)

		expect(screen.getByText('{"id":42}')).toBeDefined()
	})

	it('shows a string value without JSON quoting noise lost', () => {
		render(
			<ExpressionPreview
				fieldLabel="URL"
				state={{ status: 'resolved', value: 'https://example.test' }}
			/>,
		)

		expect(screen.getByText('"https://example.test"')).toBeDefined()
	})

	it('names the missing path as an alert, never a silent null', () => {
		render(
			<ExpressionPreview
				fieldLabel="URL"
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
