import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { credential as fixture } from '#/pages/automation/test/credential-fixtures'
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
	status: Schemas.CredentialStatusResponse = 'valid',
): Schemas.CredentialResponse {
	return fixture({ id, name, origin, status })
}

const EXPIRED_ALERT =
	'Cette identification est expirée. Renouvelez-la depuis Identifications.'

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

	it('invites to choose, without an ellipsis, while nothing is selected', () => {
		render(
			<CredentialPickerField
				label="Identification"
				htmlFor="credential-picker"
				credentials={[credential('c1', 'Odoo prod')]}
				value={null}
				error={null}
				onChange={vi.fn()}
				onCreateNew={vi.fn()}
			/>,
		)

		expect(screen.getByText('Choisir une identification')).toBeDefined()
		expect(screen.queryByText('Choisir une identification…')).toBeNull()
	})

	it('is a plain list: no search box, no empty-result message', async () => {
		const user = userEvent.setup()
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
				onChange={vi.fn()}
				onCreateNew={vi.fn()}
			/>,
		)

		await user.click(screen.getByRole('combobox', { name: 'Identification' }))
		await screen.findByRole('option', { name: 'Odoo prod' })

		expect(screen.queryByPlaceholderText(/Rechercher/)).toBeNull()
		expect(screen.queryByText('Aucun résultat')).toBeNull()
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

	it('shows a hint under the field when given one', () => {
		render(
			<CredentialPickerField
				label="Identification"
				htmlFor="credential-picker"
				credentials={[credential('c1', 'Odoo prod')]}
				value={null}
				error={null}
				hint="La spec ne déclare pas d’authentification."
				onChange={vi.fn()}
				onCreateNew={vi.fn()}
			/>,
		)

		expect(
			screen.getByText('La spec ne déclare pas d’authentification.'),
		).toBeDefined()
	})
})

describe('CredentialPickerField — creating one from the node', () => {
	it('offers it as a named action under the list, even when the list is not empty', async () => {
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
			screen.getByRole('button', { name: /Créer une identification/ }),
		)

		expect(onCreateNew).toHaveBeenCalledTimes(1)
	})
})

