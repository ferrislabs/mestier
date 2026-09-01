import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it } from 'vitest'
import { EditablePaperField } from './editable-paper-field'

Element.prototype.scrollIntoView ??= () => {}
Element.prototype.hasPointerCapture ??= () => false
Element.prototype.releasePointerCapture ??= () => {}

describe('EditablePaperField', () => {
	it('prints the value plainly until it is opened', () => {
		render(
			<EditablePaperField label="Objet" renderEditor={() => <input />}>
				<h1>Rénovation salle de bain</h1>
			</EditablePaperField>,
		)

		expect(screen.getByText('Rénovation salle de bain')).toBeDefined()
		expect(screen.queryByRole('textbox')).toBeNull()
	})

	it('opens the editor in a popover on click, labelled by the field', async () => {
		const user = userEvent.setup()
		render(
			<EditablePaperField label="Objet" renderEditor={() => <input />}>
				<h1>Rénovation salle de bain</h1>
			</EditablePaperField>,
		)

		await user.click(screen.getByRole('button', { name: /modifier.*objet/i }))

		expect(screen.getByText('Objet')).toBeDefined()
		expect(screen.getByRole('textbox')).toBeDefined()
	})

	it('lets the editor close itself', async () => {
		const user = userEvent.setup()
		render(
			<EditablePaperField
				label="Objet"
				renderEditor={(close) => (
					<button type="button" onClick={close}>
						Fermer
					</button>
				)}
			>
				<h1>Rénovation salle de bain</h1>
			</EditablePaperField>,
		)

		await user.click(screen.getByRole('button', { name: /modifier.*objet/i }))
		await user.click(screen.getByRole('button', { name: 'Fermer' }))

		expect(screen.queryByRole('button', { name: 'Fermer' })).toBeNull()
	})
})
