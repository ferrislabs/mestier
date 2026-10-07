import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import {
	buildSupplierRequest,
	EMPTY_SUPPLIER_FORM,
	firstAddressLine,
	validateSupplierName,
} from '#/pages/suppliers/types'
import { SupplierFormDialog } from '#/pages/suppliers/ui/supplier-form-dialog'

function renderDialog(onSubmit = vi.fn()) {
	render(
		<SupplierFormDialog
			open
			editingName={null}
			initialValues={EMPTY_SUPPLIER_FORM}
			isPending={false}
			error={null}
			onOpenChange={() => {}}
			onSubmit={onSubmit}
		/>,
	)
	return onSubmit
}

describe('SupplierFormDialog', () => {
	it('refuses a blank name', async () => {
		const user = userEvent.setup()
		const onSubmit = renderDialog()

		await user.type(screen.getByLabelText('Nom'), '   ')
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		expect((await screen.findByRole('alert')).textContent).toBe(
			'Le nom est obligatoire.',
		)
		expect(onSubmit).not.toHaveBeenCalled()
	})

	it('submits the values when the name is filled', async () => {
		const user = userEvent.setup()
		const onSubmit = renderDialog()

		await user.type(screen.getByLabelText('Nom'), 'Pépinières Martin')
		await user.type(screen.getByLabelText('Téléphone'), '0102030405')
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1))
		expect(onSubmit.mock.calls[0]?.[0]).toMatchObject({
			name: 'Pépinières Martin',
			phone: '0102030405',
		})
	})
})

describe('supplier helpers', () => {
	it('validates the name', () => {
		expect(validateSupplierName('  ')).toBeDefined()
		expect(validateSupplierName('A')).toBeUndefined()
	})

	it('builds a request with blanks as null', () => {
		expect(
			buildSupplierRequest({
				...EMPTY_SUPPLIER_FORM,
				name: ' Martin ',
				address: '1 rue X\n75000 Paris',
			}),
		).toEqual({
			name: 'Martin',
			registration_number: null,
			vat_number: null,
			address: '1 rue X\n75000 Paris',
			phone: null,
			email: null,
		})
	})

	it('keeps the first address line', () => {
		expect(firstAddressLine('1 rue X\n75000 Paris')).toBe('1 rue X')
		expect(firstAddressLine(null)).toBe('')
	})
})
