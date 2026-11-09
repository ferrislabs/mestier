import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { DoubleClickConfirmButton } from './double-click-confirm-button'

describe('DoubleClickConfirmButton', () => {
	it('only confirms on the second click', async () => {
		const user = userEvent.setup()
		const onConfirm = vi.fn()
		render(<DoubleClickConfirmButton label="Supprimer" onConfirm={onConfirm} />)

		await user.click(screen.getByRole('button', { name: 'Supprimer' }))
		expect(onConfirm).not.toHaveBeenCalled()

		await user.click(
			screen.getByRole('button', { name: 'Cliquer à nouveau pour confirmer' }),
		)
		expect(onConfirm).toHaveBeenCalledTimes(1)
	})

	it('disarms when the focus leaves the button', async () => {
		const user = userEvent.setup()
		const onConfirm = vi.fn()
		render(
			<>
				<DoubleClickConfirmButton label="Supprimer" onConfirm={onConfirm} />
				<button type="button">ailleurs</button>
			</>,
		)

		await user.click(screen.getByRole('button', { name: 'Supprimer' }))
		await user.click(screen.getByRole('button', { name: 'ailleurs' }))

		expect(screen.getByRole('button', { name: 'Supprimer' })).toBeDefined()
		expect(onConfirm).not.toHaveBeenCalled()
	})
})
