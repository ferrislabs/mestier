import { render, screen, within } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { ScheduleInputPanel } from '#/pages/automation/ui/schedule-input-panel'
import { WebhookInputPanel } from '#/pages/automation/ui/webhook-input-panel'

describe('WebhookInputPanel', () => {
	function renderPanel() {
		return render(
			<WebhookInputPanel
				method="POST"
				url="https://mestier.test/webhooks/paiement-recu"
				contract={'export interface PaiementRecuBody {\n  facture: string\n}'}
				example={{ facture: 'FA-1' }}
			/>,
		)
	}

	it('is the « Entrée » column, captioned « Requête attendue »', () => {
		renderPanel()

		const panel = screen.getByRole('region', { name: 'Entrée' })
		expect(within(panel).getByText('Requête attendue')).toBeDefined()
	})

	it('shows the method and the URL on one line', () => {
		renderPanel()

		expect(
			screen.getByText(
				(_, element) =>
					element?.tagName === 'P' &&
					element.textContent ===
						'POST https://mestier.test/webhooks/paiement-recu',
			),
		).toBeDefined()
	})

	it('offers the TypeScript contract and the example of the request in two tabs', () => {
		renderPanel()

		expect(
			screen.getByRole('tab', { name: 'Contrat TypeScript' }),
		).toBeDefined()
		expect(
			screen.getByRole('tab', { name: 'Exemple de requête' }),
		).toBeDefined()
		expect(screen.getByText(/PaiementRecuBody/)).toBeDefined()
	})
})

describe('ScheduleInputPanel', () => {
	it('lists the four keys of the payload and shows an example of it', () => {
		render(
			<ScheduleInputPanel
				contract="export interface SchedulePayload {}"
				example={{ jour: 'jeudi' }}
			/>,
		)

		for (const key of ['scheduled_for', 'horodatage', 'jour', 'fuseau'])
			expect(screen.getByText(key)).toBeDefined()
		expect(
			screen.getByRole('tab', { name: 'Exemple de payload' }),
		).toBeDefined()
		expect(
			screen.getByText('Le jour de la semaine, en français.'),
		).toBeDefined()
	})
})
