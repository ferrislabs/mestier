import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { useState } from 'react'
import { describe, expect, it, vi } from 'vitest'
import { WebhookPathField } from '#/pages/automation/ui/webhook-path-field'

function Harness({
	initial = '',
	invalid = false,
	onChange = vi.fn(),
}: {
	initial?: string
	invalid?: boolean
	onChange?: (value: string) => void
}) {
	const [value, setValue] = useState(initial)
	return (
		<>
			<WebhookPathField
				controlId="path"
				value={value}
				invalid={invalid}
				origin="https://mestier.test"
				onChange={(next) => {
					setValue(next)
					onChange(next)
				}}
			/>
			<button type="button">ailleurs</button>
		</>
	)
}

describe('WebhookPathField', () => {
	it('shows the imposed prefix and the full URL', () => {
		render(<Harness initial="stripe/paiement-recu" />)

		expect(screen.getByTitle('Préfixe imposé').textContent).toBe('/webhooks/')
		expect(
			screen.getByText('https://mestier.test/webhooks/stripe/paiement-recu'),
		).toBeDefined()
	})

	it('hints at the format with a placeholder', () => {
		render(<Harness />)

		expect(
			(document.getElementById('path') as HTMLInputElement).placeholder,
		).toBe('stripe/paiement-recu')
	})

	it('lets the user type freely and follows the URL as they go', async () => {
		const user = userEvent.setup()
		render(<Harness />)

		await user.type(
			document.getElementById('path') as HTMLElement,
			'Mon Chemin',
		)

		expect((document.getElementById('path') as HTMLInputElement).value).toBe(
			'Mon Chemin',
		)
		expect(
			screen.getByText('https://mestier.test/webhooks/Mon Chemin'),
		).toBeDefined()
	})

	it('normalizes the path when the field is left', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(<Harness onChange={onChange} />)

		await user.type(
			document.getElementById('path') as HTMLElement,
			'/Webhooks/Paiement Reçu/',
		)
		await user.click(screen.getByRole('button', { name: 'ailleurs' }))

		expect(onChange).toHaveBeenLastCalledWith('paiement-recu')
		expect((document.getElementById('path') as HTMLInputElement).value).toBe(
			'paiement-recu',
		)
		expect(
			screen.getByText('https://mestier.test/webhooks/paiement-recu'),
		).toBeDefined()
	})

	it('does not report a change when the path is already normalized', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(<Harness initial="paiement-recu" onChange={onChange} />)

		await user.click(document.getElementById('path') as HTMLElement)
		await user.click(screen.getByRole('button', { name: 'ailleurs' }))

		expect(onChange).not.toHaveBeenCalled()
	})

	it('marks the field invalid', () => {
		render(<Harness invalid />)

		expect(document.getElementById('path')?.getAttribute('aria-invalid')).toBe(
			'true',
		)
	})

	it('copies the full URL and says so for a moment', async () => {
		const user = userEvent.setup()
		render(<Harness initial="paiement-recu" />)

		await user.click(
			screen.getByRole('button', { name: 'Copier l’URL du webhook' }),
		)

		expect(await navigator.clipboard.readText()).toBe(
			'https://mestier.test/webhooks/paiement-recu',
		)
	})
})
