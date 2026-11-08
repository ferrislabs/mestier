import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { MemberFormValues } from '#/pages/hr/types'
import { MemberCostDialog } from '#/pages/hr/ui/rh/members/member-cost-dialog'
import { renderWithRouter } from '#/test/render-with-router'

const EMPTY: MemberFormValues = {
	lastName: '',
	firstName: '',
	hourlyRate: '',
	isSalaried: false,
	monthlyCost: '',
}

function renderDialog(
	overrides: Partial<{ open: boolean; values: MemberFormValues }> = {},
) {
	const handlers = {
		onChange: vi.fn(),
		onSubmit: vi.fn(),
		onOpenChange: vi.fn(),
	}
	return renderWithRouter(
		<MemberCostDialog
			idPrefix="test"
			open={overrides.open ?? true}
			title="Nouveau membre"
			description="Description"
			submitLabel="Ajouter"
			values={overrides.values ?? EMPTY}
			isPending={false}
			{...handlers}
		/>,
	).then((result) => ({ ...result, ...handlers }))
}

describe('MemberCostDialog', () => {
	it('shows no fields while closed', async () => {
		await renderDialog({ open: false })

		expect(screen.queryByLabelText('Nom')).toBeNull()
	})

	it('submits through the binding', async () => {
		const user = userEvent.setup()
		const { onSubmit } = await renderDialog()

		await user.click(screen.getByRole('button', { name: 'Ajouter' }))

		expect(onSubmit).toHaveBeenCalled()
	})

	it('clears the hourly rate when "Salarié" is toggled on', async () => {
		const user = userEvent.setup()
		const { onChange } = await renderDialog({
			values: { ...EMPTY, hourlyRate: '35' },
		})

		await user.click(screen.getByRole('switch', { name: /Salarié/ }))

		expect(onChange).toHaveBeenCalledWith({
			isSalaried: true,
			hourlyRate: '',
			monthlyCost: '',
		})
	})

	it('disables the hourly rate field and asks for the monthly cost while "Salarié" is on', async () => {
		await renderDialog({ values: { ...EMPTY, isSalaried: true } })

		expect(
			(screen.getByLabelText('Taux horaire') as HTMLInputElement).disabled,
		).toBe(true)
		expect(screen.getByLabelText('Coût employeur mensuel')).toBeDefined()
	})
})
