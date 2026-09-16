import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { CredentialPickerField } from '#/pages/automation/ui/credential-picker-field'

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

function credential(
	id: string,
	name: string,
	origin: 'supplied' | 'generated' = 'supplied',
): Schemas.CredentialResponse {
	return {
		id,
		name,
		kind: 'bearer_token',
		origin,
		organization_id: 'org-1',
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
	}
}

describe('CredentialPickerField', () => {
	it('offers every given credential by name and reports the chosen one', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<CredentialPickerField
				label="Identification"
				htmlFor="credential-picker"
				credentials={[
					credential('c1', 'Odoo prod'),
					credential('c2', 'Odoo recette'),
				]}
				value={null}
				error={null}
				onChange={onChange}
				onCreateNew={vi.fn()}
			/>,
		)

		await user.click(screen.getByRole('combobox', { name: 'Identification' }))
		await user.click(
			await screen.findByRole('option', { name: 'Odoo recette' }),
		)

		expect(onChange).toHaveBeenCalledWith('c2')
	})

	it('shows the currently selected credential name', () => {
		render(
			<CredentialPickerField
				label="Identification"
				htmlFor="credential-picker"
				credentials={[credential('c1', 'Odoo prod')]}
				value="c1"
				error={null}
				onChange={vi.fn()}
				onCreateNew={vi.fn()}
			/>,
		)

		expect(screen.getByText('Odoo prod')).toBeDefined()
	})

	it('says plainly when there is nothing to pick from', async () => {
		const user = userEvent.setup()
		render(
			<CredentialPickerField
				label="Identification"
				htmlFor="credential-picker"
				credentials={[]}
				value={null}
				error={null}
				onChange={vi.fn()}
				onCreateNew={vi.fn()}
			/>,
		)

		await user.click(screen.getByRole('combobox', { name: 'Identification' }))

		expect(
			await screen.findByText('Aucune identification disponible'),
		).toBeDefined()
	})

	it('calls onCreateNew when the create button is pressed', async () => {
		const user = userEvent.setup()
		const onCreateNew = vi.fn()
		render(
			<CredentialPickerField
				label="Identification"
				htmlFor="credential-picker"
				credentials={[credential('cred-1', 'Odoo prod')]}
				value={null}
				error={null}
				onChange={vi.fn()}
				onCreateNew={onCreateNew}
			/>,
		)

		await user.click(
			screen.getByRole('button', { name: /Nouvelle identification/ }),
		)

		expect(onCreateNew).toHaveBeenCalledTimes(1)
	})

	it('shows the validation error next to the picker', () => {
		render(
			<CredentialPickerField
				label="Identification"
				htmlFor="credential-picker"
				credentials={[]}
				value={null}
				error="Identifiant de credential manquant"
				onChange={vi.fn()}
				onCreateNew={vi.fn()}
			/>,
		)

		expect(screen.getByText('Identifiant de credential manquant')).toBeDefined()
	})
})

describe('CredentialPickerField — when the organization has none', () => {
	it('says so on the closed select rather than promising a choice', () => {
		render(
			<CredentialPickerField
				label="Identification"
				htmlFor="credential-picker"
				credentials={[]}
				value={null}
				error={null}
				onChange={vi.fn()}
				onCreateNew={vi.fn()}
			/>,
		)

		expect(screen.getByText('Aucune identification')).toBeDefined()
		expect(screen.queryByText('Choisir une identification…')).toBeNull()
	})

	it('offers creating one as a named action, not a bare icon', async () => {
		const user = userEvent.setup()
		const onCreateNew = vi.fn()
		render(
			<CredentialPickerField
				label="Identification"
				htmlFor="credential-picker"
				credentials={[]}
				value={null}
				error={null}
				onChange={vi.fn()}
				onCreateNew={onCreateNew}
			/>,
		)

		await user.click(screen.getByRole('button', { name: /Créer/ }))

		expect(onCreateNew).toHaveBeenCalled()
	})
})
