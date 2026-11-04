import { fireEvent, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { SupportToolbarProps } from '#/pages/support/types'
import { SupportToolbar } from '#/pages/support/ui/support-toolbar'

function renderToolbar(overrides: Partial<SupportToolbarProps> = {}) {
	const props: SupportToolbarProps = {
		filters: {},
		assignees: [
			{ id: 'member-1', label: 'Alix Martin' },
			{ id: 'member-2', label: 'Sam Roux' },
		],
		badges: [
			{ id: 'badge-1', label: 'Urgent' },
			{ id: 'badge-2', label: 'Garantie' },
		],
		fieldKeys: ['Marque', 'Modèle'],
		isLoading: false,
		isFiltered: false,
		onChange: vi.fn(),
		onClear: vi.fn(),
		...overrides,
	}
	const view = render(<SupportToolbar {...props} />)
	return { props, ...view }
}

describe('SupportToolbar', () => {
	it('offers every assignee after the all option', () => {
		renderToolbar()

		const select = screen.getByLabelText('Assigné à') as HTMLSelectElement
		expect(
			Array.from(select.options).map((option) => option.textContent),
		).toEqual(['Tout le monde', 'Alix Martin', 'Sam Roux'])
	})

	it('reports the chosen assignee', async () => {
		const { props } = renderToolbar()

		await userEvent.selectOptions(
			screen.getByLabelText('Assigné à'),
			'member-2',
		)

		expect(props.onChange).toHaveBeenCalledWith({ assignee: 'member-2' })
	})

	it('clears the assignee when the all option is chosen', async () => {
		const { props } = renderToolbar({ filters: { assignee: 'member-2' } })

		const select = screen.getByLabelText('Assigné à') as HTMLSelectElement
		expect(select.value).toBe('member-2')
		await userEvent.selectOptions(select, '')

		expect(props.onChange).toHaveBeenCalledWith({ assignee: undefined })
	})

	it('reports the chosen badge and clears it with the all option', async () => {
		const { props } = renderToolbar({ filters: { badge: 'badge-1' } })

		const select = screen.getByLabelText('Badge') as HTMLSelectElement
		await userEvent.selectOptions(select, 'badge-2')
		await userEvent.selectOptions(select, '')

		expect(props.onChange).toHaveBeenNthCalledWith(1, { badge: 'badge-2' })
		expect(props.onChange).toHaveBeenNthCalledWith(2, { badge: undefined })
	})

	it('reports the chosen field key', async () => {
		const { props } = renderToolbar()

		const select = screen.getByLabelText('Champ') as HTMLSelectElement
		expect(
			Array.from(select.options).map((option) => option.textContent),
		).toEqual(['Tous les champs', 'Marque', 'Modèle'])
		await userEvent.selectOptions(select, 'Modèle')

		expect(vi.mocked(props.onChange).mock.calls).toStrictEqual([
			[{ custom_field_key: 'Modèle', custom_field_value: undefined }],
		])
	})

	it('clears the field value when another field key replaces the current one', async () => {
		const { props } = renderToolbar({
			filters: { custom_field_key: 'Marque', custom_field_value: 'Grohe' },
		})

		await userEvent.selectOptions(screen.getByLabelText('Champ'), 'Modèle')

		expect(vi.mocked(props.onChange).mock.calls).toStrictEqual([
			[{ custom_field_key: 'Modèle', custom_field_value: undefined }],
		])
	})

	it('clears the field value together with the key when the all option is chosen', async () => {
		const { props } = renderToolbar({
			filters: { custom_field_key: 'Marque', custom_field_value: 'Grohe' },
		})

		await userEvent.selectOptions(screen.getByLabelText('Champ'), '')

		expect(props.onChange).toHaveBeenCalledWith({
			custom_field_key: undefined,
			custom_field_value: undefined,
		})
	})

	it('keeps the value input disabled until a field key is chosen', () => {
		const { rerender, props } = renderToolbar()

		expect((screen.getByLabelText('Valeur') as HTMLInputElement).disabled).toBe(
			true,
		)

		rerender(
			<SupportToolbar {...props} filters={{ custom_field_key: 'Marque' }} />,
		)

		expect((screen.getByLabelText('Valeur') as HTMLInputElement).disabled).toBe(
			false,
		)
	})

	it('shows the applied value in the input', () => {
		renderToolbar({
			filters: { custom_field_key: 'Marque', custom_field_value: 'Grohe' },
		})

		expect((screen.getByLabelText('Valeur') as HTMLInputElement).value).toBe(
			'Grohe',
		)
	})

	it('commits the value on Enter and not on each keystroke', async () => {
		const { props } = renderToolbar({ filters: { custom_field_key: 'Marque' } })
		const input = screen.getByLabelText('Valeur')

		await userEvent.type(input, 'Grohe')
		expect(props.onChange).not.toHaveBeenCalled()

		await userEvent.type(input, '{Enter}')
		expect(props.onChange).toHaveBeenCalledTimes(1)
		expect(props.onChange).toHaveBeenCalledWith({
			custom_field_value: 'Grohe',
		})
	})

	it('commits the value on blur', async () => {
		const { props } = renderToolbar({ filters: { custom_field_key: 'Marque' } })
		const input = screen.getByLabelText('Valeur')

		await userEvent.type(input, 'Hansgrohe')
		expect(props.onChange).not.toHaveBeenCalled()

		fireEvent.blur(input)
		expect(props.onChange).toHaveBeenCalledTimes(1)
		expect(props.onChange).toHaveBeenCalledWith({
			custom_field_value: 'Hansgrohe',
		})
	})

	it('commits an emptied value as undefined', async () => {
		const { props } = renderToolbar({
			filters: { custom_field_key: 'Marque', custom_field_value: 'Grohe' },
		})
		const input = screen.getByLabelText('Valeur')

		await userEvent.clear(input)
		fireEvent.blur(input)

		expect(props.onChange).toHaveBeenCalledWith({
			custom_field_value: undefined,
		})
	})

	it('does not commit again when the value is left unchanged', async () => {
		const { props } = renderToolbar({
			filters: { custom_field_key: 'Marque', custom_field_value: 'Grohe' },
		})
		const input = screen.getByLabelText('Valeur')

		await userEvent.click(input)
		fireEvent.blur(input)

		expect(props.onChange).not.toHaveBeenCalled()
	})

	it('disables the selects while loading', () => {
		renderToolbar({ isLoading: true })

		for (const label of ['Assigné à', 'Badge', 'Champ']) {
			expect((screen.getByLabelText(label) as HTMLSelectElement).disabled).toBe(
				true,
			)
		}
	})

	it('shows the clear button only when filtered, and calls onClear', async () => {
		const { props, rerender } = renderToolbar()
		expect(
			screen.queryByRole('button', { name: 'Effacer les filtres' }),
		).toBeNull()

		rerender(<SupportToolbar {...props} isFiltered />)
		await userEvent.click(
			screen.getByRole('button', { name: 'Effacer les filtres' }),
		)

		expect(props.onClear).toHaveBeenCalledTimes(1)
	})
})
