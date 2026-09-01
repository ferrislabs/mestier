import { fireEvent, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, describe, expect, it } from 'vitest'
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

	describe('a nested Select', () => {
		let foreignNode: HTMLElement | null = null

		afterEach(() => {
			foreignNode?.remove()
			foreignNode = null
		})

		/**
		 * A Select's dropdown renders in its own Radix popper, positioned
		 * outside this popover's DOM node — exactly like the option list a
		 * customer picker opens. Clicking an option there must not read as a
		 * click outside the popover: that race is what made picking a client
		 * silently do nothing (the popover closed before the Select could
		 * commit the value).
		 */
		it('stays open for a click that lands in another Radix popper', async () => {
			render(
				<EditablePaperField label="Objet" renderEditor={() => <input />}>
					<h1>Rénovation salle de bain</h1>
				</EditablePaperField>,
			)
			await userEvent
				.setup()
				.click(screen.getByRole('button', { name: /modifier.*objet/i }))
			expect(screen.getByRole('textbox')).toBeDefined()

			foreignNode = document.createElement('div')
			foreignNode.setAttribute('data-radix-popper-content-wrapper', '')
			document.body.append(foreignNode)
			fireEvent.pointerDown(foreignNode)

			expect(screen.getByRole('textbox')).toBeDefined()
		})

		it('still closes for a click truly outside everything', async () => {
			render(
				<EditablePaperField label="Objet" renderEditor={() => <input />}>
					<h1>Rénovation salle de bain</h1>
				</EditablePaperField>,
			)
			await userEvent
				.setup()
				.click(screen.getByRole('button', { name: /modifier.*objet/i }))
			expect(screen.getByRole('textbox')).toBeDefined()

			foreignNode = document.createElement('div')
			document.body.append(foreignNode)
			fireEvent.pointerDown(foreignNode)

			expect(screen.queryByRole('textbox')).toBeNull()
		})
	})
})