describe('CredentialPickerField — when the organization has none', () => {
	it('says so on the closed trigger and leaves it inert', () => {
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
		expect(screen.queryByText('Choisir une identification')).toBeNull()
		expect((screen.getByRole('combobox') as HTMLButtonElement).disabled).toBe(
			true,
		)
	})

	it('still offers creating one', async () => {
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

describe('CredentialPickerField — when a credential is optional', () => {
	it('offers going back to none once one is chosen', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<CredentialPickerField
				label="Identification"
				htmlFor="credential-picker"
				credentials={[credential('cred-1', 'Odoo prod')]}
				value="cred-1"
				error={null}
				optional
				onChange={onChange}
				onCreateNew={vi.fn()}
			/>,
		)

		await user.click(screen.getByRole('combobox'))
		await user.click(
			await screen.findByRole('option', { name: /Aucune identification/ }),
		)

		expect(onChange).toHaveBeenCalledWith(null)
	})

	it('offers no such choice when the connector demands one', async () => {
		const user = userEvent.setup()
		render(
			<CredentialPickerField
				label="Identification"
				htmlFor="credential-picker"
				credentials={[credential('cred-1', 'Odoo prod')]}
				value="cred-1"
				error={null}
				onChange={vi.fn()}
				onCreateNew={vi.fn()}
			/>,
		)

		await user.click(screen.getByRole('combobox'))

		expect(
			screen.queryByRole('option', { name: /Aucune identification/ }),
		).toBeNull()
	})
})

describe('CredentialPickerField — status of the credentials', () => {
	const OLD = credential('c-old', 'Odoo prod', 'supplied', 'expired')
	const NEW = credential('c-new', 'Odoo recette')
	const OFFLINE = credential('c-off', 'Google', 'supplied', 'not_connected')

	it('suffixes an expired credential with « (expirée) » in the list', async () => {
		const user = userEvent.setup()
		render(
			<CredentialPickerField
				label="Identification"
				htmlFor="credential-picker"
				credentials={[OLD, NEW]}
				value={null}
				error={null}
				onChange={vi.fn()}
				onCreateNew={vi.fn()}
			/>,
		)

		await user.click(screen.getByRole('combobox', { name: 'Identification' }))

		expect(
			await screen.findByRole('option', { name: 'Odoo prod (expirée)' }),
		).toBeDefined()
		expect(screen.getByRole('option', { name: 'Odoo recette' })).toBeDefined()
	})

	it('suffixes a credential whose account is not connected', async () => {
		const user = userEvent.setup()
		render(
			<CredentialPickerField
				label="Identification"
				htmlFor="credential-picker"
				credentials={[OFFLINE, NEW]}
				value={null}
				error={null}
				onChange={vi.fn()}
				onCreateNew={vi.fn()}
			/>,
		)

		await user.click(screen.getByRole('combobox', { name: 'Identification' }))

		expect(
			await screen.findByRole('option', { name: 'Google (non connectée)' }),
		).toBeDefined()
	})

	it('warns under the field when the selected credential is expired', () => {
		render(
			<CredentialPickerField
				label="Identification"
				htmlFor="credential-picker"
				credentials={[OLD]}
				value="c-old"
				error={null}
				onChange={vi.fn()}
				onCreateNew={vi.fn()}
			/>,
		)

		expect(screen.getByText(EXPIRED_ALERT)).toBeDefined()
	})

	it('warns under the field when the selected credential is not connected', () => {
		render(
			<CredentialPickerField
				label="Identification"
				htmlFor="credential-picker"
				credentials={[OFFLINE]}
				value="c-off"
				error={null}
				onChange={vi.fn()}
				onCreateNew={vi.fn()}
			/>,
		)

		expect(
			screen.getByText(
				'Cette identification n’est pas connectée. Connectez le compte depuis Identifications.',
			),
		).toBeDefined()
	})

	it('does not warn when the selected credential is valid', () => {
		render(
			<CredentialPickerField
				label="Identification"
				htmlFor="credential-picker"
				credentials={[OLD, NEW]}
				value="c-new"
				error={null}
				onChange={vi.fn()}
				onCreateNew={vi.fn()}
			/>,
		)

		expect(screen.queryByText(EXPIRED_ALERT)).toBeNull()
		expect(screen.queryByText(/n’est pas connectée/)).toBeNull()
	})
})

describe('CredentialPickerField — the warning', () => {
	const WARNING =
		'Sans identification, toute personne qui connaît cette URL peut déclencher ce workflow.'

	it('shows the warning right under the selector, before the other lines', () => {
		render(
			<CredentialPickerField
				label="Identification"
				htmlFor="credential-picker"
				credentials={[credential('c1', 'Odoo prod')]}
				value={null}
				error={null}
				optional
				hint="Types acceptés : Bearer."
				warning={WARNING}
				onChange={vi.fn()}
				onCreateNew={vi.fn()}
			/>,
		)

		const warning = screen.getByText(WARNING)
		const select = screen.getByRole('combobox', { name: 'Identification' })
		expect(
			select.compareDocumentPosition(warning) &
				Node.DOCUMENT_POSITION_FOLLOWING,
		).toBeTruthy()
		expect(
			warning.compareDocumentPosition(
				screen.getByRole('button', { name: 'Créer une identification' }),
			) & Node.DOCUMENT_POSITION_FOLLOWING,
		).toBeTruthy()
		expect(
			warning.compareDocumentPosition(
				screen.getByText('Types acceptés : Bearer.'),
			) & Node.DOCUMENT_POSITION_FOLLOWING,
		).toBeTruthy()
	})

	it('shows no warning when none is given', () => {
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

		expect(screen.queryByText(/Sans identification/)).toBeNull()
	})
})
