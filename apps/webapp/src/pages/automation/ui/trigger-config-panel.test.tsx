import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { TriggerConfigPanel } from '#/pages/automation/ui/trigger-config-panel'

function event(
	name: string,
	label: string = name,
): Schemas.EventDescriptorResponse {
	return {
		name,
		label,
		subject_kind: name.split('.')[0] ?? name,
		version: 1,
		payload_example: {},
	}
}

const EVENTS = [
	event('quote.accepted', 'Devis accepté'),
	event('quote.sent', 'Devis envoyé'),
	event('invoice.paid', 'Facture payée'),
]

function trigger(
	kind: Schemas.TriggerKindDto,
	id = 't1',
): Schemas.PlacedTriggerDto {
	return { id, kind }
}

function baseProps(
	overrides: Partial<Parameters<typeof TriggerConfigPanel>[0]> = {},
) {
	return {
		trigger: trigger({ Events: [] }),
		events: EVENTS,
		errors: [],
		onClose: vi.fn(),
		onChange: vi.fn(),
		...overrides,
	}
}

describe('TriggerConfigPanel — listing', () => {
	it('groups the events by prefix', () => {
		render(<TriggerConfigPanel {...baseProps()} />)

		expect(screen.getByText('quote')).toBeDefined()
		expect(screen.getByText('invoice')).toBeDefined()
		expect(
			screen.getByRole('checkbox', { name: 'Devis accepté' }),
		).toBeDefined()
		expect(
			screen.getByRole('checkbox', { name: 'Facture payée' }),
		).toBeDefined()
	})

	it('pre-checks the events the trigger currently subscribes to', () => {
		render(
			<TriggerConfigPanel
				{...baseProps({ trigger: trigger({ Events: ['quote.accepted'] }) })}
			/>,
		)

		expect(
			screen
				.getByRole('checkbox', { name: 'Devis accepté' })
				.getAttribute('aria-checked'),
		).toBe('true')
		expect(
			screen
				.getByRole('checkbox', { name: 'Devis envoyé' })
				.getAttribute('aria-checked'),
		).toBe('false')
	})

	it('names the trigger in the header', () => {
		render(
			<TriggerConfigPanel
				{...baseProps({ trigger: trigger({ Events: [] }, 't7') })}
			/>,
		)

		expect(screen.getByText('Déclencheur t7')).toBeDefined()
	})

	it('closes on request', async () => {
		const user = userEvent.setup()
		const onClose = vi.fn()
		render(<TriggerConfigPanel {...baseProps({ onClose })} />)

		await user.click(screen.getByRole('button', { name: /Fermer/ }))

		expect(onClose).toHaveBeenCalledTimes(1)
	})
})

describe('TriggerConfigPanel — the no-event warning', () => {
	it('warns as soon as the trigger has no event selected', () => {
		render(
			<TriggerConfigPanel
				{...baseProps({ trigger: trigger({ Events: [] }) })}
			/>,
		)

		expect(screen.getByText(/ne partira jamais/)).toBeDefined()
	})

	it('disappears once the trigger prop carries at least one event', () => {
		const { rerender } = render(
			<TriggerConfigPanel
				{...baseProps({ trigger: trigger({ Events: [] }) })}
			/>,
		)
		expect(screen.getByText(/ne partira jamais/)).toBeDefined()

		rerender(
			<TriggerConfigPanel
				{...baseProps({ trigger: trigger({ Events: ['quote.accepted'] }) })}
			/>,
		)

		expect(screen.queryByText(/ne partira jamais/)).toBeNull()
	})

	it('never shows for a manual trigger, even though it carries no event', () => {
		render(
			<TriggerConfigPanel {...baseProps({ trigger: trigger('Manual') })} />,
		)

		expect(screen.queryByText(/ne partira jamais/)).toBeNull()
	})
})

describe('TriggerConfigPanel — the mode choice', () => {
	it('offers both modes explicitly', () => {
		render(<TriggerConfigPanel {...baseProps()} />)

		expect(screen.getByRole('tab', { name: 'Sur événement(s)' })).toBeDefined()
		expect(screen.getByRole('tab', { name: 'Manuel' })).toBeDefined()
	})

	it('starts on the events tab for an events trigger, listing its checkboxes', () => {
		render(
			<TriggerConfigPanel
				{...baseProps({ trigger: trigger({ Events: ['quote.accepted'] }) })}
			/>,
		)

		expect(
			screen.getByRole('checkbox', { name: 'Devis accepté' }),
		).toBeDefined()
	})

	it('starts on the manual tab for a manual trigger, hiding the event list', () => {
		render(
			<TriggerConfigPanel {...baseProps({ trigger: trigger('Manual') })} />,
		)

		expect(screen.queryByRole('checkbox', { name: 'Devis accepté' })).toBeNull()
		expect(screen.getByText(/Exécuter maintenant/)).toBeDefined()
	})

	it('reports switching to manual as just "Manual", discarding the selection immediately', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<TriggerConfigPanel
				{...baseProps({
					trigger: trigger({ Events: ['quote.accepted'] }),
					onChange,
				})}
			/>,
		)

		await user.click(screen.getByRole('tab', { name: 'Manuel' }))

		expect(onChange).toHaveBeenCalledWith('Manual')
	})

	it('reports switching back to events with no events selected, since manual remembers none', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<TriggerConfigPanel
				{...baseProps({ trigger: trigger('Manual'), onChange })}
			/>,
		)

		await user.click(screen.getByRole('tab', { name: 'Sur événement(s)' }))

		expect(onChange).toHaveBeenCalledWith({ Events: [] })
	})
})

describe('TriggerConfigPanel — editing the event selection', () => {
	it('reports the full selection, additively checked, as one replacement', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<TriggerConfigPanel
				{...baseProps({
					trigger: trigger({ Events: ['quote.accepted'] }),
					onChange,
				})}
			/>,
		)

		await user.click(screen.getByRole('checkbox', { name: 'Facture payée' }))

		expect(onChange).toHaveBeenCalledWith({
			Events: ['quote.accepted', 'invoice.paid'],
		})
	})

	it('reports an empty array once the only selected event is unchecked', async () => {
		const user = userEvent.setup()
		const onChange = vi.fn()
		render(
			<TriggerConfigPanel
				{...baseProps({
					trigger: trigger({ Events: ['quote.accepted'] }),
					onChange,
				})}
			/>,
		)

		await user.click(screen.getByRole('checkbox', { name: 'Devis accepté' }))

		expect(onChange).toHaveBeenCalledWith({ Events: [] })
	})
})

describe('TriggerConfigPanel — validation errors', () => {
	it('banners every error message passed in', () => {
		render(
			<TriggerConfigPanel
				{...baseProps({
					trigger: trigger({ Events: ['quote.accepted'] }),
					errors: [
						{ field: null, message: 'Ce déclencheur ne mène nulle part.' },
					],
				})}
			/>,
		)

		expect(screen.getByText('Ce déclencheur ne mène nulle part.')).toBeDefined()
	})

	it('shows no error banner when there are none', () => {
		render(
			<TriggerConfigPanel
				{...baseProps({
					trigger: trigger({ Events: ['quote.accepted'] }),
					errors: [],
				})}
			/>,
		)

		expect(screen.queryByRole('alert')).toBeNull()
	})
})
