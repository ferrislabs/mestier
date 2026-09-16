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

function baseProps(
	overrides: Partial<Parameters<typeof TriggerConfigPanel>[0]> = {},
) {
	return {
		events: EVENTS,
		mode: 'events' as const,
		selectedEventNames: [],
		isSaving: false,
		saveError: null,
		onClose: vi.fn(),
		onSave: vi.fn(),
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

	it('pre-checks the currently selected events', () => {
		render(
			<TriggerConfigPanel
				{...baseProps({ selectedEventNames: ['quote.accepted'] })}
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

	it('closes on request', async () => {
		const user = userEvent.setup()
		const onClose = vi.fn()
		render(<TriggerConfigPanel {...baseProps({ onClose })} />)

		await user.click(screen.getByRole('button', { name: /Fermer/ }))

		expect(onClose).toHaveBeenCalledTimes(1)
	})
})

describe('TriggerConfigPanel — the no-event warning', () => {
	it('warns as soon as the pending selection is empty', () => {
		render(<TriggerConfigPanel {...baseProps({ selectedEventNames: [] })} />)

		expect(screen.getByText(/ne se déclenchera jamais/)).toBeDefined()
	})

	it('clears the warning once at least one event is checked', async () => {
		const user = userEvent.setup()
		render(<TriggerConfigPanel {...baseProps({ selectedEventNames: [] })} />)

		await user.click(screen.getByRole('checkbox', { name: 'Devis accepté' }))

		expect(screen.queryByText(/ne se déclenchera jamais/)).toBeNull()
	})
})

describe('TriggerConfigPanel — the mode choice', () => {
	it('offers both modes explicitly', () => {
		render(<TriggerConfigPanel {...baseProps()} />)

		expect(screen.getByRole('tab', { name: 'Sur événement(s)' })).toBeDefined()
		expect(screen.getByRole('tab', { name: 'Manuel' })).toBeDefined()
	})

	it('switching to manual hides the event list and the no-event warning', async () => {
		const user = userEvent.setup()
		render(<TriggerConfigPanel {...baseProps({ selectedEventNames: [] })} />)
		expect(screen.getByText(/ne se déclenchera jamais/)).toBeDefined()

		await user.click(screen.getByRole('tab', { name: 'Manuel' }))

		expect(screen.queryByRole('checkbox', { name: 'Devis accepté' })).toBeNull()
		expect(screen.queryByText(/ne se déclenchera jamais/)).toBeNull()
	})

	it('starts on the workflow current mode', () => {
		render(<TriggerConfigPanel {...baseProps({ mode: 'manual' })} />)

		expect(screen.queryByRole('checkbox', { name: 'Devis accepté' })).toBeNull()
	})

	it('saves manual mode, discarding whatever was pending in the event list', async () => {
		const user = userEvent.setup()
		const onSave = vi.fn()
		render(
			<TriggerConfigPanel
				{...baseProps({ selectedEventNames: ['quote.accepted'], onSave })}
			/>,
		)

		await user.click(screen.getByRole('tab', { name: 'Manuel' }))
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		expect(onSave).toHaveBeenCalledWith('manual', [])
	})
})

describe('TriggerConfigPanel — saving', () => {
	it('saves the full pending selection, additively checked, as one replacement', async () => {
		const user = userEvent.setup()
		const onSave = vi.fn()
		render(
			<TriggerConfigPanel
				{...baseProps({ selectedEventNames: ['quote.accepted'], onSave })}
			/>,
		)

		await user.click(screen.getByRole('checkbox', { name: 'Facture payée' }))
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		expect(onSave).toHaveBeenCalledWith('events', [
			'quote.accepted',
			'invoice.paid',
		])
	})

	it('saves an empty array when every event is unchecked, clearing the trigger', async () => {
		const user = userEvent.setup()
		const onSave = vi.fn()
		render(
			<TriggerConfigPanel
				{...baseProps({ selectedEventNames: ['quote.accepted'], onSave })}
			/>,
		)

		await user.click(screen.getByRole('checkbox', { name: 'Devis accepté' }))
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		expect(onSave).toHaveBeenCalledWith('events', [])
	})

	it('disables the save button while a save is in flight', () => {
		render(<TriggerConfigPanel {...baseProps({ isSaving: true })} />)

		const button = screen.getByRole('button', {
			name: /Enregistrement/,
		}) as HTMLButtonElement
		expect(button.disabled).toBe(true)
	})

	it('banners a save error', () => {
		render(
			<TriggerConfigPanel
				{...baseProps({ saveError: 'La sauvegarde a échoué' })}
			/>,
		)

		expect(screen.getByText('La sauvegarde a échoué')).toBeDefined()
	})
})
