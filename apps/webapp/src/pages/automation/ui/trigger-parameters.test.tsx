import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { TriggerParameters } from '#/pages/automation/ui/trigger-parameters'

const EVENTS: Schemas.EventDescriptorResponse[] = [
	{
		label: 'Devis envoyé',
		name: 'quote.sent',
		payload_example: {},
		subject_kind: 'quote',
		version: 1,
	},
	{
		label: 'Facture payée',
		name: 'invoice.paid',
		payload_example: {},
		subject_kind: 'invoice',
		version: 1,
	},
]

function trigger(kind: Schemas.TriggerKindDto): Schemas.PlacedTriggerDto {
	return { id: 't1', kind }
}

function renderParameters(
	kind: Schemas.TriggerKindDto,
	handlers: {
		onChooseEvent?: (replacing: string | null) => void
		onRemoveEvent?: (name: string) => void
	} = {},
) {
	return render(
		<TriggerParameters
			trigger={trigger(kind)}
			events={EVENTS}
			onChooseEvent={handlers.onChooseEvent ?? vi.fn()}
			onRemoveEvent={handlers.onRemoveEvent ?? vi.fn()}
		/>,
	)
}

describe('TriggerParameters — manual', () => {
	it('renders nothing, as the empty parameters of the prototype', () => {
		const { container } = renderParameters('Manual')

		expect(container.firstChild).toBeNull()
	})
})

describe('TriggerParameters — an event', () => {
	it('shows the event, its name and its version, with the hint of the field', () => {
		renderParameters({ Events: ['quote.sent'] })

		expect(screen.getByText('Événement')).toBeDefined()
		expect(screen.getByText('Devis envoyé')).toBeDefined()
		expect(screen.getByText('quote.sent · version 1')).toBeDefined()
		expect(
			screen.getByText(
				'Le workflow reçoit le payload contractuel de cet événement.',
			),
		).toBeDefined()
	})

	it('changes the only event with the button of the prototype', async () => {
		const user = userEvent.setup()
		const onChooseEvent = vi.fn()
		renderParameters({ Events: ['quote.sent'] }, { onChooseEvent })

		await user.click(
			screen.getByRole('button', { name: 'Changer l’événement' }),
		)

		expect(onChooseEvent).toHaveBeenCalledWith('quote.sent')
	})

	it('names the event on each button once there are several', () => {
		renderParameters({ Events: ['quote.sent', 'invoice.paid'] })

		expect(
			screen.getByRole('button', { name: 'Changer l’événement quote.sent' }),
		).toBeDefined()
		expect(
			screen.getByRole('button', { name: 'Retirer l’événement invoice.paid' }),
		).toBeDefined()
		expect(
			screen.getByRole('button', { name: 'Ajouter un événement' }),
		).toBeDefined()
	})

	it('offers no removal while a single event remains', () => {
		renderParameters({ Events: ['quote.sent'] })

		expect(screen.queryByRole('button', { name: /Retirer/ })).toBeNull()
	})

	it('invites to choose when no event is set', async () => {
		const user = userEvent.setup()
		const onChooseEvent = vi.fn()
		renderParameters({ Events: [] }, { onChooseEvent })

		expect(screen.getByText('Aucun événement choisi')).toBeDefined()
		await user.click(
			screen.getByRole('button', { name: 'Choisir l’événement' }),
		)

		expect(onChooseEvent).toHaveBeenCalledWith(null)
	})
})
