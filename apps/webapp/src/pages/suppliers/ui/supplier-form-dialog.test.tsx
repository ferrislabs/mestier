import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { emptySupplierValues } from '#/pages/suppliers/lib/supplier-form'
import { SupplierFormDialog } from '#/pages/suppliers/ui/supplier-form-dialog'

const CATEGORIES = [{ id: 'cat-materiaux', name: 'Matériaux', hue: 60 }]

function renderDialog(
	props: Partial<Parameters<typeof SupplierFormDialog>[0]> = {},
) {
	const onSubmit = vi.fn()
	const onToggleArchive = vi.fn()
	render(
		<SupplierFormDialog
			open
			editingName={null}
			initialValues={emptySupplierValues()}
			categories={CATEGORIES}
			archived={null}
			toPay={0}
			isPending={false}
			error={null}
			onOpenChange={() => {}}
			onSubmit={onSubmit}
			onToggleArchive={onToggleArchive}
			{...props}
		/>,
	)
	return { onSubmit, onToggleArchive }
}

describe('SupplierFormDialog', () => {
	it('refuses a blank name', async () => {
		const user = userEvent.setup()
		const { onSubmit } = renderDialog()

		await user.type(screen.getAllByLabelText(/^Nom/)[0] as HTMLElement, '   ')
		await user.click(
			screen.getByRole('button', { name: 'Créer le fournisseur' }),
		)

		expect((await screen.findByRole('alert')).textContent).toBe(
			'Le nom est obligatoire.',
		)
		expect(onSubmit).not.toHaveBeenCalled()
	})

	it('submits the values when the name is filled', async () => {
		const user = userEvent.setup()
		const { onSubmit } = renderDialog()

		await user.type(
			screen.getAllByLabelText(/^Nom/)[0] as HTMLElement,
			'Pépinières Martin',
		)
		await user.type(screen.getByLabelText('IBAN'), 'FR7610')
		await user.type(screen.getByLabelText('Téléphone'), '0102030405')
		await user.click(
			screen.getByRole('button', { name: 'Créer le fournisseur' }),
		)

		await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1))
		const values = onSubmit.mock.calls[0]?.[0]
		expect(values).toMatchObject({ name: 'Pépinières Martin', iban: 'FR7610' })
		expect(values.contacts[0]).toMatchObject({ phone: '0102030405' })
	})

	it('labels the registration number SIRET', () => {
		renderDialog()
		expect(screen.getByLabelText('SIRET')).toBeTruthy()
	})

	it('adds and removes contacts', async () => {
		const user = userEvent.setup()
		renderDialog()

		expect(
			screen.queryByRole('button', { name: /Retirer le contact/ }),
		).toBeNull()
		await user.click(screen.getByRole('button', { name: 'Ajouter un contact' }))
		expect(screen.getAllByLabelText('Téléphone')).toHaveLength(2)

		await user.click(
			screen.getByRole('button', { name: 'Retirer le contact 2' }),
		)
		expect(screen.getAllByLabelText('Téléphone')).toHaveLength(1)
	})

	it('adds an address', async () => {
		const user = userEvent.setup()
		renderDialog()

		await user.click(
			screen.getByRole('button', { name: 'Ajouter une adresse' }),
		)
		expect(screen.getAllByLabelText('Rue')).toHaveLength(2)
	})

	it('offers to archive an existing supplier', async () => {
		const user = userEvent.setup()
		const { onToggleArchive } = renderDialog({
			editingName: 'Pointp',
			archived: false,
		})

		await user.click(screen.getByRole('button', { name: 'Archiver' }))
		expect(onToggleArchive).toHaveBeenCalledTimes(1)
	})

	it('offers to reactivate an archived supplier', () => {
		renderDialog({ editingName: 'Pointp', archived: true })
		expect(screen.getByRole('button', { name: 'Réactiver' })).toBeTruthy()
	})
})
