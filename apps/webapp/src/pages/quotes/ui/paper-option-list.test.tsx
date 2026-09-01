import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { PaperOptionList } from './paper-option-list'

describe('PaperOptionList', () => {
	it('lists every option, plainly, with no floating popper involved', () => {
		render(
			<PaperOptionList
				ariaLabel="Client"
				value="c1"
				options={[
					{ value: 'c1', label: 'Menuiserie Dupont' },
					{ value: 'c2', label: 'Atelier Bois & Co' },
				]}
				onChange={vi.fn()}
			/>,
		)

		expect(screen.getByRole('listbox', { name: 'Client' })).toBeDefined()
		expect(
			screen.getByRole('option', { name: 'Menuiserie Dupont' }),
		).toBeDefined()
		expect(
			screen.getByRole('option', { name: 'Atelier Bois & Co' }),
		).toBeDefined()
	})

	it('marks the current value as selected', () => {
		render(
			<PaperOptionList
				ariaLabel="Client"
				value="c2"
				options={[
					{ value: 'c1', label: 'Menuiserie Dupont' },
					{ value: 'c2', label: 'Atelier Bois & Co' },
				]}
				onChange={vi.fn()}
			/>,
		)

		expect(
			screen.getByRole('option', { name: 'Menuiserie Dupont' }),
		).toHaveProperty('ariaSelected', 'false')
		expect(
			screen.getByRole('option', { name: 'Atelier Bois & Co' }),
		).toHaveProperty('ariaSelected', 'true')
	})

	it('reports the picked value on a plain click — no dropdown to open first', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<PaperOptionList
				ariaLabel="Client"
				value=""
				options={[{ value: 'c1', label: 'Menuiserie Dupont' }]}
				onChange={onChange}
			/>,
		)

		await user.click(screen.getByRole('option', { name: 'Menuiserie Dupont' }))

		expect(onChange).toHaveBeenCalledWith('c1')
	})

	it('explains an empty list instead of rendering nothing', () => {
		render(
			<PaperOptionList
				ariaLabel="Adresse de facturation"
				value=""
				options={[]}
				onChange={vi.fn()}
				emptyLabel="Aucune adresse pour ce client."
			/>,
		)

		expect(screen.getByText('Aucune adresse pour ce client.')).toBeDefined()
		expect(screen.queryByRole('listbox')).toBeNull()
	})
})
