import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeAll, describe, expect, it, vi } from 'vitest'
import { MASCULINE_ARCHIVE_OPTIONS } from '#/pages/cursus/shared/lib/list-filters'
import { MultiFilterButton } from '#/pages/cursus/shared/ui/multi-filter-button'
import { installRadixPolyfills } from '#/pages/support/feature/support-test-kit'

beforeAll(installRadixPolyfills)

const OPTIONS = [
	...MASCULINE_ARCHIVE_OPTIONS,
	{ value: 'other', label: 'Autres' },
] as const

function renderButton(selected: readonly string[], onChange = vi.fn()) {
	render(
		<MultiFilterButton
			name="Statut"
			options={OPTIONS}
			selected={selected}
			onChange={onChange}
		/>,
	)
	return onChange
}

describe('MultiFilterButton', () => {
	it('shows the filter name when nothing is selected', () => {
		renderButton([])

		const button = screen.getByRole('button', { name: 'Statut' })
		expect(button.textContent).toBe('Statut')
	})

	it('shows the option label when one is selected', () => {
		renderButton(['active'])

		expect(screen.getByRole('button', { name: 'Statut' }).textContent).toBe(
			'Actifs',
		)
	})

	it('shows the filter name and a count when several are selected', () => {
		renderButton(['active', 'archived'])

		expect(screen.getByRole('button', { name: 'Statut' }).textContent).toBe(
			'Statut2',
		)
	})

	it('adds an option to the selection in option order', {
		timeout: 15_000,
	}, async () => {
		const user = userEvent.setup()
		const onChange = renderButton(['other'])

		await user.click(screen.getByRole('button', { name: 'Statut' }))
		await user.click(screen.getByRole('menuitemcheckbox', { name: 'Actifs' }))

		expect(onChange).toHaveBeenCalledWith(['active', 'other'])
	})

	it('removes an option from the selection', {
		timeout: 15_000,
	}, async () => {
		const user = userEvent.setup()
		const onChange = renderButton(['active', 'archived'])

		await user.click(screen.getByRole('button', { name: 'Statut' }))
		await user.click(screen.getByRole('menuitemcheckbox', { name: 'Actifs' }))
		expect(onChange).toHaveBeenLastCalledWith(['archived'])
	})
})
