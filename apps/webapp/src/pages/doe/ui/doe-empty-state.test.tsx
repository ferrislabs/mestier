import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { chooseCreationMode } from '#/pages/doe/lib/empty-state'
import { DoeEmptyState } from '#/pages/doe/ui/doe-empty-state'
import { renderWithPermissions } from '#/test/with-permissions'

function renderState(models: Array<{ id: string; name: string }>) {
	const onCreate = vi.fn()
	const onCreateStarter = vi.fn()
	renderWithPermissions(
		<DoeEmptyState
			mode={chooseCreationMode(models)}
			models={models}
			selectedModelId={models[0]?.id ?? ''}
			isLoading={false}
			isPending={false}
			error={null}
			onSelectModel={() => {}}
			onCreate={onCreate}
			onCreateStarter={onCreateStarter}
		/>,
	)
	return { onCreate, onCreateStarter }
}

describe('DoeEmptyState', () => {
	it('offers the landscaping starter when there is no model', async () => {
		const user = userEvent.setup()
		const { onCreate, onCreateStarter } = renderState([])

		expect(screen.getByText("Ce projet n'a pas encore de DOE")).toBeDefined()
		expect(screen.queryByRole('button', { name: 'Créer le DOE' })).toBeNull()

		await user.click(
			screen.getByRole('button', { name: 'Créer le modèle Paysagiste' }),
		)
		expect(onCreateStarter).toHaveBeenCalledTimes(1)
		expect(onCreate).not.toHaveBeenCalled()
	})

	it('offers the model picker when the organization has models', async () => {
		const user = userEvent.setup()
		const { onCreate } = renderState([{ id: 'm1', name: 'Paysagiste' }])

		expect(
			screen.queryByRole('button', { name: 'Créer le modèle Paysagiste' }),
		).toBeNull()
		await user.click(screen.getByRole('button', { name: 'Créer le DOE' }))
		expect(onCreate).toHaveBeenCalledTimes(1)
	})
})
