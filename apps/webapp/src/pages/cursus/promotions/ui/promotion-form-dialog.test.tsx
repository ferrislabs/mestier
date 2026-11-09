import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { EMPTY_PROMOTION_FORM } from '#/pages/cursus/promotions/lib/promotion-form'
import { PromotionFormDialog } from '#/pages/cursus/promotions/ui/promotion-form-dialog'

function renderDialog(onManageTrainings?: () => void) {
	render(
		<PromotionFormDialog
			open
			title="Nouvelle promotion"
			submitLabel="Créer la promotion"
			initial={EMPTY_PROMOTION_FORM}
			trainings={[]}
			isPending={false}
			error={null}
			onOpenChange={vi.fn()}
			onSubmit={vi.fn()}
			onManageTrainings={onManageTrainings}
		/>,
	)
}

describe('PromotionFormDialog', () => {
	it('points to the trainings settings when there is no training yet', async () => {
		const onManageTrainings = vi.fn()
		renderDialog(onManageTrainings)

		expect(screen.getByText(/Aucune formation pour l’instant/)).toBeTruthy()
		await userEvent.click(
			screen.getByRole('button', {
				name: 'Gérer les formations dans Paramètres › Cursus',
			}),
		)

		expect(onManageTrainings).toHaveBeenCalledOnce()
	})

	it('shows no settings link without a handler', () => {
		renderDialog()

		expect(
			screen.queryByRole('button', { name: /Gérer les formations/ }),
		).toBeNull()
	})
})
