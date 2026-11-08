import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { NewDirectMessageDialogUI } from './new-direct-message-dialog-ui'

const members = [
	{ id: 'u-1', name: 'Karim Benali', hue: 200, presence: 'ONLINE' as const },
	{ id: 'u-2', name: 'Alix Martin', hue: 20, presence: 'OFFLINE' as const },
]

describe('NewDirectMessageDialogUI', () => {
	it('lists the members and reports the one picked', async () => {
		const user = userEvent.setup()
		const onSelect = vi.fn()
		render(
			<NewDirectMessageDialogUI
				open
				onOpenChange={vi.fn()}
				members={members}
				isSubmitting={false}
				onSelect={onSelect}
			/>,
		)

		await user.click(await screen.findByText('Alix Martin'))

		expect(onSelect).toHaveBeenCalledWith('u-2')
	})

	it('narrows the list as the user types', async () => {
		const user = userEvent.setup()
		render(
			<NewDirectMessageDialogUI
				open
				onOpenChange={vi.fn()}
				members={members}
				isSubmitting={false}
				onSelect={vi.fn()}
			/>,
		)

		await user.type(
			await screen.findByPlaceholderText('Rechercher un membre'),
			'karim',
		)

		expect(screen.queryByText('Alix Martin')).toBeNull()
		expect(screen.getByText('Karim Benali')).toBeDefined()
	})
})
