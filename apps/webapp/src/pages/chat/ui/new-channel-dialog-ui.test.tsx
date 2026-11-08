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
		channelKind: 'TEXT',
		onChangeChannelKind: vi.fn(),
		isPrivate: false,
		onChangePrivate: vi.fn(),
		roles: [
			{ id: 'r-1', name: 'Chefs de chantier' },
			{ id: 'r-2', name: 'Administration' },
		],
		rolesLoading: false,
		selectedRoleIds: new Set<string>(),
		onToggleRole: vi.fn(),
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

	it('offers the three channel types as cards with a title and a description', () => {
		render(<NewChannelDialogUI {...baseProps()} />)

		expect(screen.getByRole('radio', { name: /Texte/ })).toBeDefined()
		expect(screen.getByRole('radio', { name: /Annonces/ })).toBeDefined()
		expect(screen.getByRole('radio', { name: /Vocal/ })).toBeDefined()
		expect(
			screen.getByText(/Seuls les gestionnaires de canaux publient/),
		).toBeDefined()
	})

	it('reports the type picked', async () => {
		const user = userEvent.setup()
		const onChangeChannelKind = vi.fn()
		render(<NewChannelDialogUI {...baseProps({ onChangeChannelKind })} />)

		await user.click(screen.getByRole('radio', { name: /Annonces/ }))

		expect(onChangeChannelKind).toHaveBeenCalledWith('ANNOUNCEMENT')
	})

	it('hides the roles until the channel is private', () => {
		const { rerender } = render(<NewChannelDialogUI {...baseProps()} />)
		expect(screen.queryByText('Rôles autorisés')).toBeNull()

		rerender(<NewChannelDialogUI {...baseProps({ isPrivate: true })} />)

		expect(screen.getByText('Rôles autorisés')).toBeDefined()
		expect(screen.getByText('Chefs de chantier')).toBeDefined()
	})

	it('reports the toggle of a role', async () => {
		const user = userEvent.setup()
		const onToggleRole = vi.fn()
		render(
			<NewChannelDialogUI {...baseProps({ isPrivate: true, onToggleRole })} />,
		)

		await user.click(screen.getByRole('checkbox', { name: 'Administration' }))

		expect(onToggleRole).toHaveBeenCalledWith('r-2', true)
	})

	it('refuses a private channel with no role', () => {
		render(
			<NewChannelDialogUI
				{...baseProps({ name: 'direction', isPrivate: true })}
			/>,
		)

		expect(
			screen.getByRole('button', { name: 'Créer' }).hasAttribute('disabled'),
		).toBe(true)
	})

	it('accepts a private channel once a role is chosen', () => {
		render(
			<NewChannelDialogUI
				{...baseProps({
					name: 'direction',
					isPrivate: true,
					selectedRoleIds: new Set(['r-1']),
				})}
			/>,
		)

		expect(
			screen.getByRole('button', { name: 'Créer' }).hasAttribute('disabled'),
		).toBe(false)
	})
})
