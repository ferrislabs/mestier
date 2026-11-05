import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { TotalsPanel } from './totals-panel'

describe('TotalsPanel', () => {
	it('reads HT, each VAT rate charged, then TTC', () => {
		render(
			<TotalsPanel
				totals={{
					netCents: 31500,
					vat: [
						{ rateBp: 1000, vatCents: 30 },
						{ rateBp: 2000, vatCents: 2400 },
					],
					grossCents: 33930,
				}}
			/>,
		)

		expect(screen.getByText('Total HT')).toBeDefined()
		expect(screen.getByText('315,00 €')).toBeDefined()
		expect(screen.getByText('TVA 10 %')).toBeDefined()
		expect(screen.getByText('0,30 €')).toBeDefined()
		expect(screen.getByText('TVA 20 %')).toBeDefined()
		expect(screen.getByText('24,00 €')).toBeDefined()
		expect(screen.getByText('Total TTC')).toBeDefined()
		expect(screen.getByText('339,30 €')).toBeDefined()
	})

	it('explains the missing VAT line rather than leaving it unsaid', () => {
		render(
			<TotalsPanel
				totals={{ netCents: 10000, vat: [], grossCents: 10000 }}
				vatNotice="TVA non applicable, art. 293 B du CGI"
			/>,
		)

		expect(
			screen.getByText('TVA non applicable, art. 293 B du CGI'),
		).toBeDefined()
	})

	it('surfaces a notice about the figures it is showing', () => {
		render(
			<TotalsPanel
				totals={{ netCents: 10000, vat: [], grossCents: 10000 }}
				notice="Estimation, non enregistrée"
			/>,
		)

		expect(screen.getByText('Estimation, non enregistrée')).toBeDefined()
	})

	it('labels the gross total of a credit note', () => {
		render(
			<TotalsPanel
				totals={{ netCents: 10000, vat: [], grossCents: 10000 }}
				grossLabel="Total TTC crédité"
			/>,
		)

		expect(screen.getByText('Total TTC crédité')).toBeDefined()
	})
})
