import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { CredentialPicker } from '#/pages/automation/ui/credential-picker'

// jsdom has no pointer-capture APIs or `scrollIntoView`; Radix's `Select`
// calls both when an item is actually selected. Scoped to this file — see
// `field-form.test.tsx`'s identical comment for why this is not in
// `vitest.setup.ts`.
for (const method of [
	'hasPointerCapture',
	'setPointerCapture',
	'releasePointerCapture',
	'scrollIntoView',
] as const) {
	if (typeof Element.prototype[method] !== 'function') {
		Element.prototype[method] = (() => false) as never
	}
}

describe('CredentialPicker — options', () => {
	it('offers exactly the passed-in options, already filtered by the caller', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<CredentialPicker
				options={[{ id: 'cred-1', name: 'Odoo production' }]}
				value={null}
				onChange={onChange}
				onCreateNew={vi.fn()}
			/>,
		)

		await user.click(screen.getByRole('combobox'))
		await user.click(screen.getByRole('option', { name: 'Odoo production' }))

		expect(onChange).toHaveBeenCalledWith('cred-1')
	})

	it('shows a message instead of an empty list when there is nothing compatible', async () => {
		const user = userEvent.setup()
		render(
			<CredentialPicker
				options={[]}
				value={null}
				onChange={vi.fn()}
				onCreateNew={vi.fn()}
			/>,
		)

		await user.click(screen.getByRole('combobox'))

		expect(screen.getByText('Aucune identification compatible')).toBeDefined()
	})
})

describe('CredentialPicker — inline creation', () => {
	it('calls onCreateNew without touching the select', async () => {
		const user = userEvent.setup()
		const onCreateNew = vi.fn()
		render(
			<CredentialPicker
				options={[]}
				value={null}
				onChange={vi.fn()}
				onCreateNew={onCreateNew}
			/>,
		)

		await user.click(
			screen.getByRole('button', { name: 'Nouvelle identification' }),
		)

		expect(onCreateNew).toHaveBeenCalled()
	})
})
