import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeAll, describe, expect, it, vi } from 'vitest'
import {
	type ContractFormValues,
	EMPTY_CONTRACT_FORM,
} from '#/pages/cursus/triads/lib/contract-form'
import { ContractFormDialog } from '#/pages/cursus/triads/ui/contract-form-dialog'
import { installRadixPolyfills } from '#/pages/support/feature/support-test-kit'

beforeAll(installRadixPolyfills)

const EXISTING: ContractFormValues = {
	hostCompanyId: 'host-1',
	kind: 'APPRENTICESHIP',
	number: 'C-42',
	comment: 'Visite annuelle',
	startDate: '2026-09-01',
	endDate: '2028-08-31',
}

function renderDialog(initial: ContractFormValues, onSubmit = vi.fn()) {
	render(
		<ContractFormDialog
			open
			title="Modifier le contrat"
			submitLabel="Enregistrer"
			initial={initial}
			hostCompanies={[{ value: 'host-1', label: 'Atelier Bois' }]}
			isPending={false}
			error={null}
			onOpenChange={vi.fn()}
			onSubmit={onSubmit}
		/>,
	)
	return onSubmit
}

function currentStep() {
	return document.querySelector('[aria-current="step"]')?.textContent ?? ''
}

describe('ContractFormDialog', () => {
	it('opens an existing contract prefilled at the first step and submits it unchanged', {
		timeout: 15_000,
	}, async () => {
		const user = userEvent.setup()
		const onSubmit = renderDialog(EXISTING)

		expect(currentStep()).toContain('Contrat')
		expect(
			(screen.getByLabelText('Numéro de contrat') as HTMLInputElement).value,
		).toBe('C-42')
		await user.click(screen.getByRole('button', { name: 'Suivant' }))
		expect(
			(screen.getByLabelText(/Date de début/) as HTMLInputElement).value,
		).toBe('2026-09-01')
		await user.click(screen.getByRole('button', { name: 'Suivant' }))
		expect(currentStep()).toContain('Commentaire')
		expect(onSubmit).not.toHaveBeenCalled()
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		expect(onSubmit).toHaveBeenCalledWith(EXISTING)
	})

	it('stays on the contract step until a company is chosen', {
		timeout: 15_000,
	}, async () => {
		const user = userEvent.setup()
		renderDialog(EMPTY_CONTRACT_FORM)

		await user.click(screen.getByRole('button', { name: 'Suivant' }))

		expect(screen.getByText('Choisissez une entreprise.')).toBeTruthy()
		expect(currentStep()).toContain('Contrat')
	})

	it('stays on the period step when the end precedes the start', {
		timeout: 15_000,
	}, async () => {
		const user = userEvent.setup()
		renderDialog({ ...EXISTING, endDate: '2026-08-01' })

		await user.click(screen.getByRole('button', { name: 'Suivant' }))
		expect(screen.queryByText('La fin doit suivre le début.')).toBeNull()
		await user.click(screen.getByRole('button', { name: 'Suivant' }))

		expect(screen.getByText('La fin doit suivre le début.')).toBeTruthy()
		expect(currentStep()).toContain('Période')
	})
})
