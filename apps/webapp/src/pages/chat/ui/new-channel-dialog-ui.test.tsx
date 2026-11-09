import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { NewChannelDialogUIProps } from './new-channel-dialog-ui'
import { NewChannelDialogUI } from './new-channel-dialog-ui'

function baseProps(
	overrides: Partial<NewChannelDialogUIProps> = {},
): NewChannelDialogUIProps {
	return {
		open: true,
		onOpenChange: vi.fn(),
		kind: 'channel',
		onChangeKind: vi.fn(),
		name: '',
		onChangeName: vi.fn(),
		onSubmit: vi.fn(),
		isSubmitting: false,
		...overrides,
	}
}

describe('NewChannelDialogUI', () => {
	it('disables submit while the name is empty', () => {
		render(<NewChannelDialogUI {...baseProps({ name: '' })} />)
		expect(
			screen.getByRole('button', { name: 'Créer' }).hasAttribute('disabled'),
		).toBe(true)
	})

	it('enables submit once a name is entered', () => {
		render(<NewChannelDialogUI {...baseProps({ name: 'général' })} />)
		expect(
			screen.getByRole('button', { name: 'Créer' }).hasAttribute('disabled'),
		).toBe(false)
	})

	it('calls onSubmit when creating', async () => {
		const user = userEvent.setup()
		const onSubmit = vi.fn()
		render(<NewChannelDialogUI {...baseProps({ name: 'général', onSubmit })} />)

		await user.click(screen.getByRole('button', { name: 'Créer' }))

		expect(onSubmit).toHaveBeenCalledTimes(1)
	})

	it('switches between channel and category kind', async () => {
		const user = userEvent.setup()
		const onChangeKind = vi.fn()
		render(<NewChannelDialogUI {...baseProps({ onChangeKind })} />)

		await user.click(screen.getByRole('tab', { name: 'Catégorie' }))

		expect(onChangeKind).toHaveBeenCalledWith('category')
	})

	it('offers no category choice for a channel', () => {
		render(<NewChannelDialogUI {...baseProps()} />)

		expect(screen.queryByRole('combobox')).toBeNull()
		expect(screen.queryByText('Sans catégorie')).toBeNull()
	})

	it('names the category it was opened from and offers only a channel', () => {
		render(
			<NewChannelDialogUI
				{...baseProps({ parentCategoryName: 'Chantiers' })}
			/>,
		)

		expect(screen.getByText('Nouveau canal dans Chantiers')).toBeDefined()
		expect(screen.queryByRole('tab', { name: 'Catégorie' })).toBeNull()
	})
})
