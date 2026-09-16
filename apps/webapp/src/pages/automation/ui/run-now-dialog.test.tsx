import { fireEvent, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactElement } from 'react'
import { describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import type { RunNowDialogProps } from '#/pages/automation/ui/run-now-dialog'
import { RunNowDialog } from '#/pages/automation/ui/run-now-dialog'
import {
	renderWithPermissions,
	wrapWithPermissions,
} from '#/test/with-permissions'

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

function renderDialog(
	ui: ReactElement,
	permissions: string[] = ['MANAGE_AUTOMATION'],
) {
	return renderWithPermissions(ui, { permissions })
}

function event(
	name: string,
	label: string,
	payloadExample: unknown,
): Schemas.EventDescriptorResponse {
	return {
		name,
		label,
		subject_kind: name.split('.')[0] ?? name,
		version: 1,
		payload_example: payloadExample,
	}
}

function baseProps(
	overrides: Partial<RunNowDialogProps> = {},
): RunNowDialogProps {
	return {
		open: true,
		events: [],
		triggerEventNames: [],
		isStarting: false,
		startError: null,
		onOpenChange: vi.fn(),
		onConfirm: vi.fn(),
		...overrides,
	}
}

describe('RunNowDialog — visibility', () => {
	it('renders nothing when closed', () => {
		renderDialog(<RunNowDialog {...baseProps({ open: false })} />)

		expect(screen.queryByText('Exécuter maintenant')).toBeNull()
	})
})

describe('RunNowDialog — the warning', () => {
	it('warns that the run is real and irreversible before anything starts', () => {
		renderDialog(<RunNowDialog {...baseProps()} />)

		expect(screen.getByRole('alert').textContent).toMatch(
			/agiront pour de vrai/,
		)
		expect(screen.getByRole('alert').textContent).toMatch(
			/ne peut pas être annulée/,
		)
	})
})

describe('RunNowDialog — the payload', () => {
	it('pre-fills the payload from the first subscribed event’s example', () => {
		renderDialog(
			<RunNowDialog
				{...baseProps({
					events: [
						event('quote.accepted', 'Devis accepté', { quote_id: 'q-1' }),
					],
					triggerEventNames: ['quote.accepted'],
				})}
			/>,
		)

		const textarea = screen.getByLabelText(
			'Charge utile (JSON)',
		) as HTMLTextAreaElement
		expect(JSON.parse(textarea.value)).toEqual({ quote_id: 'q-1' })
	})

	it('offers no event picker and starts with an empty payload when nothing is subscribed', () => {
		renderDialog(
			<RunNowDialog {...baseProps({ events: [], triggerEventNames: [] })} />,
		)

		expect(screen.queryByRole('combobox')).toBeNull()
		const textarea = screen.getByLabelText(
			'Charge utile (JSON)',
		) as HTMLTextAreaElement
		expect(JSON.parse(textarea.value)).toEqual({})
	})

	it('resets the payload to the newly selected event’s example', async () => {
		const user = userEvent.setup()
		renderDialog(
			<RunNowDialog
				{...baseProps({
					events: [
						event('quote.accepted', 'Devis accepté', { quote_id: 'q-1' }),
						event('invoice.paid', 'Facture payée', { invoice_id: 'i-9' }),
					],
					triggerEventNames: ['quote.accepted', 'invoice.paid'],
				})}
			/>,
		)

		await user.click(
			screen.getByRole('combobox', { name: 'Événement déclencheur' }),
		)
		await user.click(
			await screen.findByRole('option', { name: 'Facture payée' }),
		)

		const textarea = screen.getByLabelText(
			'Charge utile (JSON)',
		) as HTMLTextAreaElement
		expect(JSON.parse(textarea.value)).toEqual({ invoice_id: 'i-9' })
	})

	it('lets the payload be edited freely', () => {
		renderDialog(
			<RunNowDialog
				{...baseProps({
					events: [event('quote.accepted', 'Devis accepté', { id: 1 })],
					triggerEventNames: ['quote.accepted'],
				})}
			/>,
		)

		const textarea = screen.getByLabelText('Charge utile (JSON)')
		fireEvent.change(textarea, { target: { value: '{"id":2}' } })

		expect((textarea as HTMLTextAreaElement).value).toBe('{"id":2}')
	})
})

describe('RunNowDialog — confirming', () => {
	it('parses the payload and confirms with the parsed value', async () => {
		const user = userEvent.setup()
		const onConfirm = vi.fn()
		renderDialog(
			<RunNowDialog
				{...baseProps({
					events: [event('quote.accepted', 'Devis accepté', { id: 1 })],
					triggerEventNames: ['quote.accepted'],
					onConfirm,
				})}
			/>,
		)

		await user.click(
			screen.getByRole('button', { name: 'Exécuter maintenant' }),
		)

		expect(onConfirm).toHaveBeenCalledWith({ id: 1 })
	})

	it('refuses invalid JSON, shows an error, and never confirms', async () => {
		const user = userEvent.setup()
		const onConfirm = vi.fn()
		renderDialog(<RunNowDialog {...baseProps({ onConfirm })} />)

		const textarea = screen.getByLabelText('Charge utile (JSON)')
		fireEvent.change(textarea, { target: { value: '{not json' } })
		await user.click(
			screen.getByRole('button', { name: 'Exécuter maintenant' }),
		)

		expect(
			screen.getByText('Cette charge utile n’est pas un JSON valide.'),
		).toBeDefined()
		expect(onConfirm).not.toHaveBeenCalled()
	})

	it('shows the pending label and disables confirm while starting', () => {
		renderDialog(<RunNowDialog {...baseProps({ isStarting: true })} />)

		const button = screen.getByRole('button', { name: 'Exécution…' })
		expect((button as HTMLButtonElement).disabled).toBe(true)
	})

	it('shows the start error when one is given', () => {
		renderDialog(
			<RunNowDialog {...baseProps({ startError: 'Le démarrage a échoué.' })} />,
		)

		expect(screen.getByText('Le démarrage a échoué.')).toBeDefined()
	})

	it('cancels without confirming', async () => {
		const user = userEvent.setup()
		const onOpenChange = vi.fn()
		const onConfirm = vi.fn()
		renderDialog(<RunNowDialog {...baseProps({ onOpenChange, onConfirm })} />)

		await user.click(screen.getByRole('button', { name: 'Annuler' }))

		expect(onOpenChange).toHaveBeenCalledWith(false)
		expect(onConfirm).not.toHaveBeenCalled()
	})
})

describe('RunNowDialog — reopening', () => {
	it('resets the payload to the default example when reopened after a different edit', async () => {
		const { rerender } = renderDialog(
			<RunNowDialog
				{...baseProps({
					events: [event('quote.accepted', 'Devis accepté', { id: 1 })],
					triggerEventNames: ['quote.accepted'],
				})}
			/>,
		)

		const textarea = screen.getByLabelText('Charge utile (JSON)')
		fireEvent.change(textarea, { target: { value: '{"id":999}' } })

		rerender(
			wrapWithPermissions(
				<RunNowDialog
					{...baseProps({
						open: false,
						events: [event('quote.accepted', 'Devis accepté', { id: 1 })],
						triggerEventNames: ['quote.accepted'],
					})}
				/>,
				{ permissions: ['MANAGE_AUTOMATION'] },
			),
		)
		rerender(
			wrapWithPermissions(
				<RunNowDialog
					{...baseProps({
						events: [event('quote.accepted', 'Devis accepté', { id: 1 })],
						triggerEventNames: ['quote.accepted'],
					})}
				/>,
				{ permissions: ['MANAGE_AUTOMATION'] },
			),
		)

		await waitFor(() => {
			const reopened = screen.getByLabelText(
				'Charge utile (JSON)',
			) as HTMLTextAreaElement
			expect(JSON.parse(reopened.value)).toEqual({ id: 1 })
		})
	})
})

describe('RunNowDialog — permission gating', () => {
	it('offers no confirm button to a caller without MANAGE_AUTOMATION', () => {
		renderDialog(<RunNowDialog {...baseProps()} />, [])

		expect(
			screen.queryByRole('button', { name: 'Exécuter maintenant' }),
		).toBeNull()
		expect(screen.getByRole('button', { name: 'Annuler' })).toBeDefined()
	})
})
