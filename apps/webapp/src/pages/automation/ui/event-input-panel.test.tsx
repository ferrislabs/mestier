import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { EventInputPanel } from '#/pages/automation/ui/event-input-panel'

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

const QUOTE: Schemas.EventDescriptorResponse = {
	label: 'Devis envoyé',
	name: 'quote.sent',
	payload_example: { quote: { id: 'q1' }, from: 'DRAFT', to: 'SENT' },
	subject_kind: 'quote',
	version: 1,
}

const INVOICE: Schemas.EventDescriptorResponse = {
	label: 'Facture payée',
	name: 'invoice.paid',
	payload_example: { invoice: { id: 'i1' } },
	subject_kind: 'invoice',
	version: 1,
}

describe('EventInputPanel', () => {
	it('presents the payload of the event, with its name and version', () => {
		render(<EventInputPanel events={[QUOTE]} onChooseEvent={vi.fn()} />)

		expect(screen.getByText('Devis envoyé')).toBeDefined()
		expect(screen.getByText('quote.sent · version 1')).toBeDefined()
		expect(
			screen.getByRole('tab', { name: 'Contrat TypeScript' }),
		).toBeDefined()
		expect(
			screen.getByRole('tab', { name: 'Exemple de payload' }),
		).toBeDefined()
	})

	it('shows no selector for a single event', () => {
		render(<EventInputPanel events={[QUOTE]} onChooseEvent={vi.fn()} />)

		expect(screen.queryByRole('combobox')).toBeNull()
	})

	it('invites to choose an event when there is none', async () => {
		const user = userEvent.setup()
		const onChooseEvent = vi.fn()
		render(<EventInputPanel events={[]} onChooseEvent={onChooseEvent} />)

		expect(screen.getByText('Aucun événement choisi')).toBeDefined()
		await user.click(
			screen.getByRole('button', { name: 'Choisir l’événement' }),
		)

		expect(onChooseEvent).toHaveBeenCalled()
	})

	it('starts on the first event of the selection when there are several', () => {
		render(
			<EventInputPanel events={[INVOICE, QUOTE]} onChooseEvent={vi.fn()} />,
		)

		expect(screen.getByText('invoice.paid · version 1')).toBeDefined()
		expect(
			screen.getByRole('combobox', { name: 'Événement affiché' }),
		).toBeDefined()
	})

	it('switches to another event of the selection', async () => {
		const user = userEvent.setup()
		render(
			<EventInputPanel events={[INVOICE, QUOTE]} onChooseEvent={vi.fn()} />,
		)

		await user.click(
			screen.getByRole('combobox', { name: 'Événement affiché' }),
		)
		await user.click(
			await screen.findByRole('option', { name: 'Devis envoyé' }),
		)

		expect(screen.getByText('quote.sent · version 1')).toBeDefined()
		expect(screen.queryByText('invoice.paid · version 1')).toBeNull()
	})

	it('falls back to the first event when the selected one leaves the selection', async () => {
		const user = userEvent.setup()
		const { rerender } = render(
			<EventInputPanel events={[INVOICE, QUOTE]} onChooseEvent={vi.fn()} />,
		)
		await user.click(
			screen.getByRole('combobox', { name: 'Événement affiché' }),
		)
		await user.click(
			await screen.findByRole('option', { name: 'Devis envoyé' }),
		)

		rerender(<EventInputPanel events={[INVOICE]} onChooseEvent={vi.fn()} />)

		expect(screen.getByText('invoice.paid · version 1')).toBeDefined()
	})
})
